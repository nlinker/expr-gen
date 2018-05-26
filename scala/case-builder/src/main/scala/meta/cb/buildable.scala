package meta.cb

import scala.annotation.compileTimeOnly
import scala.annotation.StaticAnnotation
import scala.collection.immutable.Seq
import scala.meta._

// type classes that mappable will generate for annotated classes
trait ToMap[A] { def apply(a: A): Map[String, Any] }
trait FromMap[A] { def apply(keyValues: Map[String, Any]): Option[A] }

@compileTimeOnly("@meta.cb.buildable not expanded")
class buildable extends StaticAnnotation {
  inline def apply(defn: Any): Any = meta {

    // defined class may or may not have a companion object
    val (classDefn: Defn.Class, compDefnOption: Option[Defn.Object]) = defn match {
      case classDefn: Defn.Class => (classDefn, None) //only class
      case Term.Block((classDefn: Defn.Class) :: (compDefn: Defn.Object) :: Nil) => (classDefn, Option(compDefn)) // class + companion
      case _ => abort(defn.pos, "Invalid annottee")
    }

    // get existing companion object statements (if any)
    val compStats: Seq[Stat] = compDefnOption match {
      case None => Nil
      case Some(compDefn) => compDefn.templ.stats.getOrElse(Nil)
    }

    val q"..$mods class $tName[..$tParams] ..$ctorMods (...$paramss) extends $template" = classDefn

    val typeTermName = Term.Name(tName.value)

    val tParamTypes: Seq[Type] = tParams map Helpers.toType
    val tCompleteTerm: Term =
      if (tParamTypes.isEmpty) q"$typeTermName"
      else q"$typeTermName[..$tParamTypes]"
    val tCompleteType: Type = Helpers.toType(tCompleteTerm)
    val tCompleteTypeOption: Type = Helpers.toType(q"Option[$tCompleteType]")

    object ToMapImpl {
      val mappableName: Term.Name = q"mappable"
      val paramssFlat: Seq[Term.Param] = paramss.flatten
      def keyValues(mappableName: Term.Name): Seq[Term] = paramssFlat.map { param =>
        val memberName = Term.Name(param.name.value)
        q"${param.name.value} -> $mappableName.$memberName"
      }
    }

    object FromMapImpl {
      val ctorValuesName: Term.Name = q"values"

      // get default value and store those value as a map in object
      val defaultValue:  Seq[Term.ApplyInfix] = paramss.flatten collect {
        case param if param.default.nonEmpty =>
          q"""${param.name.value} -> ${param.default.get}"""
      }

    // TODO: support multiple constructor params lists
      val ctorParamsFirst: Seq[Term.Param] = paramss.headOption.getOrElse(Nil)
      def ctorArgs(valuesName: Term.Name): Seq[Term] = ctorParamsFirst.map { param =>
        val nameTerm = Term.Name(param.name.value)
        val tpe: Type = param.decltpe.get.asInstanceOf[Type.Name] // TODO: don't do option.get, don't cast
        q""" $nameTerm = $valuesName(${param.name.value}).asInstanceOf[$tpe] """
      }
    }

    val res = q"""
      ..$mods class $tName[..$tParams](...$paramss) extends $template

      object $typeTermName {
        val defaultValueMap: Map[String, Any] = Map(..${FromMapImpl.defaultValue})

        implicit def toMap[..$tParams] = new meta.cb.ToMap[$tCompleteType] {
          override def apply(${ToMapImpl.mappableName}: ${Option(tCompleteType)}): Map[String, Any] =
            Map[String, Any](..${ToMapImpl.keyValues(ToMapImpl.mappableName)})
        }

        implicit class ToMapOps[..$tParams](instance: $tCompleteType) {
          def toMap(implicit toMap: ToMap[$tCompleteType]): Map[String, Any] = toMap(instance)
        }

        def fromMap[..$tParams] = new meta.cb.FromMap[$tCompleteType] {
          override def apply(v: Map[String, Any]): ${Option(tCompleteTypeOption)} = {
              val values = defaultValueMap ++ v
              scala.util.Try {
                ${tCompleteTerm}(..${FromMapImpl.ctorArgs(FromMapImpl.ctorValuesName)})
              }.toOption
            }
        }

        ..$compStats
      }
    """


    // println(res)
    res
  }
}

object Helpers {
  def toType(term: Term): Type = term match {
    case name: Term.Name => Type.Name(name.value)
    case applyType: Term.ApplyType => Type.Apply(toType(applyType.fun), applyType.targs)
  }

  def toType(tparam: Type.Param): Type = Type.Name(tparam.name.value)
}
