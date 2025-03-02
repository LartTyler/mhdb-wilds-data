using System.Text;
using RszTool.Common;

namespace RszTool
{
    public class RSZFile : BaseRszFile
    {
        public struct HeaderStruct
        {
            public uint magic;
            public uint version;
            public int objectCount;
            public int instanceCount;
            public long userdataCount;
            public long instanceOffset;
            public long dataOffset;
            public long userdataOffset;
        }

        public struct ObjectTable
        {
            public int instanceId;
        }

        public StructModel<HeaderStruct> Header { get; } = new();
        public List<StructModel<ObjectTable>> ObjectTableList { get; } = new();
        public List<InstanceInfo> InstanceInfoList { get; } = new();
        public List<IRSZUserDataInfo> RSZUserDataInfoList { get; } = new();

        /// <summary>
        /// 基于InstanceInfoList生成的实例列表
        /// 一般第一个项是NULL，手动构建时需要注意
        /// </summary>
        public List<RszInstance> InstanceList { get; } = new();
        /// <summary>
        /// 基于ObjectTableList生成的实例列表，是对外公开的实例，
        /// InstanceList中包括里面依赖的成员实例或者实例数组的项
        /// </summary>
        public List<RszInstance> ObjectList { get; private set; } = new();
        public List<RSZFile>? EmbeddedRSZFileList { get; private set; }
        // if struct changed, need rebuild

        public RSZFile(RszFileOption option, FileHandler fileHandler) : base(option, fileHandler)
        {
        }

        public const uint Magic = 0x5a5352;

        public override RSZFile GetRSZ() => this;

        protected override bool DoRead()
        {
            ObjectTableList.Clear();
            InstanceInfoList.Clear();
            RSZUserDataInfoList.Clear();
            InstanceList.Clear();
            ObjectList.Clear();
            EmbeddedRSZFileList?.Clear();

            var handler = FileHandler;
            handler.Seek(0);
            if (!Header.Read(handler)) return false;
            if (Header.Data.magic != Magic)
            {
                throw new InvalidDataException($"Not a RSZ data");
            }

            var rszParser = RszParser;

            ObjectTableList.Read(handler, Header.Data.objectCount);

            handler.Seek(Header.Data.instanceOffset);
            for (int i = 0; i < Header.Data.instanceCount; i++)
            {
                InstanceInfo instanceInfo = new();
                instanceInfo.Read(handler);
                instanceInfo.ReadClassName(rszParser);
                InstanceInfoList.Add(instanceInfo);
            }

            handler.Seek(Header.Data.userdataOffset);
            Dictionary<int, int> instanceIdToUserData = new();
            if (Option.Version <= GameVersion.re2)
            {
                if (Header.Data.userdataCount > 0)
                {
                    EmbeddedRSZFileList = new();
                    for (int i = 0; i < (int)Header.Data.userdataCount; i++)
                    {
                        RSZUserDataInfo_TDB_LE_67 rszUserDataInfo = new();
                        rszUserDataInfo.Read(handler);
                        rszUserDataInfo.ReadClassName(rszParser);
                        RSZUserDataInfoList.Add(rszUserDataInfo);
                        instanceIdToUserData[rszUserDataInfo.instanceId] = i;

                        long pos = handler.Tell();
                        RSZFile embeddedRSZFile = new(Option, handler.WithOffset(rszUserDataInfo.RSZOffset));
                        embeddedRSZFile.Read();
                        EmbeddedRSZFileList.Add(embeddedRSZFile);
                        handler.Seek(pos);
                    }
                }
            }
            else
            {
                for (int i = 0; i < (int)Header.Data.userdataCount; i++)
                {
                    RSZUserDataInfo rszUserDataInfo = new();
                    rszUserDataInfo.Read(handler);
                    RSZUserDataInfoList.Add(rszUserDataInfo);
                    instanceIdToUserData[rszUserDataInfo.instanceId] = i;
                }
            }

            // read instance data
            handler.Seek(Header.Data.dataOffset);
            for (int i = 0; i < InstanceInfoList.Count; i++)
            {
                RszClass? rszClass = RszParser.GetRSZClass(InstanceInfoList[i].typeId);
                if (rszClass == null)
                {
                    if (InstanceInfoList[i].typeId == 0)
                    {
                        rszClass = RszClass.Empty;
                    }
                    else
                    {
                        throw new Exception($"RszClass {InstanceInfoList[i].typeId} not found!");
                    }
                }
                if (!instanceIdToUserData.TryGetValue(i, out int userDataIdx))
                {
                    userDataIdx = -1;
                }
                RszInstance instance;
                if (rszClass == RszClass.Empty)
                {
                    instance = RszInstance.NULL;
                }
                else
                {
                    instance = new(rszClass, i, userDataIdx != -1 ?
                        RSZUserDataInfoList[userDataIdx] : null);
                    if (instance.RSZUserData == null)
                    {
                        instance.Read(handler);
                    }
                }
                InstanceList.Add(instance);
            }

            // 正常的数据，被依赖的实例排在前面，但为了兼容手动改过的数据，还是先读取数据，再Unflatten
            InstanceListUnflatten();

            for (int i = 0; i < ObjectTableList.Count; i++)
            {
                StructModel<ObjectTable>? item = ObjectTableList[i];
                var instance = InstanceList[item.Data.instanceId];
                instance.ObjectTableIndex = i;
                ObjectList.Add(instance);
            }
            return true;
        }

        protected override bool DoWrite()
        {
            if (StructChanged)
            {
                RebuildInstanceInfo();
            }
            FileHandler handler = FileHandler;
            ref var header = ref Header.Data;
            handler.Seek(Header.Size);
            ObjectTableList.Write(handler);

            handler.Align(16);
            header.instanceOffset = handler.Tell();
            InstanceInfoList.Write(handler);

            handler.Align(16);
            header.userdataOffset = handler.Tell();
            RSZUserDataInfoList.Write(handler);

            handler.Align(16);
            handler.StringTableFlush();

            // embedded userdata
            if (Option.Version <= GameVersion.re2 && EmbeddedRSZFileList != null)
            {
                for (int i = 0; i < RSZUserDataInfoList.Count; i++)
                {
                    handler.Align(16);
                    var item = (RSZUserDataInfo_TDB_LE_67)RSZUserDataInfoList[i];
                    item.RSZOffset = handler.Tell();
                    var embeddedRSZ = EmbeddedRSZFileList[i];
                    embeddedRSZ.WriteTo(handler.WithOffset(item.RSZOffset));
                    // rewrite
                    item.dataSize = (uint)embeddedRSZ.Size;
                    item.Rewrite(handler);
                }
            }

            handler.Align(16);
            header.dataOffset = handler.Tell();
            InstanceList.Write(handler);

            header.magic = Magic;
            header.objectCount = ObjectTableList.Count;
            header.instanceCount = InstanceList.Count;
            header.userdataCount = RSZUserDataInfoList.Count;
            Header.Write(handler, 0);
            return true;
        }

        // 下面的函数用于重新构建数据，基本读写功能都在上面

        public void ClearInstances()
        {
            InstanceList.Clear();
            InstanceList.Add(RszInstance.NULL);
        }

        public void ClearObjects()
        {
            ObjectTableList.Clear();
            ObjectList.Clear();
        }

        public string InstanceStringify(RszInstance instance)
        {
            StringBuilder sb = new();
            sb.AppendLine(instance.Stringify(InstanceList));
            return sb.ToString();
        }

        public string ObjectsStringify()
        {
            StringBuilder sb = new();
            foreach (var instance in ObjectList)
            {
                sb.AppendLine(instance.Stringify(InstanceList));
            }
            return sb.ToString();
        }

        private void FixObjectTableInstanceId(RszInstance instance)
        {
            if (instance.ObjectTableIndex >= 0 && instance.ObjectTableIndex < ObjectTableList.Count)
            {
                ObjectTableList[instance.ObjectTableIndex].Data.instanceId = instance.Index;
            }
        }

        /// <summary>
        /// 实例修正序号，如果不在实例列表中，插入到列表
        /// 设计上，只用于导入游戏对象这种不影响内部Instance结构的场景，改变结构的场景应该使用RebuildInstanceInfo
        /// </summary>
        /// <param name="instance"></param>
        public void FixInstanceIndex(RszInstance instance)
        {
            var list = InstanceList;
            if (instance.Index < 0 || instance.Index >= list.Count || list[instance.Index] != instance)
            {
                if (instance.Index >= 0)
                {
                    instance.Index = list.IndexOf(instance);
                }
                if (instance.Index < 0)
                {
                    instance.Index = list.Count;
                    list.Add(instance);
                }
                FixObjectTableInstanceId(instance);
            }
        }

        /// <summary>
        /// 实例修正序号，如果不在实例列表中，插入到列表
        /// </summary>
        /// <param name="instance"></param>
        public void FixInstanceIndexRecurse(RszInstance instance)
        {
            foreach (var item in instance.Flatten())
            {
                FixInstanceIndex(item);
            }
        }

        /// <summary>
        /// 实例修正序号，如果不在实例列表中，插入到列表
        /// </summary>
        public void FixInstanceListIndex(IEnumerable<RszInstance> list)
        {
            foreach (var instance in list)
            {
                FixInstanceIndexRecurse(instance);
            }
        }

        /// <summary>
        /// 根据srcList和依赖顺序重构InstanceList，被依赖的实例排在前面
        /// 未被依赖的实例会被移除
        /// </summary>
        public void RebuildInstanceList(IList<RszInstance> srcList)
        {
            ClearInstances();
            var list = InstanceList;
            foreach (var instance in srcList)
            {
                foreach (var item in instance.Flatten(new(){ Predicate = x => x.Index != -1 }))
                {
                    item.Index = -1;
                }
            }
            foreach (var instance in srcList)
            {
                if (instance.Index != -1) continue;
                foreach (var item in instance.Flatten())
                {
                    if (item.Index < 0 || item.Index >= list.Count || list[item.Index] != item)
                    {
                        item.Index = list.Count;
                        list.Add(item);
                        FixObjectTableInstanceId(item);
                    }
                }
            }
        }

        /// <summary>
        /// 根据ObjectList和依赖顺序重构InstanceList，被依赖的实例排在前面
        /// 未被依赖的实例会被移除
        /// </summary>
        public void RebuildInstanceList()
        {
            RebuildInstanceList(ObjectList);
        }

        /// <summary>
        /// 实例的字段值，如果是对象序号，替换成对应的实例对象
        /// </summary>
        /// <param name="instance"></param>
        public void InstanceUnflatten(RszInstance instance)
        {
            if (instance.RSZUserData != null) return;
            var fields = instance.RszClass.fields;
            for (int i = 0; i < fields.Length; i++)
            {
                var field = fields[i];
                if (field.IsReference)
                {
                    if (field.array)
                    {
                        var items = (List<object>)instance.Values[i];
                        for (int j = 0; j < items.Count; j++)
                        {
                            if (items[j] is int instanceId)
                            {
                                if (instanceId >= instance.Index && field.IsTypeInferred)
                                {
                                    // TODO: may detect error, should roll back
                                    field.type = RszFieldType.S32;
                                    throw new RszRetryOpenException($"Detected {instance.RszClass.name}.{field.name} as Object before, but seems wrong");
                                }
                                items[j] = InstanceList[instanceId];
                                InstanceUnflatten(InstanceList[instanceId]);
                            }
                        }
                    }
                    else if (instance.Values[i] is int instanceId)
                    {
                        if (instanceId >= instance.Index && field.IsTypeInferred)
                        {
                            // TODO: may detect error, should roll back
                            field.type = RszFieldType.S32;
                            throw new RszRetryOpenException($"Detected {instance.RszClass.name}.{field.name} as Object before, but seems wrong");
                        }
                        instance.Values[i] = InstanceList[instanceId];
                        InstanceUnflatten(InstanceList[instanceId]);
                    }
                }
            }
        }

        /// <summary>
        /// 所有实例的字段值，如果是对象序号，替换成对应的实例对象
        /// </summary>
        public void InstanceListUnflatten()
        {
            foreach (var instance in InstanceList)
            {
                InstanceUnflatten(instance);
            }
        }

        /// <summary>
        /// 所有实例的字段值，如果是对象序号，替换成对应的实例对象
        /// </summary>
        public void InstanceListUnflatten(IEnumerable<RszInstance> list)
        {
            foreach (var instance in list)
            {
                InstanceUnflatten(instance);
            }
        }

        /// <summary>
        /// 根据实例列表，重建InstanceInfo
        /// </summary>
        /// <param name="flatten">是否先进行flatten</param>
        public void RebuildInstanceInfo(bool rebuldInstances = true, bool rebuildObjectTable = true)
        {
            if (rebuldInstances)
            {
                RebuildInstanceList();
            }
            InstanceInfoList.Clear();
            RSZUserDataInfoList.Clear();
            foreach (var instance in InstanceList)
            {
                InstanceInfoList.Add(new InstanceInfo
                {
                    typeId = instance.RszClass.typeId,
                    CRC = instance.RszClass.crc
                });
                if (instance.RSZUserData != null)
                {
                    RSZUserDataInfoList.Add(instance.RSZUserData);
                }
            }
            if (rebuildObjectTable)
            {
                // Rebuild ObjectTableList
                ObjectTableList.Clear();
                List<RszInstance> newObjectList = new();
                foreach (var instance in ObjectList)
                {
                    if (InstanceList.Contains(instance))
                    {
                        instance.ObjectTableIndex = ObjectTableList.Count;
                        var objectTableInfo = new StructModel<ObjectTable>();
                        objectTableInfo.Data.instanceId = instance.Index;
                        ObjectTableList.Add(objectTableInfo);
                        newObjectList.Add(instance);
                    }
                }
                ObjectList = newObjectList;
            }
            StructChanged = false;
        }

        /// <summary>
        /// 添加到ObjectTable，会自动修正instance.ObjectTableIndex
        /// </summary>
        /// <param name="instance"></param>
        public void AddToObjectTable(RszInstance instance)
        {
            if (instance.ObjectTableIndex >= 0 && instance.ObjectTableIndex < ObjectTableList.Count &&
                ObjectTableList[instance.ObjectTableIndex].Data.instanceId == instance.Index)
            {
                return;
            }
            var objectTableItem = new StructModel<ObjectTable>();
            FixInstanceIndex(instance);
            objectTableItem.Data.instanceId = instance.Index;
            instance.ObjectTableIndex = ObjectTableList.Count;
            ObjectTableList.Add(objectTableItem);
            ObjectList.Add(instance);
        }

        /// <summary>
        /// 创建实例，会设置StructChanged
        /// </summary>
        /// <param name="className"></param>
        public RszInstance CreateInstance(string className)
        {
            var rszClass = RszParser.GetRSZClass(className) ??
                throw new Exception($"RszClass {className} not found!");
            RszInstance instance = RszInstance.CreateInstance(RszParser, rszClass);
            FixInstanceIndexRecurse(instance);
            StructChanged = true;
            return instance;
        }

        /// <summary>
        /// 拷贝实例，会设置StructChanged
        /// </summary>
        /// <param name="src"></param>
        /// <param name="cached"></param>
        /// <returns></returns>
        public RszInstance CloneInstance(RszInstance src, bool cached = true)
        {
            RszInstance newItem = cached ? src.CloneCached() : (RszInstance)src.Clone();
            if (cached)
            {
                RszInstance.CleanCloneCache();
            }
            // 为了可视化重新排序号，否则会显示序号是-1，但实际上保存的时候的序号和现在编号的可能不一致
            FixInstanceIndexRecurse(newItem);
            StructChanged = true;
            return newItem;
        }

        /// <summary>
        /// 数组拷贝并插入Object，会设置StructChanged
        /// </summary>
        /// <param name="array">数组</param>
        /// <param name="insertItem">待插入元素</param>
        /// <param name="insertPos">插入位置</param>
        /// <param name="isDuplicate">在原先元素的后面插入</param>
        public RszInstance ArrayInsertInstance(List<object> array, RszInstance insertItem,
                                               int insertPos = -1, bool clone = true, bool isDuplicate = false)
        {
            RszInstance newItem = clone ? CloneInstance(insertItem) : insertItem;
            if (insertPos == -1 && isDuplicate)
            {
                insertPos = array.IndexOf(insertItem);
                if (insertPos != -1) insertPos++;
            }
            if (insertPos == -1)
            {
                insertPos = array.Count;
            }
            array.Insert(insertPos, newItem);
            StructChanged = true;
            return newItem;
        }

        /// <summary>
        /// 数组插入普通项，会设置StructChanged
        /// </summary>
        /// <param name="array"></param>
        /// <param name="item"></param>
        /// <param name="insertPos"></param>
        public void ArrayInsertItem(List<object> array, object item, int insertPos = -1)
        {
            if (insertPos == -1)
            {
                insertPos = array.Count;
            }
            array.Insert(insertPos, item);
            StructChanged = true;
        }

        /// <summary>
        /// 数组移除元素，会设置StructChanged
        /// </summary>
        /// <param name="array">数组</param>
        /// <param name="item">待移除元素</param>
        public void ArrayRemoveItem(List<object> array, object item)
        {
            array.Remove(item);
            StructChanged = true;
        }

        /// <summary>
        /// 将instanceSrc的值复制到instanceDst，会设置StructChanged
        /// </summary>
        /// <param name="instanceDst"></param>
        /// <param name="instanceSrc"></param>
        /// <returns></returns>
        public bool InstanceCopyValues(RszInstance instanceDst, RszInstance instanceSrc)
        {
            bool result = instanceDst.CopyValuesFrom(instanceSrc, true);
            RszInstance.CleanCloneCache();
            FixInstanceIndexRecurse(instanceDst);
            StructChanged = true;
            return result;
        }
    }


    public class InstanceInfo : BaseModel
    {
        public uint typeId;
        public uint CRC;
        public string? ClassName { get; set; }

        protected override bool DoRead(FileHandler handler)
        {
            handler.Read(ref typeId);
            handler.Read(ref CRC);
            return true;
        }

        protected override bool DoWrite(FileHandler handler)
        {
            handler.Write(typeId);
            handler.Write(CRC);
            return true;
        }

        public void ReadClassName(RszParser parser)
        {
            ClassName = parser.GetRSZClassName(typeId);
        }
    }


    public interface IRSZUserDataInfo : IModel, ICloneable
    {
        int InstanceId { get; set; }
        uint TypeId { get; }
        string? ClassName { get; }
    }


    public class RSZUserDataInfo : BaseModel, IRSZUserDataInfo
    {
        public int instanceId;
        public uint typeId;
        public ulong pathOffset;
        public string? Path { get; set; }

        public int InstanceId { get => instanceId; set => instanceId = value; }
        public uint TypeId => typeId;
        public string? ClassName { get; set; }

        protected override bool DoRead(FileHandler handler)
        {
            handler.Read(ref instanceId);
            handler.Read(ref typeId);
            handler.Read(ref pathOffset);
            Path = handler.ReadWString((long)pathOffset);
            return true;
        }

        protected override bool DoWrite(FileHandler handler)
        {
            handler.Write(instanceId);
            handler.Write(typeId);
            handler.StringTableAdd(Path);
            handler.Write(pathOffset);
            return true;
        }

        public void ReadClassName(RszParser parser)
        {
            ClassName = parser.GetRSZClassName(typeId);
        }

        public UserdataInfo ToUserdataInfo(RszParser parser)
        {
            UserdataInfo info = new()
            {
                typeId = typeId,
                Path = Path,
                // CRC = parser.GetRSZClassCRC(typeId),
            };
            return info;
        }
    }


    public class RSZUserDataInfo_TDB_LE_67 : BaseModel, IRSZUserDataInfo
    {
        public int instanceId;
        public uint typeId;
        public uint jsonPathHash;
        public uint dataSize;
        public long RSZOffset;

        public int InstanceId { get => instanceId; set => instanceId = value; }
        public uint TypeId => typeId;
        public string? ClassName { get; set; }

        protected override bool DoRead(FileHandler handler)
        {
            handler.Read(ref instanceId);
            handler.Read(ref typeId);
            handler.Read(ref jsonPathHash);
            handler.Read(ref dataSize);
            handler.Read(ref RSZOffset);
            return true;
        }

        protected override bool DoWrite(FileHandler handler)
        {
            handler.Write(instanceId);
            handler.Write(typeId);
            handler.Write(jsonPathHash);
            handler.Write(dataSize);
            handler.Write(RSZOffset);
            return true;
        }

        public void ReadClassName(RszParser parser)
        {
            ClassName = parser.GetRSZClassName(typeId);
        }
    }
}
