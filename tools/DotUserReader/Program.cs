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

        var valueIndex = 0;

        if (args.Length >= 2)
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

        List<Dictionary<string, object>> output = [];

        var instances = (List<object>)file.RSZ.ObjectList[0].Values[valueIndex];

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