using RszTool;
using System.ComponentModel;
using System.Numerics;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Xml.Linq;

class Program
{
    static void Main(string[] args)
    {
        var path = args[0];
        var outPath = args.ElementAtOrDefault(1) ?? "output.json";

        var valueIndex = -1;

        if (args.Length > 2)
        {
            var indexParsed = int.TryParse(args.ElementAtOrDefault(2), out valueIndex);

            if (!indexParsed)
            {
                Console.Error.WriteLine("Could not parse index");
                return;
            }
        }

        if (path == null || path.Length == 0)
        {
            Console.Error.WriteLine("Missing <path> argument");
            return;
        }

        RszFileOption option = new(GameName.mhwilds);
        UserFile file = new(option, new FileHandler(path));
        file.Read();

        if (file.RSZ == null)
        {
            Console.Error.WriteLine("Could not read file");
            return;
        }

        object output;
        
        if (valueIndex > -1)
        {
            output = ProcessValue(file.RSZ.ObjectList[0].Values[valueIndex]);
        } else
        {
            var dict = ProcessInstance(file.RSZ.ObjectList[0]);
            output = Flatten(dict);
        }

        using FileStream fs = File.Create(outPath);
        fs.Write(JsonSerializer.SerializeToUtf8Bytes(output));
    }

    private static Dictionary<string, object> ProcessInstance(RszInstance instance)
    {
        Dictionary<string, object> output = [];

        for (int i = 0; i < instance.Fields.Length; i++)
        {
            var field = instance.Fields[i];
            var value = instance.Values[i];

            var element = ProcessValue(value);

            if (element == null)
                continue;

            output.Add(field.name, element);
        }

        return output;
    }

    private static object ProcessValue(object value) {
        if (value is List<object> list)
        {
            List<object> output = [];

            foreach (var instance in list)
            {
                output.Add(Flatten(instance));
            }

            if (output.Count == 1)
                return output[0];
            else
                return output;
        }
        else if (value is RszInstance child)
        {
            if (child.Values.Length == 1 && child.Values[0] is List<object>)
                return ProcessValue(child.Values[0]);
            else
                return Flatten(ProcessInstance(child));
        }
        else
            return Flatten(value);
    }

    private static Dictionary<string, object> ProcessObject(RszInstance instance)
    {
        Dictionary<string, object> contents = [];

        for (int i = 0; i < instance.Fields.Length; i++)
        {
            contents.Add(instance.Fields[i].name, Flatten(instance.Values[i]));
        }

        return contents;
    }

    private static object Flatten(object value)
    {
        if (value is RszInstance instance)
        {
            if (instance.Values.Length == 1)
                return instance.Values[0];
            else
                return Flatten(ProcessObject(instance));
        }
        else if (value is List<object> list)
            return list.Select(Flatten).ToArray();
        else if (value is Dictionary<string, object> dict && dict.Count == 1)
            return dict.First().Value;
        else if (value is Vector3 vec)
        {
            Dictionary<string, float> d = [];
            d.Add("x", vec.X);
            d.Add("y", vec.Y);
            d.Add("z", vec.Z);

            return d;
        }
        else
            return value;
    }
}

class Entry(string name, object value)
{
    [JsonPropertyName("name")]
    public string Name { get; set; } = name;
    [JsonPropertyName("value")]
    public object Value { get; set; } = value;
}