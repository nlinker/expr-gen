
use std::fmt::{Display, Formatter, Error};
use std::iter::Peekable;

#[derive(Copy, Clone, PartialEq, Debug)]
enum Ary {
    Unary,
    Binary,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Assoc {
    Left,
    Right,
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Op {
    symbol: char,
    ary: Ary,
    assoc: Assoc,
    prec: u8,
}

#[derive(Clone, PartialEq, Debug)]
enum Token {
    Number(i32),
    RealVar(String),
    BoolVar(String),
    Operator(Op),
    LeftBracket,
    RightBracket,
    Function(String),
}

impl Token {
    fn binary_left_assoc(c: char, prec: u8) -> Token {
        Token::Operator(
            Op {
                ary: Ary::Binary,
                symbol: c,
                assoc: Assoc::Left,
                prec,
            })
    }
    fn unary_right_assoc(c: char, prec: u8) -> Token {
        Token::Operator(
            Op {
                ary: Ary::Unary,
                symbol: c,
                assoc: Assoc::Right,
                prec,
            })
    }

    fn is_operator(&self) -> bool {
        match self {
            Token::Operator(_) => true,
            _ => false
        }
    }

    fn is_left_paren(&self) -> bool {
        match self {
            Token::LeftBracket => true,
            _ => false
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::RealVar(id) => write!(f, "{}", id),
            Token::BoolVar(id) => write!(f, "{}", id),
            Token::Operator(op) => if op.ary == Ary::Unary {
                // unary `-` and `+` we display as `m` and `p`
                let x = match op.symbol {
                    '-' => 'm',
                    '+' => 'p',
                    _ => op.symbol
                };
                write!(f, "{}", x)
            } else {
                write!(f, "{}", op.symbol)
            },
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::Function(name) => write!(f, "{}", name),
        }
    }
}

fn lex(input: &str) -> Result<Vec<Token>, char> {
    let mut result: Vec<Token> = Vec::with_capacity(input.len());
    let mut it = input.chars().peekable();
    // set to true if some operand have seen
    let mut binary = false;

    while let Some(c) = it.next() {
        // if the current char is whitespace, just skip
        if !c.is_whitespace() {
            let t = match c {
                '0'..='9' => Token::Number(get_number(&mut it, c)),
                '+' | '-' => if binary { Token::binary_left_assoc(c, 5) } else { Token::unary_right_assoc(c, 8) },
                '*' | '/' => Token::binary_left_assoc(c, 7),
                '^'       => Token::binary_left_assoc(c, 6),
                '<' | '>' => Token::binary_left_assoc(c, 4),
                '=' | '#' => Token::binary_left_assoc(c, 4),
                '~'       => Token::unary_right_assoc(c, 3),
                '&'       => Token::binary_left_assoc(c, 2),
                '!'       => Token::binary_left_assoc(c, 2),
                'R'       => Token::RealVar(get_identifier(&mut it, c)),
                'B'       => Token::BoolVar(get_identifier(&mut it, c)),
                '[' | '(' => Token::LeftBracket,
                ']' | ')' => Token::RightBracket,
                _ => return Err(c),
            };
            // if we have seen an operand, then we expect the binary operator
            binary = match c {
                '0'..='9' => true,
                'R' => true,
                'B' => true,
                ']' => true,
                _ => false,
            };
            result.push(t);
        }
    }
    Ok(result)
}

fn get_number<I: Iterator<Item = char>>(it: &mut Peekable<I>, c: char) -> i32 {
    let mut n = c.to_digit(10).expect("gen_number invariant violation") as i32;
    while let Some(Some(k)) = it.peek().map(|c| c.to_digit(10)) {
        n = n * 10 + (k as i32);
        it.next();
    }
    n
}

fn get_identifier<I: Iterator<Item = char>>(it: &mut Peekable<I>, c: char) -> String {
    let mut id = c.to_string();
    while let Some(s) = it.peek().filter(|c| c.is_alphanumeric()) {
        id += &(s.to_string());
        it.next();
    }
    id
}

fn rpn(input: &[Token], debug: bool) -> Vec<&Token> {
    let mut stack: Vec<&Token> = Vec::new(); // holds operators and left brackets
    let mut output: Vec<&Token> = Vec::with_capacity(input.len());
    println!("{:30}{:30}{:10}", "output", "stack", "token");
    for token in input {
        let str_stack = stack.iter().map(|t| format!(" {}", t)).collect::<String>();
        let str_output = output.iter().map(|t| format!(" {}", t)).collect::<String>();
        println!("{:30}{:30}{:10}", str_output, str_stack, token);
        match token {
            Token::Number(_) => output.push(token),
            Token::RealVar(_) => output.push(token),
            Token::BoolVar(_) => output.push(token),
            Token::LeftBracket => stack.push(token),
            Token::Operator(op) => {
                loop {
                    if let Some(&t) = stack.last() {
                        if !t.is_left_paren() {
                            // the top of the stack is not a left parenthesis AND
                            let cond = match t {
                                // the top of the stack is a function
                                Token::Function(_) => true,
                                // the top of the stack has greater precedence than op
                                Token::Operator(top) if top.prec > op.prec => true,
                                // the top of the stack has equal precedence to op and is left associative
                                Token::Operator(top) if top.prec == op.prec && top.assoc == Assoc::Left => true,
                                // otherwise
                                _ => false
                            };
                            if cond {
                                // pop operators from the operator stack onto the output queue
                                // we know for sure the stack.pop() doesn't fail here
                                output.push(stack.pop().unwrap());
                            } else {
                                break; // the conditions are not satisfied
                            }
                        } else {
                            break; // if left parenthesis
                        }
                    } else {
                        break; // if stack is empty
                    }
                }
                stack.push(token)
            },
            Token::RightBracket => {
                loop {
                    // while the operator at the top of the operator stack is not a left paren
                    if let Some(&t) = stack.last() {
                        if !t.is_left_paren() {
                            // we know for sure the top exists and is not a LeftBracket
                            output.push(stack.pop().unwrap());
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            Token::Function(_) => stack.push(token),
        };
    }
    // after the loop, if operator stack not empty, pop everything to output queue
    if !stack.is_empty() {
        while let Some(t) = stack.pop() {
            output.push(t);
        }
    }
    output
}

fn main() {
//    let input = "- 1 * - [2 + 3] + 4 * [- 5 * 6] + 7 * - 8";
    let input = "3 + 4 * 2 / ( 1 - 5 ) ^ 6 ^ 7";
    let tokens = lex(input).unwrap();
    let rpn = rpn(&tokens[..], true);
    let output = (&rpn).into_iter().map(|t| format!(" {}", t)).collect::<String>();
    println!("--------------------------");
    println!("input: {}", input);
    println!("output: {}", output);
}

#[cfg(test)]
mod tests {
    use super::Op;
    use super::Token::*;
    use crate::{lex, rpn, Token};
    use crate::Ary::*;
    use crate::Assoc::*;

    #[test]
    fn unary_detect() {
        let expected = vec![
            Operator(Op { symbol: '-', ary: Unary, assoc: Right, prec: 8 }),
            Operator(Op { symbol: '+', ary: Unary, assoc: Right, prec: 8 }),
            Operator(Op { symbol: '-', ary: Unary, assoc: Right, prec: 8 }),
            Number(1),
            Operator(Op { symbol: '*', ary: Binary, assoc: Left, prec: 7 }),
            Operator(Op { symbol: '-', ary: Unary, assoc: Right, prec: 8 }),
            Operator(Op { symbol: '+', ary: Unary, assoc: Right, prec: 8 }),
            Operator(Op { symbol: '-', ary: Unary, assoc: Right, prec: 8 }),
            RealVar("R1".to_string()),
        ];
        assert_eq!(lex("- + - 1 * - + - R1").unwrap(), expected);
    }

    #[test]
    fn long_unary_chains() {
        let input = "+ - - 1 * + - - 2";
        let tokens = lex(input).unwrap();
        let expected = vec![
            Number(1),
            Operator(Op { symbol: '-', ary: Unary, assoc: Right, prec: 8 }),
            Operator(Op { symbol: '-', ary: Unary, assoc: Right, prec: 8 }),
            Operator(Op { symbol: '+', ary: Unary, assoc: Right, prec: 8 }),
            Number(2),
            Operator(Op { symbol: '-', ary: Unary, assoc: Right, prec: 8 }),
            Operator(Op { symbol: '-', ary: Unary, assoc: Right, prec: 8 }),
            Operator(Op { symbol: '+', ary: Unary, assoc: Right, prec: 8 }),
            Operator(Op { symbol: '*', ary: Binary, assoc: Left, prec: 7 }),
        ];
        let actual: Vec<Token> = rpn(&tokens[..], false)
            .iter()
            .map(|t| *t)
            .cloned()
            .collect();
        assert_eq!(actual, expected);
    }
}