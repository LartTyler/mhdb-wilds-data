REtool by FluffyQuack (patreon.com/fluffyquack)

If you're going to unpack a PAK, remember to get the latest file list for RE2, RE3, or DMC5. Where ever you downloaded this tool from should have information on where to get up-to-date file lists.

The tool comes with various batch files to help with automating operations:
- extract-pak.bat - Drag a PAK file onto this to unpack it (remember to manually edit the batch file and update the filename of the filelist. By default it's set to RE2's file list name).
- texture-conv.bat - Drag and drop TEX/DDS textures files to convert them (note that for converting DDS to TEX, you'll need the original TEX file in the same directory as the tool updates the TEX file rather than creating it from scratch).
- texture-conv-all-dds.bat - Run this to convert all DDS files in the same directory to TEX.
- texture-conv-all-tex.bat - Run this to convert all TEX files in the same directory to DDS.
- Create-PAK-2023.bat - Drag a folder onto this to create a PAK file (meant for RE Engine games from 2023 and earlier)
- Create-PAK-2024.bat - Drag a folder onto this to create a PAK file (meant for RE Engine games from 2024 and newer)

A couple of notes:
- By default, extract-pak.bat has "-skipUnknowns" set which means it will not extract files with unknown filename. Remove this if you want those files extracted too.
- DDS -> TEX conversion works by copying over the image data. It does not update any part of the TEX header, so you should make sure the image type, size, and mipmap count are the same. This information is displayed when you convert from TEX to DDS.
- You can add "-tex" to extract-pak.bat if you want the tool to convert all found TEX files to DDS while extracting a PAK archive.
- You can find more detailed usage information here: https://www.patreon.com/posts/36746173

Thanks to Ekey who has done work figuring out the PAK format used in RE Engine games.

---------------------------------------------------------------------------------------------------------------------------------

License info:

BSD License

For Zstandard software

Copyright (c) 2016-present, Facebook, Inc. All rights reserved.

Redistribution and use in source and binary forms, with or without modification,
are permitted provided that the following conditions are met:

 * Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

 * Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

 * Neither the name Facebook nor the names of its contributors may be used to
   endorse or promote products derived from this software without specific
   prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR
ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.