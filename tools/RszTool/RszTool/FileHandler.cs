using RszTool.Common;
using System.Collections;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;

namespace RszTool
{
    public class FileHandler : IDisposable
    {
        public string? FilePath { get; private set; }
        public Stream Stream { get; private set; }
        public long Offset { get; set; }
        public bool IsMemory => Stream is MemoryStream;
        private StringTable? StringTable;
        private OffsetContentTable? OffsetContentTable;
        private Sunday? searcher = new();

        public long Position => Stream.Position;

        public FileHandler() : this(new MemoryStream())
        {
        }

        public FileHandler(string path, bool holdFile = false)
        {
            Open(path, holdFile);
            Stream ??= new MemoryStream();
        }

        public FileHandler(Stream stream)
        {
            Stream = stream;
        }

        ~FileHandler()
        {
            Dispose(false);
        }

        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        protected virtual void Dispose(bool disposing)
        {
            if (disposing)
            {
                Stream.Dispose();
            }
        }

        public void Open(string path, bool holdFile = false)
        {
            FilePath = path;
            FileStream fileStream = new(path, FileMode.OpenOrCreate, FileAccess.ReadWrite);
            if (!holdFile) {
                Stream = new MemoryStream();
                fileStream.CopyTo(Stream);
                fileStream.Dispose();
                Stream.Position = 0;
            } else {
                Stream = fileStream;
            }
        }

        public void Reopen()
        {
            if (FilePath != null)
            {
                Stream.Dispose();
                Open(FilePath, !IsMemory);
            }
        }

        public FileHandler AsMemory()
        {
            var newStream = new MemoryStream();
            long pos = Stream.Position;
            Stream.Position = 0;
            Stream.CopyTo(newStream);
            Stream.Position = pos;
            newStream.Position = pos;
            return new FileHandler(newStream) { Offset = Offset };
        }

        public FileHandler WithOffset(long offset)
        {
            return new FileHandler(Stream) { Offset = Offset + offset };
        }

        public void Save(string? path = null)
        {
            if ((path == null || path == FilePath) && !IsMemory)
            {
                Stream.Flush();
            }
            else
            {
                path ??= FilePath;
                if (path == null) return;
                using FileStream fileStream = File.Create(path);
                long pos = Stream.Position;
                Stream.Position = 0;
                Stream.CopyTo(fileStream);
                Stream.Position = pos;
            }
        }

        public void SaveAs(string path)
        {
            FilePath = path;
            FileStream fileStream = File.Create(path);
            long pos = Stream.Position;
            Stream.Position = 0;
            Stream.CopyTo(fileStream);
            Stream.Position = pos;
            if (!IsMemory)
            {
                Stream = fileStream;
            }
            else
            {
                fileStream.Dispose();
            }
        }

        public void Clear()
        {
            Stream.SetLength(0);
            Stream.Flush();
        }

        public long FileSize()
        {
            return Stream.Length;
        }

        public void Seek(long tell)
        {
            Stream.Position = tell + Offset;
        }

        public void Align(int alignment)
        {
            long delta = Stream.Position % alignment;
            if (delta != 0)
            {
                Stream.Position += alignment - delta;
            }
        }

        public void SeekOffsetAligned(int offset, int align = 4)
        {
            Seek(Utils.AlignSize(Tell() + offset, align));
        }

        public long Tell()
        {
            return Stream.Position - Offset;
        }

        public void Skip(long skip)
        {
            Stream.Seek(skip, SeekOrigin.Current);
        }

        public void CheckRange()
        {
            if (Stream.Position > Stream.Length)
            {
                throw new IndexOutOfRangeException($"Seek out of range {Stream.Position} > {Stream.Length}");
            }
        }

        public readonly struct JumpBackGuard : IDisposable
        {
            private readonly FileHandler handler;
            private readonly long position;
            private readonly bool jumpBack;

            public JumpBackGuard(FileHandler handler, bool jumpBack = true)
            {
                this.handler = handler;
                position = handler.Tell();
                this.jumpBack = jumpBack;
            }

            public void Dispose()
            {
                if (jumpBack)
                {
                    handler.Seek(position);
                }
            }
        }

        public JumpBackGuard SeekJumpBack(bool jumpBack = true)
        {
            return new JumpBackGuard(this, jumpBack);
        }

        public JumpBackGuard SeekJumpBack(long tell, bool jumpBack = true)
        {
            var defer = new JumpBackGuard(this, jumpBack);
            Seek(tell);
            return defer;
        }

        public byte ReadByte()
        {
            return (byte)Stream.ReadByte();
        }

        public byte ReadByte(long tell, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            return (byte)Stream.ReadByte();
        }

        public sbyte ReadSByte()
        {
            return (sbyte)Stream.ReadByte();
        }

        public sbyte ReadSByte(long tell, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            return (sbyte)Stream.ReadByte();
        }

        public char ReadChar()
        {
            return Read<char>();
        }

        public char ReadChar(long tell, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            return Read<char>();
        }

        public bool ReadBoolean()
        {
            return Stream.ReadByte() != 0;
        }

        public bool ReadBoolean(long tell, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            return Stream.ReadByte() != 0;
        }

        public short ReadShort()
        {
            return Read<short>();
        }

        public short ReadShort(long tell, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            return Read<short>();
        }

        public ushort ReadUShort()
        {
            return Read<ushort>();
        }

        public ushort ReadUShort(long tell, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            return Read<ushort>();
        }

        public int ReadInt(long tell, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            return Read<int>();
        }

        public int ReadInt()
        {
            return Read<int>();
        }

        public uint ReadUInt(long tell, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            return Read<uint>();
        }

        public uint ReadUInt()
        {
            return Read<uint>();
        }

        public long ReadInt64()
        {
            return Read<long>();
        }

        public long ReadInt64(long tell, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            return Read<long>();
        }

        public ulong ReadUInt64()
        {
            return Read<ulong>();
        }

        public ulong ReadUInt64(long tell, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            return Read<ulong>();
        }

        public float ReadFloat(long tell, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            return Read<float>();
        }

        public float ReadFloat()
        {
            return Read<float>();
        }

        public double ReadDouble(long tell, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            return Read<double>();
        }

        public double ReadDouble()
        {
            return Read<double>();
        }

        public int ReadBytes(byte[] buffer, int length = -1)
        {
            return Stream.Read(buffer, 0, length == -1 ? buffer.Length : length);
        }

        public int ReadBytes(long tell, byte[] buffer, int length = -1, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            return Stream.Read(buffer, 0, length == -1 ? buffer.Length : length);
        }

        public byte[] ReadBytes(long tell, int length, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            var buffer = new byte[length];
            Stream.Read(buffer);
            return buffer;
        }

        public void WriteByte(byte value)
        {
            Stream.WriteByte(value);
        }

        public void WriteByte(long tell, byte value, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            Stream.WriteByte(value);
        }

        public void WriteSByte(sbyte value)
        {
            Stream.WriteByte((byte)value);
        }

        public void WriteSByte(long tell, sbyte value, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            Stream.WriteByte((byte)value);
        }

        public void WriteChar(char value)
        {
            Write(value);
        }

        public void WriteChar(long tell, char value, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            Write(value);
        }

        public void WriteBoolean(bool value)
        {
            Stream.WriteByte(value ? (byte)1 : (byte)0);
        }

        public void WriteBoolean(long tell, bool value, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            Stream.WriteByte(value ? (byte)1 : (byte)0);
        }

        public void WriteShort(short value)
        {
            Write(value);
        }

        public void WriteShort(long tell, short value, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            Write(value);
        }

        public void WriteUShort(ushort value)
        {
            Write(value);
        }

        public void WriteUShort(long tell, ushort value, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            Write(value);
        }

        public void WriteInt(int value)
        {
            Write(value);
        }

        public void WriteInt(long tell, int value, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            Write(value);
        }

        public void WriteUInt(uint value)
        {
            Write(value);
        }

        public void WriteUInt(long tell, uint value, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            Write(value);
        }

        public void WriteInt64(long value)
        {
            Write(value);
        }

        public void WriteInt64(long tell, long value, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            Write(value);
        }

        public void WriteUInt64(ulong value)
        {
            Write(value);
        }

        public void WriteUInt64(long tell, ulong value, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            Write(value);
        }

        public void WriteBytes(byte[] buffer, int length = -1)
        {
            Stream.Write(buffer, 0, length == -1 ? buffer.Length : length);
        }

        public void WriteBytes(long tell, byte[] buffer, int length = -1, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            WriteBytes(buffer, length);
        }

        public void FillBytes(byte value, int length)
        {
            if (length < 0)
            {
                // throw new ArgumentOutOfRangeException(nameof(length), $"{nameof(length)} must > 0");
                return;
            }
            if (length < 256)
            {
                Span<byte> bytes = stackalloc byte[length];
                bytes.Fill(value);
                WriteSpan<byte>(bytes);
            }
            else
            {
                Span<byte> bytes = stackalloc byte[256];
                bytes.Fill(value);
                int count = length >> 8;
                for (int i = 0; i < count; i++)
                {
                    WriteSpan<byte>(bytes);
                }
                if ((length & 0xFF) != 0)
                {
                    WriteSpan<byte>(bytes.Slice(0, length & 0xFF));
                }
            }
        }

        public void FillBytes(long tell, byte value, int length, bool jumpBack = true)
        {
            using var defer = SeekJumpBack(tell, jumpBack);
            FillBytes(value, length);
        }

        public static string MarshalStringTrim(string text)
        {
            int n = text.IndexOf('\0');
            if (n != -1)
            {
                text = text.Substring(0, n);
            }
            return text;
        }

        private static string AsciiGetString(Span<byte> buffer)
        {
#if !NET5_0_OR_GREATER
            return Encoding.ASCII.GetString(buffer.ToArray());
#else
            return Encoding.ASCII.GetString(buffer);
#endif
        }

        public string ReadAsciiString(long pos = -1, int charCount = -1, bool jumpBack = true)
        {
            long originPos = Tell();
            if (pos != -1) Seek(pos);
            string? result = null;
            if (charCount > 1024)
            {
                throw new Exception($"{nameof(charCount)} {charCount} too large");
            }
            if (charCount == -1) charCount = 128;
            Span<byte> buffer = charCount <= 128 ? stackalloc byte[charCount] : new byte[charCount];
            int readCount = Stream.Read(buffer);
            if (readCount != 0)
            {
                int n = buffer.IndexOf((byte)0);
                if (n != -1)
                {
                    result = n == 0 ? "" : AsciiGetString(buffer.Slice(0, n));
                }
            }
            else
            {
                result = "";
            }
            if (result == null)
            {
                StringBuilder sb = new();
                sb.Append(AsciiGetString(buffer));
                do
                {
                    readCount = Stream.Read(buffer);
                    if (readCount != 0)
                    {
                        int n = buffer.IndexOf((byte)0);
                        sb.Append(AsciiGetString(n != -1 ? buffer.Slice(0, n): buffer));
                        if (n != -1) break;
                    }
                } while (readCount == buffer.Length);
                result = sb.ToString();
            }
            Seek(jumpBack ? originPos : originPos + result.Length + 1);
            return result;
        }

        public bool WriteAsciiString(string text)
        {
            return WriteBytes(Encoding.ASCII.GetBytes(text)) && Write<byte>(0);
        }

        public string ReadWString(long pos = -1, int charCount = -1, bool jumpBack = true)
        {
            long originPos = Tell();
            if (pos != -1) Seek(pos);
            string? result = null;
            if (charCount > 1024)
            {
                throw new Exception($"{nameof(charCount)} {charCount} too large");
            }
            if (charCount == -1) charCount = 128;
            Span<char> buffer = charCount <= 128 ? stackalloc char[charCount] : new char[charCount];
            Span<byte> bytes = MemoryMarshal.AsBytes(buffer);
            int readCount = Stream.Read(bytes);
            if (readCount != 0)
            {
                int n = buffer.IndexOf((char)0);
                if (n != -1)
                {
                    result = n == 0 ? "" : buffer.Slice(0, n).ToString();
                }
            }
            else
            {
                result = "";
            }
            if (result == null)
            {
                StringBuilder sb = new();
                sb.Append(buffer);
                do
                {
                    readCount = Stream.Read(bytes);
                    if (readCount != 0)
                    {
                        int n = buffer.IndexOf((char)0);
                        sb.Append(n != -1 ? buffer.Slice(0, n): buffer);
                        if (n != -1) break;
                    }
                } while (readCount == bytes.Length);
                result = sb.ToString();
            }
            Seek(jumpBack ? originPos : originPos + result.Length * 2 + 2);
            return result;
        }

        public int ReadWStringLength(long pos = -1, int maxLen = -1, bool jumpBack = true)
        {
            long originPos = Tell();
            if (pos != -1) Seek(pos);
            int length = 0;
            char newByte = ReadChar();
            while (newByte != 0)
            {
                length++;
                newByte = ReadChar();
            }
            if (jumpBack) Seek(originPos);
            return length;
        }

        public bool WriteWString(string text)
        {
            return WriteSpan(text.AsSpan()) && Write<ushort>(0);
        }

        public T Read<T>() where T : struct
        {
            T value = default;
            Stream.Read(MemoryUtils.StructureAsBytes(ref value));
            return value;
        }

        public T Read<T>(long tell, bool jumpBack = true) where T : struct
        {
            long pos = Tell();
            Seek(tell);
            T value = Read<T>();
            if (jumpBack) Seek(pos);
            return value;
        }

        public int Read<T>(ref T value) where T : struct
        {
            return Stream.Read(MemoryUtils.StructureAsBytes(ref value));
        }

        public int Read<T>(long tell, ref T value, bool jumpBack = true) where T : struct
        {
            long pos = Tell();
            Seek(tell);
            int result = Read(ref value);
            if (jumpBack) Seek(pos);
            return result;
        }

        public bool Write<T>(T value) where T : struct
        {
            Stream.Write(MemoryUtils.StructureAsBytes(ref value));
            return true;
        }

        public bool Write<T>(long tell, T value, bool jumpBack = true) where T : struct
        {
            long pos = Tell();
            Seek(tell);
            bool result = Write(value);
            if (jumpBack) Seek(pos);
            return result;
        }

        public bool Write<T>(ref T value) where T : struct
        {
            Stream.Write(MemoryUtils.StructureAsBytes(ref value));
            return true;
        }

        public bool Write<T>(long tell, ref T value, bool jumpBack = true) where T : struct
        {
            long pos = Tell();
            Seek(tell);
            bool result = Write(ref value);
            if (jumpBack) Seek(pos);
            return result;
        }

        /// <summary>
        /// 获取start..end之间的数据，长度在2-128
        /// 涉及unsafe操作，注意内存范围和对齐
        /// </summary>
        public static Span<byte> GetRangeSpan<TS, TE>(ref TS start, ref TE end) where TS : struct where TE : struct
        {
            unsafe
            {
                var startPtr = (nint)Unsafe.AsPointer(ref start);
                var endPtr = (nint)Unsafe.AsPointer(ref end) + Unsafe.SizeOf<TE>();
                int size = (int)(endPtr - startPtr);
                if (size < 2 || size > 128)
                {
                    throw new InvalidDataException($"Size {size} is out of range [2, 128]");
                }
                return new Span<byte>((void*)startPtr, size);
            }
        }

        /// <summary>
        /// 读取数据到start..end，长度在2-128
        /// 涉及unsafe操作，注意内存范围和对齐
        /// </summary>
        public int ReadRange<TS, TE>(ref TS start, ref TE end) where TS : struct where TE : struct
        {
            return Stream.Read(GetRangeSpan(ref start, ref end));
        }

        /// <summary>
        /// 写入start..end范围内的数据，长度在2-128
        /// 涉及unsafe操作，注意内存范围和对齐
        /// </summary>
        public bool WriteRange<TS, TE>(ref TS start, ref TE end) where TS : struct where TE : struct
        {
            Stream.Write(GetRangeSpan(ref start, ref end));
            return true;
        }

        public byte[] ReadBytes(int length)
        {
            byte[] buffer = new byte[length];
            Stream.Read(buffer);
            return buffer;
        }

        public bool WriteBytes(byte[] buffer)
        {
            Stream.Write(buffer);
            return true;
        }

        /// <summary>读取数组</summary>
        public T[] ReadArray<T>(int length) where T : struct
        {
            T[] array = new T[length];
            Stream.Read(MemoryMarshal.AsBytes((Span<T>)array));
            return array;
        }

        /// <summary>读取数组</summary>
        public bool ReadArray<T>(T[] array) where T : struct
        {
            Stream.Read(MemoryMarshal.AsBytes((Span<T>)array));
            return true;
        }

        /// <summary>读取数组</summary>
        public bool ReadArray<T>(T[] array, int start = 0, int length = -1) where T : struct
        {
            if (length == -1 || length > array.Length - start)
            {
                length = array.Length - start;
            }
            Stream.Read(MemoryMarshal.AsBytes(array.AsSpan(start, length)));
            return true;
        }

        /// <summary>写入数组</summary>
        public bool WriteArray<T>(T[] array) where T : struct
        {
            Stream.Write(MemoryMarshal.AsBytes((ReadOnlySpan<T>)array));
            return true;
        }

        /// <summary>写入数组</summary>
        public bool WriteArray<T>(T[] array, int start = 0, int length = -1) where T : struct
        {
            if (length == -1 || length > array.Length - start)
            {
                length = array.Length - start;
            }
            Stream.Write(MemoryMarshal.AsBytes(new ReadOnlySpan<T>(array, start, length)));
            return true;
        }

        /// <summary>读取数组</summary>
        public bool ReadSpan<T>(Span<T> span) where T : struct
        {
            Stream.Write(MemoryMarshal.AsBytes(span));
            return true;
        }

        /// <summary>写入数组</summary>
        public bool WriteSpan<T>(ReadOnlySpan<T> span) where T : struct
        {
            Stream.Write(MemoryMarshal.AsBytes(span));
            return true;
        }

        /// <summary>查找字节数组</summary>
        public long FindBytes(byte[] pattern, in SearchParam param = default)
        {
            const int PAGE_SIZE = 8192;
            byte[] buffer = new byte[PAGE_SIZE];
            long addr = param.start;
            int sizeAligned = pattern.Length; // 数据大小对齐4字节

            if ((sizeAligned & 3) != 0)
            {
                sizeAligned += 4 - (sizeAligned & 3);
            }

            long end = param.end;
            if (end == -1)
            {
                end = FileSize();
            }

            searcher ??= new();
            searcher.Update(pattern, param.wildcard);

            while (addr < end)
            {
                int readCount = ReadBytes(addr, buffer, PAGE_SIZE);
                if (readCount != 0)
                {
                    int result = searcher.Search(buffer, 0, readCount, param.ordinal);
                    if (result != -1)
                    {
                        return addr + result;
                    }
                }
                addr += PAGE_SIZE - sizeAligned;
            }

            return -1;
        }

        public long FindFirst(byte[] pattern, in SearchParam param = default)
        {
            return FindBytes(pattern, param);
        }

        public long FindFirst(string pattern, in SearchParam param = default, Encoding? encoding = null)
        {
            encoding ??= Encoding.Unicode;
            return FindBytes(encoding.GetBytes(pattern), param);
        }

        public long FindFirst<T>(T pattern, in SearchParam param = default) where T : struct
        {
            return FindBytes(MemoryUtils.StructureRefToBytes(ref pattern), param);
        }

        public void InsertBytes(Span<byte> buffer, long position)
        {
            var stream = Stream;
            // 将当前位置保存到临时变量中
            long currentPosition = stream.Position;

            // 将流的位置设置为插入点
            stream.Position = position;

            // 将后续数据读取到临时缓冲区
            byte[] tempBuffer = new byte[stream.Length - position];
            int bytesRead = stream.Read(tempBuffer, 0, tempBuffer.Length);

            // 将插入数据写入到流中
            stream.Position = position;
            stream.Write(buffer);

            // 将后续数据写入到流中
            stream.Write(tempBuffer, 0, bytesRead);

            // 将流的位置恢复到原始位置
            stream.Position = currentPosition;
        }

        public string ReadOffsetAsciiString()
        {
            return ReadAsciiString(ReadInt64());
        }

        public void ReadOffsetAsciiString(out string text)
        {
            text = ReadAsciiString(ReadInt64());
        }

        public string ReadOffsetWString()
        {
            return ReadWString(ReadInt64());
        }

        public void ReadOffsetWString(out string text)
        {
            long offset = ReadInt64();
            text = ReadWString(offset);
        }

        public void WriteOffsetWString(string text)
        {
            StringTableAdd(text);
            WriteInt64(0);
        }

        public void StringTableAdd(string? text)
        {
            if (text != null)
            {
                StringTable ??= new();
                StringTable.Add(text, Tell());
            }
        }

        /// <summary>
        /// 写入字符串表字符串和偏移
        /// </summary>
        public void StringTableFlush()
        {
            StringTable?.Flush(this);
        }

        /// <summary>
        /// 写入字符串表的字符串
        /// </summary>
        public void StringTableWriteStrings()
        {
            StringTable?.WriteStrings(this);
        }

        /// <summary>
        /// 写入字符串表的偏移，并清空数据
        /// </summary>
        public void StringTableFlushOffsets()
        {
            StringTable?.FlushOffsets(this);
        }

        public void OffsetContentTableAdd(Action<FileHandler> write)
        {
            OffsetContentTable ??= new();
            OffsetContentTable.Add(write, Tell());
        }

        /// <summary>
        /// 写入字符串表字符串和偏移
        /// </summary>
        public void OffsetContentTableFlush()
        {
            OffsetContentTable?.Flush(this);
        }

        /// <summary>
        /// 写入字符串表的字符串
        /// </summary>
        public void OffsetContentTableWriteContents()
        {
            OffsetContentTable?.WriteContents(this);
        }

        /// <summary>
        /// 写入字符串表的偏移，并清空数据
        /// </summary>
        public void OffsetContentTableFlushOffsets()
        {
            OffsetContentTable?.FlushOffsets(this);
        }
    }


    public class StringTableItem
    {
        public string Text { get; }
        // 引用改字符串的偏移集合
        public HashSet<long> OffsetStart { get; } = new();
        public long TextStart { get; set; } = -1;
        public bool IsAscii { get; set; }

        public StringTableItem(string text)
        {
            Text = text;
        }

        public void Write(FileHandler handler)
        {
            if (IsAscii)
            {
                handler.WriteAsciiString(Text);
            }
            else
            {
                handler.WriteWString(Text);
            }
        }
    }


    /// <summary>
    /// 待写入的字符串表
    /// </summary>
    public class StringTable : IEnumerable<StringTableItem>
    {
        private List<StringTableItem> Items { get; } = new();
        private Dictionary<string, StringTableItem> StringMap { get; } = new();

        public int Count => Items.Count;

        public void Clear()
        {
            Items.Clear();
            StringMap.Clear();
        }

        public void Add(string text, long offset)
        {
            if (!StringMap.TryGetValue(text, out var item))
            {
                StringMap[text] = item = new(text);
                Items.Add(item);
            }
            item.OffsetStart.Add(offset);
        }

        public void Flush(FileHandler handler)
        {
            if (Count == 0) return;
            foreach (var item in Items)
            {
                item.TextStart = handler.Tell();
                foreach (var offsetStart in item.OffsetStart)
                {
                    handler.WriteInt64(offsetStart, item.TextStart);
                }
                handler.Seek(item.TextStart);
                item.Write(handler);
            }
            Clear();
        }

        public void WriteStrings(FileHandler handler)
        {
            if (Count == 0) return;
            foreach (var item in Items)
            {
                item.TextStart = handler.Tell();
                item.Write(handler);
            }
        }

        public void FlushOffsets(FileHandler handler)
        {
            if (Count == 0) return;
            foreach (var item in Items)
            {
                if (item.TextStart == -1)
                {
                    throw new Exception($"StringStart of {item.Text} not set");
                }
                foreach (var offsetStart in item.OffsetStart)
                {
                    handler.WriteInt64(offsetStart, item.TextStart);
                }
            }
            Clear();
        }

        public IEnumerator<StringTableItem> GetEnumerator() => Items.GetEnumerator();

        IEnumerator IEnumerable.GetEnumerator() => Items.GetEnumerator();
    }


    public class OffsetContent
    {
        public long OffsetStart { get; set; }
        public long Offset { get; set; }
        public Action<FileHandler> Write { get; set; }

        public OffsetContent(Action<FileHandler> write)
        {
            Write = write;
        }
    }


    public class OffsetContentTable : IEnumerable<OffsetContent>
    {
        public List<OffsetContent> Items { get; } = new();
        public int Count => Items.Count;

        public void Clear()
        {
            Items.Clear();
        }

        public void Add(Action<FileHandler> write, long offset)
        {
            var item = new OffsetContent(write)
            {
                OffsetStart = offset
            };
            Items.Add(item);
        }

        public void Flush(FileHandler handler)
        {
            if (Count == 0) return;
            foreach (var item in Items)
            {
                item.Offset = handler.Tell();
                handler.WriteInt64(item.OffsetStart, item.Offset);
                item.Write(handler);
            }
            Clear();
        }

        public void WriteContents(FileHandler handler)
        {
            if (Count == 0) return;
            foreach (var item in Items)
            {
                item.Offset = handler.Tell();
                item.Write(handler);
            }
        }

        public void FlushOffsets(FileHandler handler)
        {
            if (Count == 0) return;
            foreach (var item in Items)
            {
                if (item.Offset == -1)
                {
                    throw new Exception($"Offset of {item} not set");
                }
                handler.WriteInt64(item.OffsetStart, item.Offset);
            }
            Clear();
        }

        public IEnumerator<OffsetContent> GetEnumerator() => Items.GetEnumerator();

        IEnumerator IEnumerable.GetEnumerator() => Items.GetEnumerator();
    }


    public interface IFileHandlerAction
    {
        FileHandler Handler { get; }
        bool Success { get; }
        bool Handle<T>(ref T value) where T : struct;
        IFileHandlerAction HandleOffsetWString(ref string value);
    }


    public static class IFileHandlerActionExtension
    {
        public static IFileHandlerAction? Then<T>(this IFileHandlerAction action, ref T value) where T : struct
        {
            if (action.Handle(ref value))
            {
                return action;
            }
            return null;
        }

        public static IFileHandlerAction? Then<T>(this IFileHandlerAction action, bool condition, ref T value) where T : struct
        {
            return condition ? action.Then(ref value) : action;
        }

        public static IFileHandlerAction? Skip(this IFileHandlerAction action, long skip)
        {
            action.Handler.Skip(skip);
            return action;
        }
    }


    public struct FileHandlerRead(FileHandler handler) : IFileHandlerAction
    {
        public FileHandler Handler { get; set; } = handler;
        public int LastResult { get; set; } = -1;
        public readonly bool Success => LastResult != 0;

        public bool Handle<T>(ref T value) where T : struct
        {
            LastResult = Handler.Read(ref value);
            return Success;
        }

        public readonly IFileHandlerAction HandleOffsetWString(ref string value)
        {
            Handler.ReadOffsetWString(out value);
            return this;
        }
    }


    public struct FileHandlerWrite(FileHandler handler) : IFileHandlerAction
    {
        public FileHandler Handler { get; set; } = handler;
        public bool Success { get; set; }

        public bool Handle<T>(ref T value) where T : struct
        {
            Success = Handler.Write(ref value);
            return Success;
        }

        public readonly IFileHandlerAction HandleOffsetWString(ref string value)
        {
            Handler.WriteOffsetWString(value);
            return this;
        }
    }
}
