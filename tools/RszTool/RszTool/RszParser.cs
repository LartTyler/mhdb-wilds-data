using System.Globalization;
using System.Text.Json;
using System.Text.Json.Serialization;
using RszTool.Common;

namespace RszTool
{
    /// <summary>
    /// Rsz json parser
    /// </summary>
    public class RszParser
    {
        private static readonly Dictionary<string, RszParser> instanceDict = new();

        public static RszParser GetInstance(string jsonPath)
        {
            if (!instanceDict.TryGetValue(jsonPath, out var rszParser))
            {
                rszParser = new RszParser(jsonPath);
                instanceDict[jsonPath] = rszParser;
            }
            return rszParser;
        }

        private readonly Dictionary<uint, RszClass> classDict;
        private readonly Dictionary<string, RszClass> classNameDict;

        public Dictionary<uint, RszClass> ClassDict => classDict;

        public RszParser(string jsonPath)
        {
            classDict = new();
            classNameDict = new();
            using FileStream fileStream = File.OpenRead(jsonPath);
            var dict = JsonSerializer.Deserialize<Dictionary<string, RszClass>>(fileStream);
            if (dict != null)
            {
                foreach (var item in dict)
                {
                    var value = item.Value;
                    value.typeId = uint.Parse(item.Key, NumberStyles.HexNumber);
                    classDict[value.typeId] = value;
                    classNameDict[value.name] = value;

                    foreach (var field in value.fields)
                    {
                        if (field.type == RszFieldType.Data)
                        {
                            field.GuessDataType();
                        }
                    }
                }

                ReadPatch(jsonPath);
            }
        }

        /// <summary>
        /// Read patch json
        /// </summary>
        private void ReadPatch(string originalJsonPath)
        {
            string patchJsonPath = Path.Combine(
                "Data", "RszPatch",
                Path.GetFileNameWithoutExtension(originalJsonPath) + "_patch.json");
            if (!File.Exists(patchJsonPath)) return;
            using FileStream fileStream = File.OpenRead(patchJsonPath);
            var dict = JsonSerializer.Deserialize<Dictionary<string, RszClassPatch>>(fileStream);
            if (dict != null)
            {
                foreach (var item in dict)
                {
                    var classPatch = item.Value;
                    classPatch.Name = item.Key;
                    if (classNameDict.TryGetValue(classPatch.Name, out var rszClass))
                    {
                        if (!string.IsNullOrEmpty(classPatch.ReplaceName))
                        {
                            rszClass.name = classPatch.ReplaceName!;
                        }
                        if (classPatch.FieldPatches != null)
                        {
                            foreach (var fieldPatch in classPatch.FieldPatches)
                            {
                                if (rszClass.GetField(fieldPatch.Name!) is RszField rszField)
                                {
                                    if (!string.IsNullOrEmpty(fieldPatch.ReplaceName))
                                    {
                                        rszField.name = fieldPatch.ReplaceName!;
                                    }
                                    if (!string.IsNullOrEmpty(fieldPatch.OriginalType))
                                    {
                                        rszField.original_type = fieldPatch.OriginalType!;
                                    }
                                    if (fieldPatch.Type != RszFieldType.ukn_error)
                                    {
                                        if (rszField.type != RszFieldType.Data)
                                        {
                                            Console.WriteLine(
                                                $"Warning: {classPatch.Name}.{fieldPatch.Name} change type " +
                                                $"from {rszField.type} to {fieldPatch.Type}");
                                        }
                                        rszField.type = fieldPatch.Type;
                                        rszField.IsTypeInferred = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        public static RszFieldType GetFieldTypeInternal(string typeName)
        {
#if NET5_0_OR_GREATER
            return Enum.Parse<RszFieldType>(typeName, true);
#else
            return (RszFieldType)Enum.Parse(typeof(RszFieldType), typeName, true);
#endif
        }

        public string GetRSZClassName(uint classHash)
        {
            return classDict.TryGetValue(classHash, out var rszClass) ? rszClass.name : "Unknown Class!";
        }

        public RszClass? GetRSZClass(uint classHash)
        {
            return classDict.TryGetValue(classHash, out var rszClass) ? rszClass : null;
        }

        public RszClass? GetRSZClass(string className)
        {
            return classNameDict.TryGetValue(className, out var rszClass) ? rszClass : null;
        }

        public uint GetRSZClassCRC(uint classHash)
        {
            return classDict.TryGetValue(classHash, out var rszClass) ? rszClass.crc : 0;
        }

        public int GetFieldCount(uint classHash)
        {
            return classDict.TryGetValue(classHash, out var rszClass) ? rszClass.fields.Length : -1;
        }

        public RszField? GetField(uint classHash, uint fieldIndex)
        {
            if (classDict.TryGetValue(classHash, out var rszClass))
            {
                if (fieldIndex >= 0 && fieldIndex < rszClass.fields.Length)
                {
                    return rszClass.fields[fieldIndex];
                }
            }
            return null;
        }

        public int GetFieldAlignment(uint classHash, uint fieldIndex)
        {
            return GetField(classHash, fieldIndex)?.align ?? -1;
        }

        public bool GetFieldArrayState(uint classHash, uint fieldIndex)
        {
            return GetField(classHash, fieldIndex)?.array ?? false;
        }

        public string GetFieldName(uint classHash, uint fieldIndex)
        {
            return GetField(classHash, fieldIndex)?.name ?? "not found";
        }

        public string GetFieldTypeName(uint classHash, uint fieldIndex)
        {
            return GetField(classHash, fieldIndex)?.type.ToString() ?? "not found";
        }

        public string GetFieldOrgTypeName(uint classHash, uint fieldIndex)
        {
            return GetField(classHash, fieldIndex)?.original_type ?? "not found";
        }

        public int GetFieldSize(uint classHash, uint fieldIndex)
        {
            return GetField(classHash, fieldIndex)?.size ?? -1;
        }

        public RszFieldType GetFieldType(uint classHash, uint fieldIndex)
        {
            if (classDict.TryGetValue(classHash, out var rszClass))
            {
                if (fieldIndex >= 0 && fieldIndex < rszClass.fields.Length)
                {
                    return rszClass.fields[fieldIndex].type;
                }
                return RszFieldType.out_of_range;
            }
            return RszFieldType.class_not_found;
        }

        public bool IsClassNative(uint classHash)
        {
            return classDict.TryGetValue(classHash, out var rszClass) && rszClass.native;
        }

        public bool IsFieldNative(uint classHash, uint fieldIndex)
        {
            return GetField(classHash, fieldIndex)?.native ?? false;
        }
    }

    public class RszClass
    {
        [JsonIgnore]
        public uint typeId { get; set; }
        [JsonConverter(typeof(HexUIntJsonConverter))]
        public uint crc { get; set; }
        public string name { get; set; } = "";
        public bool native { get; set; }
        public RszField[] fields { get; set; } = [];

        public static readonly RszClass Empty = new();

        public int IndexOfField(string name)
        {
            for (int i = 0; i < fields.Length; i++)
            {
                if (fields[i].name == name)
                {
                    return i;
                }
            }
            return -1;
        }

        public RszField? GetField(string fieldName)
        {
            int index = IndexOfField(fieldName);
            if (index != -1) return fields[index];
            return null;
        }
    }

    public class RszField
    {
        public string name { get; set; } = "";
        public int align { get; set; }
        public int size { get; set; }
        public bool array { get; set; }
        public bool native { get; set; }
        [JsonConverter(typeof(EnumJsonConverter<RszFieldType>))]
        public RszFieldType type { get; set; }
        public string original_type { get; set; } = "";
        /// <summary>
        /// rsz json中存的是Data，根据实际情况推测是其他类型
        /// </summary>
        [JsonIgnore]
        public bool IsTypeInferred { get; set; }
        private string? displayType = null;
        /// <summary>
        /// 显示类型
        /// </summary>
        public string DisplayType
        {
            get
            {
                if (displayType == null)
                {
                    if (string.IsNullOrEmpty(original_type))
                    {
                        displayType = array ? $"{type}[]" : type.ToString();
                        if (IsTypeInferred) displayType += "?";
                    }
                    else
                    {
                        displayType = original_type;
                    }
                }
                return displayType;
            }
        }

        public void GuessDataType()
        {
            if (type != RszFieldType.Data) return;
            type = size switch
            {
                64 => RszFieldType.Mat4,
                16 => align == 8 ? RszFieldType.Guid : RszFieldType.Vec4,
                8 => align == 8 ? RszFieldType.U64 : RszFieldType.Vec2,
                1 => RszFieldType.U8,
                _ => type
            };
            // IsTypeInferred = type != RszFieldType.Data;
        }

        public bool IsReference => type == RszFieldType.Object || type == RszFieldType.UserData;
        public bool IsString => type == RszFieldType.String || type == RszFieldType.Resource;
    }


    /// <summary>
    /// rsz json patch
    /// </summary>
    public class RszClassPatch
    {
        public string? Name { get; set; }
        public string? ReplaceName { get; set; }
        public RszFieldPatch[]? FieldPatches { get; set; }
    }


    public class RszFieldPatch
    {
        public string? Name { get; set; }
        public string? ReplaceName { get; set; }
        public string? OriginalType { get; set; }
        [JsonConverter(typeof(EnumJsonConverter<RszFieldType>))]
        public RszFieldType Type { get; set; }
    }
}
