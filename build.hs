#!/usr/bin/env runhaskell

import System.Directory
import System.Process
import System.FilePath
import Data.Maybe
import Data.List (isSuffixOf, isPrefixOf, unsnoc)
import Control.Monad

includeArgs :: [String]
includeArgs = ["-I", "include/", "-I", "lib/include/"]

includeArgsExport :: [String]
includeArgsExport = ["-I", "include/", "-I", "lib/include/", "-I", "export/include/"]

main :: IO ()
main = do
    sources <- filesWithExtension ".c" "src"

    buildExists <- doesDirectoryExist "build"
    unless buildExists (createDirectory "build")

    maybeBins <- forM sources (buildSourceFile "build/" includeArgs)
    let bins = maybeBins >>= maybeToList
    projDir <- getCurrentDirectory

    let name = case pathBasename projDir of
        { Nothing -> "a.out";
        ; Just basename -> basename
        }

    libraries <- filesWithExtension ".a" "lib"
    let libFilenames = mapMaybe pathBasename libraries
    let libs = ("-l" ++) <$> mapMaybe libName libFilenames

    forM libraries (\l -> putStrLn ("[link] " ++ l))

    putStrLn $ "[compile] build/* -> " ++ name
    _ <- readProcess "gcc" ("-o" : name : "-Llib/" : libs ++ bins ++ includeArgs) ""


    -- Build static library export
    
    exportExists <- doesDirectoryExist "export"

    if not exportExists then pure () else do

        exSources <- filesWithExtension ".c" "export"

        exBuildExists <- doesDirectoryExist "build/export"
        unless exBuildExists (createDirectory "build/export")

        maybeExBins <- forM exSources (buildSourceFile "build/export/" includeArgsExport)
        let exBins = maybeExBins >>= maybeToList
        let objectName = name ++ ".a.o"
        let archiveName = "lib" ++ name ++ ".a"

        putStrLn $ "[compile] build/export/* -> " ++ objectName
        _ <- readProcess "gcc" ("-o" : ("build/export/" ++ name ++ ".a.o") : "-Llib/" : libs ++ bins ++ includeArgsExport) ""
        putStrLn $ "[archive] build/export/" ++ objectName ++ " -> build/export/" ++ archiveName
        _ <- readProcess "ar" ("rcs" : ("build/export/" ++ archiveName) : ["build/export/" ++ objectName]) ""

        pure ()

pathBasename :: FilePath -> Maybe String
pathBasename path = snd <$> unsnoc (splitPath path)

libName :: FilePath -> Maybe String
libName ('l' : 'i' : 'b' : name) = Just $ remove ".a" name
  where
    remove w "" = ""
    remove w s@(c:cs) 
        | w `isPrefixOf` s = remove w (drop (length w) s)
        | otherwise = c : remove w cs
libName _ = Nothing

-- | Builds a source file into the "build" directory and returns its build artifact path.
buildSourceFile :: String -> [String] -> FilePath -> IO (Maybe FilePath)
buildSourceFile dir includes source = case pathBasename source of
    Nothing -> pure Nothing
    Just name -> do
        let bin = dir ++ name ++ ".o"
        let args = ["-c", "-o", bin, source] ++ includes

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
