using System.Buffers;
using System.Runtime.InteropServices;
using System.Text;

namespace System
{
#if !NET5_0_OR_GREATER
    public static class SystemPatch
    {
        public static bool Contains(this string text, char ch)
        {
            return text.IndexOf(ch) != -1;
        }

        public static bool StartsWith(this string text, char ch)
        {
            return text[0] == ch;
        }

        public static bool EndsWith(this string text, char ch)
        {
            return text[text.Length - 1] == ch;
        }

        public static void ArrayFill<T>(T[] array, T value)
        {
            if (array == null)
            {
                throw new NullReferenceException("array is null");
            }

            for (int i = 0; i < array.Length; i++)
            {
                array[i] = value;
            }
        }

        public static bool IsAssignableTo(this Type typeA, Type typeB)
        {
            return typeB.IsAssignableFrom(typeA);
        }
    }


    public static class IOPatch
    {
        public static int Read(this Stream stream, Span<byte> buffer)
        {
            byte[] sharedBuffer = ArrayPool<byte>.Shared.Rent(buffer.Length);
            try
            {
                int numRead = stream.Read(sharedBuffer, 0, buffer.Length);
                if ((uint)numRead > (uint)buffer.Length)
                {
                    throw new IOException("Stream too long.");
                }
                new Span<byte>(sharedBuffer, 0, numRead).CopyTo(buffer);
                return numRead;
            }
            finally { ArrayPool<byte>.Shared.Return(sharedBuffer); }
        }

        public static void Write(this Stream stream, ReadOnlySpan<byte> buffer)
        {
            byte[] sharedBuffer = ArrayPool<byte>.Shared.Rent(buffer.Length);
            try
            {
                buffer.CopyTo(sharedBuffer);
                stream.Write(sharedBuffer, 0, buffer.Length);
            }
            finally { ArrayPool<byte>.Shared.Return(sharedBuffer); }
        }

        public static void Write(this Stream stream, byte[] buffer)
        {
            stream.Write(buffer, 0, buffer.Length);
        }
    }


    public static class TextPatch
    {
        public static bool Equals(this StringBuilder sb, ReadOnlySpan<char> span)
        {
            if (span.Length != sb.Length)
            {
                return false;
            }

            for (int i = 0; i < sb.Length; i++)
            {
                if (sb[i] != span[i]) return false;
            }
            return true;
        }

        public static StringBuilder Append(this StringBuilder sb, ReadOnlySpan<char> value)
        {
            unsafe
            {
                fixed (char* ptr = &MemoryMarshal.GetReference(value))
                {
                    sb.Append(ptr, value.Length);
                }
            }
            return sb;
        }
    }


    public static class CollectionsPatch
    {
        public static TValue? GetValueOrDefault<TKey, TValue>(this Dictionary<TKey, TValue> dict, TKey key)
        {
            return dict.TryGetValue(key, out var value) ? value : default;
        }

        public static TValue GetValueOrDefault<TKey, TValue>(this Dictionary<TKey, TValue> dict, TKey key, TValue defaultValue)
        {
            return dict.TryGetValue(key, out var value) ? value : defaultValue;
        }
    }
#endif
}
