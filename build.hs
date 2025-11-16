#!/usr/bin/env runhaskell

import System.Directory
import System.Process
import System.FilePath
import Data.Maybe
import Data.List (isSuffixOf, isPrefixOf, unsnoc)
import Control.Monad

-- | A path to a director containing header files which can be included in a C project.
newtype Include = Include { path :: FilePath }

sourceDirectory :: FilePath
sourceDirectory = "src"

buildDirectory :: FilePath
buildDirectory = "build"

libraryDirectory :: FilePath
libraryDirectory = "lib"

-- | Headers for the primary project. These will be included in the compilation of the executable
-- binary for this project.
projectHeaders :: [Include]
projectHeaders = [Include "include", Include "lib/include"]

-- | Headers which will be copied over to the build directory.
copyHeaders :: [Include]
copyHeaders = [Include "include"]

-- | Generate the GCC arguments to include headers from the given `Include`.
include :: Include -> String
include (Include path) = "-I" ++ path

-- | Generate the GCC arguments to include all headers from all given `Include`s.
includeAll :: [Include] -> [String]
includeAll = fmap include

main :: IO ()
main = do
    sources <- filesWithExtension ".c" sourceDirectory

    buildExists <- doesDirectoryExist buildDirectory
    unless buildExists (createDirectory buildDirectory)

    maybeBins <- forM sources (buildSourceFile (includeAll projectHeaders) buildDirectory)
    let bins = maybeBins >>= maybeToList
    projDir <- getCurrentDirectory

    let name = case pathBasename projDir of
        { Nothing -> "a.out";
        ; Just basename -> basename
        }

    libraries <- filesWithExtension ".a" libraryDirectory
    let libFilenames = mapMaybe pathBasename libraries
    let libs = ("-l" ++) <$> mapMaybe libName libFilenames

    forM libraries (\l -> putStrLn ("[link] " ++ l))

    putStrLn $ "[compile] " ++ buildDirectory ++ "/* -> " ++ name
    _ <- readProcess "gcc" ("-o" : name : "-Llib/" : libs ++ bins ++ (includeAll projectHeaders)) ""


    -- Build static library export
    
    exportSources <- filter (not . ("main.c" `isSuffixOf`)) <$> filesWithExtension ".c" sourceDirectory
    maybeExBins <- forM exportSources (buildSourceFile (includeAll projectHeaders) (buildDirectory ++ "/export"))
    let exBins = maybeExBins >>= maybeToList
    let objectName = name ++ ".a.o"
    let archiveName = "lib" ++ name ++ ".a"

    putStrLn $ "[compile] " ++ (buildDirectory ++ "/export") ++ "/* -> " ++ objectName
    _ <- readProcess
        "gcc"
        ( "-o"
        : (buildDirectory ++ "/export/" ++ name ++ ".a.o")
        : "-Llib/"
        : libs ++ bins ++ includeAll projectHeaders
        ) ""

    putStrLn $ "[archive] " ++ buildDirectory ++ "/export/" ++ objectName ++ " -> " ++ buildDirectory ++ "export/" ++ archiveName
    _ <- readProcess
        "ar"
        ( "rcs"
        : (buildDirectory ++ "/export/" ++ archiveName)
        : [buildDirectory ++ "/export/" ++ objectName]
        ) ""

    exIncludeExists <- doesDirectoryExist $ buildDirectory ++ "/export/include"
    unless exIncludeExists (createDirectory (buildDirectory ++ "/export/include"))

    headers <- mapM (filesWithExtension ".h") (path <$> copyHeaders)
    forM_ (concat headers) (\h -> case pathBasename h of
        Just base -> do
            putStrLn $ "[copy] " ++ h ++ " -> " ++ buildDirectory ++ "/export/include/" ++ base
            readProcess "cp" [h, buildDirectory ++ "/export/include/"] ""
        Nothing -> pure ""
        )

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

-- | Builds a C source file and produces an object file in the given directory. 
buildSourceFile :: [String] -> FilePath -> FilePath -> IO (Maybe FilePath)
buildSourceFile includes dir source = case pathBasename source of
    Nothing -> pure Nothing
    Just name -> do
        let bin = dir ++ "/" ++ name ++ ".o"
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
