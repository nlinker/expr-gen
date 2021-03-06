{-# LANGUAGE OverloadedStrings #-}

module Lib where

import Options.Applicative
import Data.Functor.Identity
import Data.List (intercalate)

data Arg = Arg { an :: Integer }

main :: IO ()
main = execParser opts >>= bookletCalc
  where
    opts = info (helper <*> parser)
     (  fullDesc
     <> progDesc "Calculate pages to print booklets"
     <> header "Make sure to have 'pages per side = 2' and landscape in printing dialog"
     )
parser :: Parser Arg
parser = Arg <$> argument auto (metavar "INTEGER" <> help "Number of pages to print")

bookletCalc :: Arg -> IO ()
bookletCalc arg = do
  let (front, back) = calc (an arg)
  putStrLn $ "front pages: " <> commaStr front
  putStrLn $ "back pages:  " <> commaStr back
  where
    commaStr xs = intercalate "," (map show xs)

-- | This is the calculation for booklet paging
calc :: Integer -> ([Integer], [Integer])
calc n' = calcDef n' n'

calcDef :: Integer -> Integer -> ([Integer], [Integer])
calcDef def n' =
  let n = if n' `mod` 4 == 0 then n' else (n' `div` 4 + 1) * 4 in
  let xs = [1..n'] ++ replicate (fromIntegral (n - n')) def in
  runIdentity $ split xs

split :: (Monad m, Show a) => [a] -> m ([a], [a])
split xxs = do
  let (x2, y2) = split' xxs
  let (fs, rs) = (reverse y2, x2)
  let us = concatMap (\x -> [fst x, snd x]) $ zip fs rs
  return $ splitAt (fromIntegral (length fs)) us
  where
    split' [] = ([],[])
    split' (x:y:xs) = let (x1s, y1s) = split' xs in (x:x1s, y:y1s)
    split' [x] = ([x], [x])
