@echo off
cd /d %~dp0
REMSG_Converter.exe -m csv %1 %2
pause
