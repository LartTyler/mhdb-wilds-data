@echo off
cd /d %~dp0
echo Japanese: ja
echo English: en
echo French: fr
echo Italian: it
echo German: de
echo Spanish: es
echo Russian: ru
echo Polish: pl
echo Dutch: nl
echo Portuguese: pt
echo PortugueseBr: ptbr
echo Korean: ko
echo TraditionalChinese: zhtw
echo SimplifiedChinese: zhcn
echo Finnish: fi
echo Swedish: sv
echo Danish: da
echo Norwegian: no
echo Czech: cs
echo Hungarian: hu
echo Slovak: sk
echo Arabic: ar
echo Turkish: tr
echo Bulgarian: bg
echo Greek: el
echo Romanian: ro
echo Thai: th
echo Ukrainian: ua
echo Vietnamese: vi
echo Indonesian: id
echo Fiction: cc
echo Hindi: hi
echo LatinAmericanSpanish: es419
set /p lang="Please enter one of the lang to extract: "
REMSG_Converter.exe -i %1 -m txt -l %lang%
pause
