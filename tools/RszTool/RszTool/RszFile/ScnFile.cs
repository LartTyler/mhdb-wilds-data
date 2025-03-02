using RszTool.Common;
using System.Collections.ObjectModel;

namespace RszTool
{
    using GameObjectInfoModel = StructModel<ScnFile.GameObjectInfo>;
    using FolderInfoModel = StructModel<ScnFile.FolderInfo>;

    public class ScnFile : BaseRszFile
    {
        public class HeaderStruct : BaseModel {
            public uint magic;
            public int infoCount;
            public int resourceCount;
            public int folderCount;
            public int prefabCount;
            public int userdataCount;
            public long folderInfoOffset;
            public long resourceInfoOffset;
            public long prefabInfoOffset;
            public long userdataInfoOffset;
            public long dataOffset;

            private GameVersion Version { get; }

            public HeaderStruct(GameVersion version) {
                Version = version;
            }

            protected override bool DoRead(FileHandler handler)
            {
                handler.Read(ref magic);
                handler.Read(ref infoCount);
                handler.Read(ref resourceCount);
                handler.Read(ref folderCount);
                if (Version > GameVersion.re2)
                {
                    handler.Read(ref prefabCount);
                    handler.Read(ref userdataCount);
                }
                else
                {
                    handler.Read(ref userdataCount);
                    handler.Read(ref prefabCount);
                }
                handler.Read(ref folderInfoOffset);
                handler.Read(ref resourceInfoOffset);
                handler.Read(ref prefabInfoOffset);
                handler.Read(ref userdataInfoOffset);
                handler.Read(ref dataOffset);
                return true;
            }

            protected override bool DoWrite(FileHandler handler)
            {
                handler.Write(ref magic);
                handler.Write(ref infoCount);
                handler.Write(ref resourceCount);
                handler.Write(ref folderCount);
                if (Version > GameVersion.re2)
                {
                    handler.Write(ref prefabCount);
                    handler.Write(ref userdataCount);
                }
                else
                {
                    handler.Write(ref userdataCount);
                    handler.Write(ref prefabCount);
                }
                handler.Write(ref folderInfoOffset);
                handler.Write(ref resourceInfoOffset);
                handler.Write(ref prefabInfoOffset);
                handler.Write(ref userdataInfoOffset);
                handler.Write(ref dataOffset);
                return true;
            }
        }

        public struct GameObjectInfo {
            public Guid guid;
            public int objectId;
            public int parentId;
            public short componentCount;
            public short ukn;
            public int prefabId;
        }

        public struct FolderInfo {
            public int objectId;
            public int parentId;
        }

        // ResourceInfo

        public class PrefabInfo : BaseModel {
            public uint pathOffset;
            public string? Path { get; set; }
            public int parentId;

            protected override bool DoRead(FileHandler handler)
            {
                handler.Read(ref pathOffset);
                handler.Read(ref parentId);
                Path = handler.ReadWString(pathOffset);
                return true;
            }

            protected override bool DoWrite(FileHandler handler)
            {
                handler.StringTableAdd(Path);
                handler.Write(ref pathOffset);
                handler.Write(ref parentId);
                return true;
            }

            public override string ToString()
            {
                return Path ?? "";
            }
        }


        public class FolderData
        {
            private WeakReference<FolderData>? parentRef;
            public FolderInfoModel? Info { get; set; }
            public ObservableCollection<FolderData> Children { get; private set; } = new();
            public ObservableCollection<GameObjectData> GameObjects { get; private set; } = new();
            public ObservableCollection<PrefabInfo> Prefabs { get; private set; } = new();
            public RszInstance? Instance { get; set; }

            public FolderData? Parent
            {
                get => parentRef?.GetTarget();
                set => parentRef = value != null ? new(value) : null;
            }

            public int? ObjectId => Info?.Data.objectId;

            public string? Name
            {
                get => (Instance?.GetFieldValue("v0") ?? Instance?.GetFieldValue("Name")) as string;
                set
                {
                    if (value == null) return;
                    if (Instance?.SetFieldValue("v0", value) == true) return;
                    Instance?.SetFieldValue("Name", value);
                }
            }

            public override string ToString()
            {
                return Name ?? "";
            }
        }


        public class GameObjectData : IGameObjectData, ICloneable
        {
            private WeakReference<FolderData>? folderRef;
            private WeakReference<GameObjectData>? parentRef;
            public GameObjectInfoModel? Info { get; set; }
            public PrefabInfo? Prefab { get; set; }
            public RszInstance? Instance { get; set; }
            public ObservableCollection<RszInstance> Components { get; private set; } = new();
            public ObservableCollection<GameObjectData> Children { get; private set; } = new();

            public FolderData? Folder
            {
                get => folderRef?.GetTarget();
                set => folderRef = value != null ? new(value) : null;
            }

            public GameObjectData? Parent
            {
                get => parentRef?.GetTarget();
                set => parentRef = value != null ? new(value) : null;
            }

            public string? Name
            {
                get => (Instance?.GetFieldValue("v0") ?? Instance?.GetFieldValue("Name")) as string;
                set
                {
                    if (value == null) return;
                    if (Instance?.SetFieldValue("v0", value) == true) return;
                    Instance?.SetFieldValue("Name", value);
                }
            }

            public int? ObjectId => Info?.Data.objectId;

            public Guid Guid
            {
                get
                {
                    return Info?.Data.guid ?? Guid.Empty;
                }
                set
                {
                    if (Info != null) Info.Data.guid = value;
                }
            }

            // Guid of GameObject Cloned from
            public Guid OriginalGuid { get; set; }

            public object Clone()
            {
                GameObjectData gameObject = new()
                {
                    Info = Info != null ? new() { Data = Info.Data } : null,
                    Components = new(Components.Select(item => item.CloneCached())),
                    Instance = Instance?.CloneCached(),
                    Prefab = Prefab != null ? (PrefabInfo)Prefab.Clone() : null,
                    OriginalGuid = Guid,
                    Guid = Guid.NewGuid()
                };
                foreach (var child in Children)
                {
                    var newChild = (GameObjectData)child.Clone();
                    newChild.Parent = gameObject;
                    gameObject.Children.Add(newChild);
                }
                return gameObject;
            }

            public override string ToString()
            {
                return Name ?? "";
            }

            public static GameObjectData FromPfbGameObject(PfbFile.GameObjectData pfbGameObject)
            {
                GameObjectData gameObject = new()
                {
                    Info = new()
                    {
                        Data = new GameObjectInfo
                        {
                            // objectId 和 parentId 应该重新生成
                            guid = Guid.NewGuid(),
                            objectId = -1,
                            parentId = -1,
                            componentCount = (short)pfbGameObject.Components.Count,
                            prefabId = -1,
                        }
                    },
                    Components = new(pfbGameObject.Components.Select(item => item.CloneCached())),
                    Instance = pfbGameObject.Instance?.CloneCached()
                };
                foreach (var child in pfbGameObject.Children)
                {
                    var newChild = FromPfbGameObject(child);
                    newChild.Parent = gameObject;
                    gameObject.Children.Add(newChild);
                }
                RszInstance.CleanCloneCache();
                return gameObject;
            }

            public IEnumerable<IGameObjectData> GetChildren() => Children;
        }

        public HeaderStruct Header { get; }
        public List<GameObjectInfoModel> GameObjectInfoList { get; } = new();
        public List<FolderInfoModel> FolderInfoList { get; } = new();
        public List<ResourceInfo> ResourceInfoList { get; } = new();
        public List<PrefabInfo> PrefabInfoList { get; } = new();
        public List<UserdataInfo> UserdataInfoList { get; } = new();
        public RSZFile? RSZ { get; private set; }

        public ObservableCollection<FolderData>? FolderDatas { get; set; }
        public ObservableCollection<GameObjectData>? GameObjectDatas { get; set; }
        public bool ResourceChanged { get; set; } = false;

        public ScnFile(RszFileOption option, FileHandler fileHandler) : base(option, fileHandler)
        {
            Header = new(option.Version);
            if (fileHandler.FilePath != null)
            {
                RszUtils.CheckFileExtension(fileHandler.FilePath, Extension2, GetVersionExt());
            }
        }

        public const uint Magic = 0x4e4353;
        public const string Extension2 = ".scn";

        public string? GetVersionExt()
        {
            return Option.GameName switch
            {
                GameName.re2 => ".19",
                GameName.re2rt => ".20",
                GameName.re3 => ".20",
                GameName.re4 => ".20",
                GameName.re8 => ".20",
                GameName.re7 => ".18",
                GameName.re7rt => ".20",
                GameName.dmc5 =>".19",
                GameName.mhrise => ".20",
                GameName.sf6 => ".20",
                _ => null
            };
        }

        public override RSZFile? GetRSZ() => RSZ;

        protected override bool DoRead()
        {
            GameObjectInfoList.Clear();
            FolderInfoList.Clear();
            ResourceInfoList.Clear();
            PrefabInfoList.Clear();
            UserdataInfoList.Clear();
            FolderDatas?.Clear();
            GameObjectDatas?.Clear();

            var handler = FileHandler;
            var header = Header;
            if (!header.Read(handler)) return false;
            if (header.magic != Magic)
            {
                throw new InvalidDataException($"{handler.FilePath} Not a SCN file");
            }

            GameObjectInfoList.Read(handler, header.infoCount);

            handler.Seek(header.folderInfoOffset);
            FolderInfoList.Read(handler, header.folderCount);

            handler.Seek(header.resourceInfoOffset);
            ResourceInfoList.Read(handler, header.resourceCount);

            handler.Seek(header.prefabInfoOffset);
            PrefabInfoList.Read(handler, header.prefabCount);

            handler.Seek(header.userdataInfoOffset);
            UserdataInfoList.Read(handler, header.userdataCount);

            RSZ = ReadRsz(header.dataOffset);

            return true;
        }

        protected override bool DoWrite()
        {
            if (StructChanged)
            {
                RebuildInfoTable();
            }
            else if (ResourceChanged)
            {
                RszUtils.SyncResourceFromRsz(ResourceInfoList, RSZ!);
                ResourceChanged = false;
            }

            FileHandler handler = FileHandler;
            handler.Clear();
            var header = Header;
            handler.Seek(header.Size);
            GameObjectInfoList.Write(handler);

            if (FolderInfoList.Count > 0)
            {
                handler.Align(16);
                header.folderInfoOffset = handler.Tell();
                FolderInfoList.Write(handler);
            }

            handler.Align(16);
            header.resourceInfoOffset = handler.Tell();
            ResourceInfoList.Write(handler);

            if (PrefabInfoList.Count > 0)
            {
                handler.Align(16);
                header.prefabInfoOffset = handler.Tell();
                PrefabInfoList.Write(handler);
            }

            handler.Align(16);
            header.userdataInfoOffset = handler.Tell();
            UserdataInfoList.Write(handler);

            handler.StringTableFlush();

            handler.Align(16);
            header.dataOffset = handler.Tell();
            WriteRsz(RSZ!, header.dataOffset);

            header.magic = Magic;
            header.infoCount = GameObjectInfoList.Count;
            header.resourceCount = ResourceInfoList.Count;
            header.folderCount = FolderInfoList.Count;
            header.prefabCount = PrefabInfoList.Count;
            header.userdataCount = UserdataInfoList.Count;
            header.Write(handler, 0);

            return true;
        }

        /// <summary>
        /// 解析关联的关系，形成树状结构
        /// </summary>
        public void SetupGameObjects()
        {
            Dictionary<int, FolderData> folderIdxMap = new();
            FolderDatas ??= new();
            FolderDatas.Clear();
            if (FolderInfoList.Count > 0)
            {
                foreach (var info in FolderInfoList)
                {
                    FolderData folderData = new()
                    {
                        Info = info,
                        Instance = RSZ!.ObjectList[info.Data.objectId],
                    };
                    if (info.Data.parentId == -1)
                    {
                        FolderDatas.Add(folderData);
                    }
                    folderIdxMap[info.Data.objectId] = folderData;
                }
            }

            Dictionary<int, GameObjectData> gameObjectMap = new();
            GameObjectDatas ??= new();
            GameObjectDatas.Clear();
            foreach (var info in GameObjectInfoList)
            {
                GameObjectData gameObjectData = new()
                {
                    Info = info,
                    Instance = RSZ!.ObjectList[info.Data.objectId],
                };
                for (int i = 0; i < info.Data.componentCount; i++)
                {
                    gameObjectData.Components.Add(RSZ!.ObjectList[info.Data.objectId + 1 + i]);
                }
                if (info.Data.prefabId >= 0 && info.Data.prefabId < PrefabInfoList.Count)
                {
                    gameObjectData.Prefab = PrefabInfoList[info.Data.prefabId];
                }
                gameObjectMap[info.Data.objectId] = gameObjectData;
                if (info.Data.parentId == -1)
                {
                    GameObjectDatas.Add(gameObjectData);
                }
            }

            // add children and set parent
            foreach (var info in GameObjectInfoList)
            {
                var gameObject = gameObjectMap[info.Data.objectId];
                if (gameObjectMap.TryGetValue(info.Data.parentId, out var parent))
                {
                    parent.Children.Add(gameObject);
                    gameObject.Parent = parent;
                }
                if (folderIdxMap.TryGetValue(info.Data.parentId, out var folder))
                {
                    folder.GameObjects.Add(gameObject);
                    gameObject.Folder = folder;
                }
            }

            foreach (var info in FolderInfoList)
            {
                if (folderIdxMap.TryGetValue(info.Data.parentId, out var folder))
                {
                    var childFolder = folderIdxMap[info.Data.objectId];
                    folder.Children.Add(childFolder);
                    childFolder.Parent = folder;
                }
            }

            foreach (var info in PrefabInfoList)
            {
                if (info.Path != null && folderIdxMap.TryGetValue(info.parentId, out var folder))
                {
                    folder.Prefabs.Add(info);
                }
            }
        }

        /// <summary>
        /// 收集GameObject以及子物体的实例和组件实例
        /// </summary>
        /// <param name="gameObject"></param>
        /// <param name="rszInstances"></param>
        public static void CollectGameObjectInstances(GameObjectData gameObject, List<RszInstance> rszInstances)
        {
            rszInstances.Add(gameObject.Instance!);
            foreach (var item in gameObject.Components)
            {
                rszInstances.Add(item);
            }
            foreach (var child in gameObject.Children)
            {
                CollectGameObjectInstances(child, rszInstances);
            }
        }

        /// <summary>
        /// 迭代GameObject以及子物体的实例和组件实例
        /// </summary>
        /// <param name="gameObject"></param>
        /// <returns></returns>
        public static IEnumerable<RszInstance> IterGameObjectInstances(GameObjectData gameObject)
        {
            yield return gameObject.Instance!;
            foreach (var item in gameObject.Components)
            {
                yield return item;
            }
            foreach (var child in gameObject.Children)
            {
                foreach (var item in IterGameObjectInstances(child))
                {
                    yield return item;
                }
            }
        }

        /// <summary>
        /// 收集Folder以及子Folder的实例和组件实例
        /// </summary>
        /// <param name="gameObject"></param>
        /// <param name="rszInstances"></param>
        public static void CollectFolderGameObjectInstances(FolderData folder, List<RszInstance> rszInstances)
        {
            rszInstances.Add(folder.Instance!);
            foreach (var gameObject in folder.GameObjects)
            {
                CollectGameObjectInstances(gameObject, rszInstances);
            }
            foreach (var child in folder.Children)
            {
                CollectFolderGameObjectInstances(child, rszInstances);
            }
        }

        /// <summary>
        /// 迭代Folder以及子Folder的实例和组件实例
        /// </summary>
        /// <param name="folder"></param>
        /// <returns></returns>
        public static IEnumerable<RszInstance> IterFolderGameObjectInstances(FolderData folder)
        {
            yield return folder.Instance!;
            foreach (var gameObject in folder.GameObjects)
            {
                foreach (var item in IterGameObjectInstances(gameObject))
                {
                    yield return item;
                }
            }
            foreach (var child in folder.Children)
            {
                foreach (var item in IterFolderGameObjectInstances(child))
                {
                    yield return item;
                }
            }
        }

        /// <summary>
        /// 根据GameObjectDatas和FolderDatas重建其他表
        /// </summary>
        public void RebuildInfoTable()
        {
            if (RSZ == null)
            {
                throw new InvalidOperationException("RSZ is null");
            }

            // 重新生成实例表
            List<RszInstance> rszInstances = new() { RszInstance.NULL };
            if (FolderDatas != null)
            {
                foreach (var folder in FolderDatas)
                {
                    CollectFolderGameObjectInstances(folder, rszInstances);
                }
            }
            if (GameObjectDatas != null)
            {
                foreach (var gameObjectData in GameObjectDatas)
                {
                    CollectGameObjectInstances(gameObjectData, rszInstances);
                }
            }

            RSZ.RebuildInstanceList(rszInstances);
            RSZ.RebuildInstanceInfo(false, false);
            foreach (var instance in rszInstances)
            {
                instance.ObjectTableIndex = -1;
            }
            RSZ.ClearObjects();

            // 重新构建
            GameObjectInfoList.Clear();
            FolderInfoList.Clear();
            PrefabInfoList.Clear();
            if (FolderDatas != null)
            {
                foreach (var folder in FolderDatas)
                {
                    AddFolderInfoRecursion(folder);
                }
            }
            if (GameObjectDatas != null)
            {
                foreach (var gameObjectData in GameObjectDatas)
                {
                    AddGameObjectInfoRecursion(gameObjectData);
                }
            }
            RszUtils.SyncUserDataFromRsz(UserdataInfoList, RSZ);
            RszUtils.SyncResourceFromRsz(ResourceInfoList, RSZ);
            StructChanged = false;
            ResourceChanged = false;
        }

        /// <summary>
        /// 添加GameObject到Info表，并把Instance添加到RSZ的ObjectTable
        /// </summary>
        /// <param name="gameObject"></param>
        private void AddGameObjectInfo(GameObjectData gameObject)
        {
            var instance = gameObject.Instance!;
            if (instance.ObjectTableIndex != -1) return;
            // int prefabId = gameObject.Prefab == null ? -1 :
            //     PrefabInfoList.GetIndexOrAdd(gameObject.Prefab);
            int prefabId = -1;
            if (gameObject.Prefab != null)
            {
                prefabId = PrefabInfoList.FindIndex(item => item.Path == gameObject.Prefab.Path);
                if (prefabId == -1)
                {
                    prefabId = PrefabInfoList.Count;
                    if (gameObject.Folder != null)
                    {
                        gameObject.Prefab.parentId = gameObject.Folder.ObjectId ?? -1;
                        gameObject.Folder.Prefabs.Add(gameObject.Prefab);
                    }
                    else
                    {
                        gameObject.Prefab.parentId = -1;
                    }
                    PrefabInfoList.Add(gameObject.Prefab);
                }
                else
                {
                    gameObject.Prefab = PrefabInfoList[prefabId];
                }
            }
            ref var infoData = ref gameObject.Info!.Data;
            // AddToObjectTable会修正ObjectTableIndex
            RSZ!.AddToObjectTable(instance);
            infoData.objectId = instance.ObjectTableIndex;
            infoData.componentCount = (short)gameObject.Components.Count;
            infoData.parentId = gameObject.Parent?.ObjectId ?? gameObject.Folder?.ObjectId ?? -1;
            infoData.prefabId = prefabId;

            GameObjectInfoList.Add(gameObject.Info);
            foreach (var item in gameObject.Components)
            {
                RSZ.AddToObjectTable(item);
            }
        }

        /// <summary>
        /// 递归添加GameObject到Info表，并把Instance添加到RSZ的ObjectTable
        /// </summary>
        /// <param name="gameObject"></param>
        private void AddGameObjectInfoRecursion(GameObjectData gameObject)
        {
            AddGameObjectInfo(gameObject);
            foreach (var child in gameObject.Children)
            {
                AddGameObjectInfoRecursion(child);
            }
        }

        /// <summary>
        /// 递归添加文件夹和其中的GameObject到Info表，并把Instance添加到RSZ的ObjectTable
        /// </summary>
        /// <param name="gameObject"></param>
        private void AddFolderInfoRecursion(FolderData folder)
        {
            RSZ!.AddToObjectTable(folder.Instance!);
            ref var infoData = ref folder.Info!.Data;
            infoData.objectId = folder.Instance!.ObjectTableIndex;
            infoData.parentId = folder.Parent?.ObjectId ?? -1;
            FolderInfoList.Add(folder.Info!);
            foreach (var gameObject in folder.GameObjects)
            {
                AddGameObjectInfoRecursion(gameObject);
            }
            foreach (var child in folder.Children)
            {
                AddFolderInfoRecursion(child);
            }
            foreach (var prefab in folder.Prefabs)
            {
                prefab.parentId = infoData.objectId;
                // PrefabInfoList.GetIndexOrAdd(prefab);
            }
        }

        /// <summary>
        /// 根据名字查找游戏对象
        /// </summary>
        /// <param name="name">对象名称</param>
        /// <param name="parent">父对象</param>
        /// <param name="recursive">查找游戏对象的子对象</param>
        /// <returns></returns>
        public GameObjectData? FindGameObject(string name, GameObjectData? parent = null, bool recursive = false)
        {
            var children = parent?.Children ?? GameObjectDatas;
            if (children == null)
            {
                Console.Error.WriteLine("GameObjectDatas and parent is null");
                return null;
            }
            foreach (var child in children)
            {
                if (child.Name == name) return child;
            }
            if (recursive)
            {
                foreach (var child in children)
                {
                    var result = FindGameObject(name, child);
                    if (result != null) return result;
                }
            }
            Console.Error.WriteLine($"GameObject {name} not found");
            return null;
        }

        /// <summary>
        /// 在文件夹中根据名字查找游戏对象
        /// </summary>
        /// <param name="name">对象名称</param>
        /// <param name="parent">父文件夹，如果为空则从根文件夹开始</param>
        /// <param name="recursive">递归在子文件夹中查找</param>
        /// <param name="gameObjectRecursive">查找游戏对象的子对象</param>
        /// <returns></returns>
        public GameObjectData? FindGameObjectInFolder(
            string name, FolderData? parent = null, bool recursive = false, bool gameObjectRecursive = false)
        {
            var folders = parent?.Children ?? FolderDatas;
            if (folders == null)
            {
                Console.Error.WriteLine("FolderDatas and parent is null");
                return null;
            }
            foreach (var folder in folders!)
            {
                foreach (var gameObject in folder.GameObjects)
                {
                    var result = FindGameObject(name, gameObject, gameObjectRecursive);
                    if (result != null) return result;
                }
            }
            if (recursive)
            {
                foreach (var folder in folders)
                {
                    var result = FindGameObjectInFolder(name, folder, recursive, gameObjectRecursive);
                    if (result != null) return result;
                }
            }
            Console.Error.WriteLine($"GameObject {name} not found");
            return null;
        }

        /// <summary>
        /// 在文件夹中迭代游戏对象
        /// </summary>
        /// <param name="parent"></param>
        /// <param name="includeChildren">包括子对象</param>
        /// <returns></returns>
        public IEnumerable<GameObjectData> IterGameObjectsInFolder(FolderData? parent = null, bool includeChildren = false)
        {
            var folders = parent?.Children ?? FolderDatas;
            if (folders == null)
            {
                Console.Error.WriteLine("FolderDatas and parent is null");
                yield break;
            }
            foreach (var folder in folders)
            {
                foreach (var gameObject in folder.GameObjects)
                {
                    yield return gameObject;
                    if (includeChildren)
                    {
                        foreach (var child in IterGameObjects(gameObject, includeChildren))
                        {
                            yield return child;
                        }
                    }
                }
            }
            foreach (var folder in folders)
            {
                foreach (var item in IterGameObjectsInFolder(folder, includeChildren))
                {
                    yield return item;
                }
            }
        }

        public IEnumerable<GameObjectData> IterGameObjects(GameObjectData? parent = null, bool includeChildren = false)
        {
            var items = parent?.Children ?? GameObjectDatas;
            if (items == null)
            {
                yield break;
            }
            foreach (var item in items)
            {
                yield return item;
                if (includeChildren)
                {
                    foreach (var child in IterGameObjects(item, includeChildren))
                    {
                        yield return child;
                    }
                }
            }
        }

        public IEnumerable<GameObjectData> IterAllGameObjects(bool includeChildren = false)
        {
            return IterGameObjectsInFolder(includeChildren: includeChildren).Concat(
                IterGameObjects(includeChildren: includeChildren));
        }

        public void RemoveFolder(FolderData folder)
        {
            if (folder.Parent != null)
            {
                folder.Parent.Children.Remove(folder);
                folder.Parent = null;
            }
            else
            {
                FolderDatas?.Remove(folder);
            }
            StructChanged = true;
        }

        public void RemoveGameObject(GameObjectData gameObject)
        {
            if (gameObject.Parent != null)
            {
                gameObject.Parent.Children.Remove(gameObject);
                gameObject.Parent = null;
            }
            else if (gameObject.Folder != null)
            {
                gameObject.Folder.GameObjects.Remove(gameObject);
                gameObject.Folder = null;
            }
            else
            {
                GameObjectDatas?.Remove(gameObject);
            }
            StructChanged = true;
        }

        /// <summary>
        /// 提取某个游戏对象，生成Pfb
        /// </summary>
        /// <param name="name"></param>
        public bool ExtractGameObjectToPfb(string name, PfbFile pfbFile)
        {
            GameObjectData? gameObject = FindGameObject(name);
            if (gameObject == null) return false;

            List<RszInstance> rszInstances = new();
            CollectGameObjectInstances(gameObject, rszInstances);

            // 这里有拷贝instance
            pfbFile.PfbFromScnGameObject(gameObject);
            pfbFile.RSZ!.Header.Data.version = RSZ!.Header.Data.version;
            return true;
        }

        /// <summary>
        /// Fix GameObjectRef after clone
        /// </summary>
        /// <param name="rootGameObject"></param>
        private void FixGameObjectRef(GameObjectData rootGameObject)
        {
            Dictionary<Guid, List<GameObjectRefData>>? gameObjectRefDatas = null;
            // record GameObjectRef
            foreach (var item in IterGameObjectInstances(rootGameObject))
            {
                if (item.RszClass.name == "via.GameObject") continue;
                foreach (var instance in item.Flatten())
                {
                    if (instance.RSZUserData != null) continue;
                    var fields = instance.Fields;
                    for (int i = 0; i < fields.Length; i++)
                    {
                        RszField field = fields[i];
                        if (field.type == RszFieldType.GameObjectRef)
                        {
                            gameObjectRefDatas ??= new();
                            if (field.array)
                            {
                                var items = (List<object>)instance.Values[i];
                                for (int j = 0; j < items.Count; j++)
                                {
                                    Guid guid = (Guid)items[j];
                                    if (guid != Guid.Empty)
                                    {
                                        if (!gameObjectRefDatas.TryGetValue(guid, out var refDatas))
                                        {
                                            gameObjectRefDatas[guid] = refDatas = new();
                                        }
                                        refDatas.Add(new(items, j));
                                    }
                                }
                            }
                            else
                            {
                                Guid guid = (Guid)instance.Values[i];
                                if (guid != Guid.Empty)
                                {
                                    if (!gameObjectRefDatas.TryGetValue(guid, out var refDatas))
                                    {
                                        gameObjectRefDatas[guid] = refDatas = new();
                                    }
                                    refDatas.Add(new(instance.Values, i));
                                }
                            }
                        }
                    }
                }
            }
            foreach (var item in IterGameObjects(rootGameObject, true))
            {
                // fix GameObjectRef
                if (gameObjectRefDatas != null && item.OriginalGuid != Guid.Empty &&
                    gameObjectRefDatas.TryGetValue(item.OriginalGuid, out var refDatas))
                {
                    foreach (var refData in refDatas)
                    {
                        refData.Values[refData.ValueIndex] = item.Guid;
                    }
                }
            }
        }

        /// <summary>
        /// 导入外部的游戏对象
        /// 文件夹和父对象只能指定一个
        /// </summary>
        /// <param name="gameObject"></param>
        /// <param name="folder">文件夹</param>
        /// <param name="parent">父对象</param>
        /// <param name="isDuplicate">在原对象的位置后面添加</param>
        /// <returns>新游戏对象</returns>
        public GameObjectData ImportGameObject(GameObjectData gameObject, FolderData? folder = null,
                                     GameObjectData? parent = null, bool isDuplicate = false)
        {
            GameObjectData newGameObject = (GameObjectData)gameObject.Clone();
            FixGameObjectRef(newGameObject);

            newGameObject.Folder = null;
            newGameObject.Parent = null;
            ObservableCollection<GameObjectData> collection;
            if (folder != null)
            {
                newGameObject.Folder = folder;
                collection = folder.GameObjects;
            }
            else if (parent != null)
            {
                newGameObject.Parent = parent;
                collection = parent.Children;
            }
            else
            {
                GameObjectDatas ??= [];
                collection = GameObjectDatas;
            }

            // 为了可视化重新排序号，否则会显示序号是-1，但实际上保存的时候的序号和现在编号的可能不一致
            // 所以要考虑这步操作是否有必要
            RSZ!.FixInstanceListIndex(IterGameObjectInstances(newGameObject));

            if (isDuplicate)
            {
                int index = collection.IndexOf(gameObject);
                index = index == -1 ? collection.Count : index + 1;
                collection.Insert(index, newGameObject);
            }
            else
            {
                collection.Add(newGameObject);
            }
            StructChanged = true;
            RszInstance.CleanCloneCache();
            return newGameObject;
        }

        /// <summary>
        /// 导入外部的游戏对象
        /// 批量添加建议直接GameObjectDatas添加，最后再RebuildInfoTable
        /// </summary>
        /// <param name="gameObject"></param>
        public void ImportGameObjects(
            IEnumerable<GameObjectData> gameObjects,
            FolderData? folder = null, GameObjectData? parent = null)
        {
            foreach (var gameObject in gameObjects)
            {
                ImportGameObject(gameObject, folder, parent);
            }
        }

        /// <summary>
        /// 复制游戏对象
        /// </summary>
        /// <param name="gameObject"></param>
        /// <returns>新游戏对象</returns>
        public GameObjectData DuplicateGameObject(GameObjectData gameObject)
        {
            return ImportGameObject(gameObject, gameObject.Folder, gameObject.Parent, true);
        }

        public void AddFolder(string name, FolderData? parent = null)
        {
            var folder = new FolderData
            {
                Instance = RSZ!.CreateInstance("via.Folder"),
                Info = new FolderInfoModel(),
                Name = name
            };
            if (parent != null)
            {
                parent.Children.Add(folder);
                folder.Parent = parent;
            }
            else
            {
                FolderDatas?.Add(folder);
            }
            AddFolderInfoRecursion(folder);
            StructChanged = true;
        }

        /// <summary>
        /// 添加组件
        /// </summary>
        /// <param name="gameObject"></param>
        /// <param name="className"></param>
        public void AddComponent(IGameObjectData gameObject, string className)
        {
            var component = RSZ!.CreateInstance(className);
            gameObject.Components.Add(component);
            StructChanged = true;
        }

        /// <summary>
        /// 添加组件
        /// </summary>
        /// <param name="gameObject"></param>
        /// <param name="component"></param>
        public void AddComponent(IGameObjectData gameObject, RszInstance component)
        {
            gameObject.Components.Add(component);
            StructChanged = true;
        }

        /// <summary>
        /// 移除组件
        /// </summary>
        /// <param name="gameObject"></param>
        /// <param name="component"></param>
        public bool RemoveComponent(IGameObjectData gameObject, RszInstance component)
        {
            bool result = gameObject.Components.Remove(component);
            if (result) StructChanged = true;
            return result;
        }
    }


    internal record GameObjectRefData(IList<object> Values, int ValueIndex)
    {
    }
}
