namespace RszTool
{
    public static class RszUtils
    {
        public static bool IsResourcePath(string path)
        {
            return path.Contains('/') && Path.GetExtension(path) != "";
        }

        /// <summary>
        /// 添加RSZ中用到的UserData
        /// </summary>
        /// <param name="userdataInfos"></param>
        /// <param name="rsz"></param>
        /// <param name="userDataStart"></param>
        /// <param name="length"></param>
        public static void AddUserDataFromRsz(List<UserdataInfo> userdataInfos, RSZFile rsz, int userDataStart = 0, int length = -1)
        {
            if (length == -1) length = rsz.RSZUserDataInfoList.Count - userDataStart;
            for (int i = 0; i < length; i++)
            {
                var item = rsz.RSZUserDataInfoList[i + userDataStart];
                if (item is RSZUserDataInfo info)
                {
                    userdataInfos.Add(info.ToUserdataInfo(rsz.RszParser));
                }
            }
        }

        /// <summary>
        /// 同步成RSZ中用到的UserData
        /// </summary>
        /// <param name="userdataInfos"></param>
        /// <param name="rsz"></param>
        public static void SyncUserDataFromRsz(List<UserdataInfo> userdataInfos, RSZFile rsz)
        {
            userdataInfos.Clear();
            AddUserDataFromRsz(userdataInfos, rsz);
        }

        /// <summary>
        /// 添加RSZ中用到的资源
        /// </summary>
        /// <param name="resourcesInfos"></param>
        /// <param name="rsz"></param>
        /// <param name="instanceStart"></param>
        /// <param name="length"></param>
        public static void AddResourceFromRsz(List<ResourceInfo> resourcesInfos, RSZFile rsz, int instanceStart = 0, int length = -1)
        {
            if (length == -1) length = rsz.InstanceList.Count - instanceStart;
            HashSet<string> addedPath = new();
            foreach (var item in resourcesInfos)
            {
                if (item.Path != null)
                {
                    addedPath.Add(item.Path);
                }
            }
            void CheckResouce(string path)
            {
                if (IsResourcePath(path) && !addedPath.Contains(path))
                {
                    addedPath.Add(path);
                    resourcesInfos.Add(new ResourceInfo { Path = path });
                }
            }

            for (int i = 0; i < length; i++)
            {
                var instance = rsz.InstanceList[i + instanceStart];
                if (instance.RSZUserData != null) continue;
                var fields = instance.RszClass.fields;
                // avoid reference unused resource
                if (instance.RszClass.name == "via.Folder")
                {
                    if (instance.GetFieldValue("v4") is (byte)0) continue;
                }
                for (int j = 0; j < fields.Length; j++)
                {
                    var field = fields[j];
                    if (field.IsString)
                    {
                        if (field.array)
                        {
                            foreach (var item in (List<object>)instance.Values[j])
                            {
                                CheckResouce((string)item);
                            }
                        }
                        else
                        {
                            CheckResouce((string)instance.Values[j]);
                        }
                    }
                }
            }
        }

        /// <summary>
        /// 同步成RSZ中用到的资源
        /// </summary>
        /// <param name="resourcesInfos"></param>
        /// <param name="rsz"></param>
        public static void SyncResourceFromRsz(List<ResourceInfo> resourcesInfos, RSZFile rsz)
        {
            resourcesInfos.Clear();
            AddResourceFromRsz(resourcesInfos, rsz);
        }

        public static void GetFileExtension(string path, out string extension, out string version)
        {
            version = Path.GetExtension(path);
            extension = Path.GetExtension(path[0..^version.Length]);
        }

        public static FileType GetFileType(string path)
        {
            GetFileExtension(path, out string extension, out _);
            return extension switch {
                ".user" => FileType.user,
                ".pfb" => FileType.pfb,
                ".scn" => FileType.scn,
                ".mdf2" => FileType.mdf2,
                _ => FileType.unknown,
            };
        }

        public static void CheckFileExtension(string path, string extension, string? version)
        {
            GetFileExtension(path, out string realExtension, out string realVersion);
            if (extension != realExtension)
            {
                Console.Error.WriteLine($"extension should be {extension}, got {realExtension}");
            }
            if (version != realVersion)
            {
                Console.Error.WriteLine($"extension should be {version}, got {realVersion}");
            }
        }
    }
}
