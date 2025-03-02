using RszTool;
using System.ComponentModel;
using System.Text.Json;
using System.Text.Json.Serialization;

class Program
{
    static void Main(string[] args)
    {
        var path = args[0];
        var outPath = args.ElementAtOrDefault(1) ?? "output.json";

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

        List<Dictionary<string, object>> output = [];

        // For some reason, the object list (which contains the actual instance objects we care about) holds all the objects
        // in the first element, inside the first element in its `Values` field.
        var instances = (List<object>)file.RSZ.ObjectList[0].Values[0];

        foreach (var instance in instances)
        {
            var item = (RszInstance)instance;
            var contents = ProcessObject(item);

            if (contents.Count == 0)
                continue;

            output.Add(contents);
        }

        using FileStream fs = File.Create(outPath);
        fs.Write(JsonSerializer.SerializeToUtf8Bytes(output));
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
                return ProcessObject(instance);
        }
        else if (value is List<object> list)
            return list.Select(Flatten).ToArray();
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