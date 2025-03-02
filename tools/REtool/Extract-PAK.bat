@setlocal enableextensions
@pushd %~dp0
.\REtool.exe -h re2_pak_names_release.list -x -skipUnknowns %1
@popd
@pause