@echo off
cd /d %~dp0
REMSG_Converter.exe -i %1 -m json
pause
