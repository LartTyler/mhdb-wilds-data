@ECHO OFF
SETLOCAL enableextensions
PUSHD %~dp0

if "%~1"=="" (
    ECHO Usage: %~nx0 ^<directory^>
    EXIT /b 1
)

SET "dir=%~f1"
if "%dir:~-1%"=="\" SET "dir=%dir:~0,-1%"

CD .\data

FOR %%I in ("%dir%\*.pak") DO (
    CALL :extractpak %%I
)

POPD
PAUSE

EXIT /b 0

:extractpak
    SET "foldername=%~n1"

    REM Patches without data file changes ship as PAKs that are exactly 144 bytes.
    REM Weird choice, but we can just ignore those.
    if "%~z1"=="144" (
        ECHO Skipping %~1, empty data file.
        EXIT /b 0
    )

    REM If the destination folder exists and the PAK isn't newer, we don't need to process it again.
    if exist "%foldername%" (
        FOR %%A in ("%~1") DO SET pakdate=%%~tA
        FOR %%B in ("%foldername%") DO SET folderdate=%%~tB

        if "%pakdate%" LEQ "%folderdate%" (
            ECHO Skipping %~1, source is not newer.
            EXIT /b 0
        )
    )

    ..\tools\REtool\REtool.exe -h ..\tools\REtool\MHWs.list -skipUnknowns -x %~1