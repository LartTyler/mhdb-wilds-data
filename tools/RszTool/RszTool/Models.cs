using System.Drawing;
using System.Reflection;
using System.Runtime.CompilerServices;
using RszTool.Common;

namespace RszTool
{
    public interface IOffsetField
    {
        int Offset { get; set; }
        bool Read(FileHandler handler);
        bool Write(FileHandler handler);

        object ObejctValue { get; }
    }


    public struct OffsetField<T> : IOffsetField where T : struct
    {
        public OffsetField()
        {
        }

        public T Value { get; set; }
        public int Offset { get; set; } = -1;

        public readonly object ObejctValue => Value;
        public static implicit operator T(OffsetField<T> field) => field.Value;

        public bool Read(FileHandler handler)
        {
            Value = handler.Read<T>();
            return true;
        }

        public readonly bool Write(FileHandler handler)
        {
            return handler.Write(Value);
        }
    }


    public interface IModel
    {
        long Start { get; set; }
        long Size { get; }
        bool Read(FileHandler handler);
        bool Write(FileHandler handler);
    }


    public static class IModelExtensions
    {
        public static bool Read(this IModel model, FileHandler handler, long start, bool jumpBack = true)
        {
            long pos = handler.Tell();
            if (start != -1)
            {
                handler.Seek(start);
            }
            bool result = model.Read(handler);
            if (jumpBack) handler.Seek(pos);
            return result;
        }

        public static bool Read<T>(this List<T> list, FileHandler handler, int count) where T : IModel, new()
        {
            for (int i = 0; i < count; i++)
            {
                T item = new();
                if (!item.Read(handler)) return false;
                list.Add(item);
            }
            return true;
        }

        public static bool Write(this IModel model, FileHandler handler, long start, bool jumpBack = true)
        {
            long pos = handler.Tell();
            if (start != -1)
            {
                handler.Seek(start);
            }
            bool result = model.Write(handler);
            if (jumpBack) handler.Seek(pos);
            return result;
        }

        public static bool Rewrite(this IModel model, FileHandler handler, bool jumpBack = true)
        {
            return Write(model, handler, model.Start, jumpBack);
        }

        public static bool Write(this IEnumerable<IModel> list, FileHandler handler)
        {
            foreach (var item in list)
            {
                if (!item.Write(handler)) return false;
            }
            return true;
        }

        public static bool Rewrite(this IEnumerable<IModel> list, FileHandler handler)
        {
            foreach (var item in list)
            {
                if (!item.Rewrite(handler)) return false;
            }
            return true;
        }
    }


    public class StructModel<T> : ICloneable, IModel where T : struct
    {
        public T Data = default;
        public long Start { get; set; } = -1;
        public long Size => Unsafe.SizeOf<T>();

        public bool Read(FileHandler handler)
        {
            Start = handler.Tell();
            handler.Read(ref Data);
            return true;
        }

        public bool Write(FileHandler handler)
        {
            Start = handler.Tell();
            return handler.Write(ref Data);
        }

        public object Clone()
        {
            return MemberwiseClone();
        }
    }


    public abstract class BaseModel : IModel, ICloneable
    {
        public long Start { get; set; }
        public long Size { get; protected set; }

        public bool Read(FileHandler handler)
        {
            handler.CheckRange();
            Start = handler.Tell();
            bool result = DoRead(handler);
            Size = handler.Tell() - Start;
            return result;
        }

        public bool Write(FileHandler handler)
        {
            Start = handler.Tell();
            bool result = DoWrite(handler);
            Size = handler.Tell() - Start;
            return result;
        }

        protected abstract bool DoRead(FileHandler handler);

        protected abstract bool DoWrite(FileHandler handler);

        public virtual object Clone()
        {
            return MemberwiseClone();
        }
    }


    public abstract class ReadWriteModel : BaseModel
    {
        protected abstract bool ReadWrite(IFileHandlerAction action);

        protected override bool DoRead(FileHandler handler)
        {
            return ReadWrite(new FileHandlerRead(handler));
        }

        protected override bool DoWrite(FileHandler handler)
        {
            return ReadWrite(new FileHandlerWrite(handler));
        }
    }
}
