using System.Text.Json;
using System.Text.Json.Serialization;

namespace RszTool.Common
{
    public class HexUIntJsonConverter : JsonConverter<uint>
    {
        public override uint Read(ref Utf8JsonReader reader, Type type, JsonSerializerOptions options)
        {
            if (reader.TokenType == JsonTokenType.String)
            {
                string? text = reader.GetString();
                if (text == null) return default;
                return Convert.ToUInt32(text, 16);
            }

            return reader.GetUInt32();
        }

        public override void Write(Utf8JsonWriter writer, uint value, JsonSerializerOptions options)
        {
            writer.WriteStringValue($"0x{value:x}");
        }
    }

    public class HexIntJsonConverter : JsonConverter<int>
    {
        public override int Read(ref Utf8JsonReader reader, Type type, JsonSerializerOptions options)
        {
            if (reader.TokenType == JsonTokenType.String)
            {
                string? text = reader.GetString();
                if (text == null) return default;
                return Convert.ToInt32(text, 16);
            }

            return reader.GetInt32();
        }

        public override void Write(Utf8JsonWriter writer, int value, JsonSerializerOptions options)
        {
            writer.WriteStringValue($"0x{value:x}");
        }
    }

    public class HexIntPtrJsonConverter : JsonConverter<nint>
    {
        public override nint Read(ref Utf8JsonReader reader, Type type, JsonSerializerOptions options)
        {
            if (reader.TokenType == JsonTokenType.String)
            {
                string? text = reader.GetString();
                if (text == null) return default;
                return (nint)Convert.ToInt64(text, 16);
            }

            return (nint)reader.GetInt64();
        }

        public override void Write(Utf8JsonWriter writer, nint value, JsonSerializerOptions options)
        {
            writer.WriteStringValue($"0x{value:x}");
        }
    }

    public class ConstObjectConverter : JsonConverter<object?>
    {
        public override object? Read(ref Utf8JsonReader reader, Type type, JsonSerializerOptions options)
        {
            if (reader.TokenType == JsonTokenType.String)
            {
                return reader.GetString();
            }
            else if (reader.TokenType == JsonTokenType.Number)
            {
                return reader.GetDecimal();
            }
            return null;
        }

        public override void Write(Utf8JsonWriter writer, object? value, JsonSerializerOptions options)
        {
            if (value is string stringValue)
            {
                writer.WriteStringValue(stringValue);
            }
            else if (value is IConvertible convertible)
            {
                writer.WriteNumberValue(convertible.ToDecimal(null));
            }
        }
    }


    public class EnumJsonConverter<T> : JsonConverter<T> where T : struct
    {
        public override bool CanConvert(Type typeToConvert)
        {
            return typeToConvert.IsEnum;
        }

        public override T Read(ref Utf8JsonReader reader, Type enumType, JsonSerializerOptions options)
        {
            if (reader.TokenType == JsonTokenType.String)
            {
                string? text = reader.GetString();
                if (text != null)
                {
#if NET5_0_OR_GREATER
                    return Enum.Parse<T>(text);
#else
                    return (T)Enum.Parse(typeof(T), text);
#endif
                }
            }
            return default;
        }

        public override void Write(Utf8JsonWriter writer, T value, JsonSerializerOptions options)
        {
            writer.WriteStringValue(value.ToString());
        }
    }
}
