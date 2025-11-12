#!/usr/bin/env runhaskell

import System.Directory
import System.Process
import System.FilePath
import Data.Maybe
import Data.List (isSuffixOf, unsnoc)
import Control.Monad

main :: IO ()
main = do
    sources <- filesWithExtension ".c" "src"

    buildExists <- doesDirectoryExist "build"
    unless buildExists (createDirectory "build")

    maybeBins <- forM sources buildSourceFile
    let bins = maybeBins >>= maybeToList
    projDir <- getCurrentDirectory

    let name = case pathBasename projDir of
        { Nothing -> "a.out";
        ; Just basename -> basename
        }

    putStrLn $ "[compile] build/* -> " ++ name
    _ <- readProcess "gcc" ("-o" : name : bins ++ ["-I", "include/"]) ""
    pure ()

pathBasename :: FilePath -> Maybe String
pathBasename path = snd <$> unsnoc (splitPath path)

-- | Builds a source file into the "build" directory and returns its build artifact path.
buildSourceFile :: FilePath -> IO (Maybe FilePath)
buildSourceFile source = case pathBasename source of
    Nothing -> pure Nothing
    Just name -> do
        let bin = "build/" ++ name ++ ".o"
        let args = ["-c", "-o", bin, source, "-I", "include/"]

        putStrLn $ "[compile] " ++ source ++ " -> " ++ bin
        _ <- readProcess "gcc" args ""
        pure $ Just bin

-- | Recursively find files in the given `FilePath` that have the given file extension. The file
-- extension _should_ include the leading dot. i.e. ".c"
filesWithExtension :: String -> FilePath -> IO [FilePath]
filesWithExtension ext dir = do
    entries <- listDirectory dir
    filtered <- forM (fmap ((dir ++ "/") ++) entries) (filterRec ext)
    pure (concat filtered)
  where
    filterRec = \ext path -> do
        isDir <- doesDirectoryExist path
        isFile <- doesFileExist path

        if isDir
            then filesWithExtension ext path
            else if isFile && (ext `isSuffixOf` path)
                then pure [path]
                else pure []
