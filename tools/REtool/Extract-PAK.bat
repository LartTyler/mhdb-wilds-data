@setlocal enableextensions
@pushd %~dp0
.\REtool.exe -h MHWs.list -x -skipUnknowns %1
@popd
@pause