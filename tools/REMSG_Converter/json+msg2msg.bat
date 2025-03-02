@echo off
cd /d %~dp0
REMSG_Converter.exe -m json %1 %2
pause
