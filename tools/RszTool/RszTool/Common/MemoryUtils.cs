using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace RszTool.Common
{
    public static class MemoryUtils
    {
#if !NET5_0_OR_GREATER
        public static ref T AsRef<T>(Span<byte> span) where T : struct
        {
            int size = Unsafe.SizeOf<T>();
            if (size > (uint)span.Length)
            {
                throw new ArgumentOutOfRangeException($"{typeof(T).Name}");
            }
            return ref Unsafe.As<byte, T>(ref MemoryMarshal.GetReference(span));
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static unsafe Span<T> CreateSpan<T>(scoped ref T reference, int length) => new Span<T>(Unsafe.AsPointer(ref reference), length);

#else
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static ref T AsRef<T>(Span<byte> span) where T : struct => ref MemoryMarshal.AsRef<T>(span);

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static Span<T> CreateSpan<T>(ref T reference, int length) => MemoryMarshal.CreateSpan(ref reference, length);
#endif

        public static object? BytesToStruct(byte[] buffer, Type type)
        {
            IntPtr ptr = Marshal.UnsafeAddrOfPinnedArrayElement(buffer, 0);
            return Marshal.PtrToStructure(ptr, type);
        }

        /// <summary>
        /// bytes转结构体引用
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="buffer"></param>
        /// <returns></returns>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static ref T BytesAsStructure<T>(Span<byte> buffer) where T : struct
        {
            return ref AsRef<T>(buffer);
        }

        /// <summary>
        /// 结构体转bytes，对bytes的改动会影响原数据
        /// </summary>
        /// <typeparam name="T"></typeparam>
        /// <param name="value"></param>
        /// <returns></returns>
        public static Span<byte> StructureAsBytes<T>(ref T value) where T : struct
        {
            var span = CreateSpan(ref value, 1);
            return MemoryMarshal.AsBytes(span);
        }

        /// <summary>
        /// 结构体引用转byte[]
        /// </summary>
        public static byte[] StructureRefToBytes<T>(ref T value, byte[]? buffer = null) where T : struct
        {
            int size = Unsafe.SizeOf<T>();
            if (buffer == null || buffer.Length < size)
            {
                buffer = new byte[size];
            }
#if NET8_0_OR_GREATER
            MemoryMarshal.Write(buffer, in value);
#else
            MemoryMarshal.Write(buffer, ref value);
#endif
            return buffer;
        }

        /// <summary>
        /// 结构体转byte[]
        /// </summary>
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static byte[] StructureToBytes<T>(T value, byte[]? buffer = null) where T : struct
        {
            return StructureRefToBytes(ref value, buffer);
        }

        /// <summary>
        /// 结构体转byte[]
        /// </summary>
        public static byte[] StructureToBytes(object value, byte[]? buffer = null)
        {
            int size = Marshal.SizeOf(value);
            if (buffer == null || buffer.Length < size)
            {
                buffer = new byte[size];
            }
            IntPtr ptr = Marshal.UnsafeAddrOfPinnedArrayElement(buffer, 0);
            Marshal.StructureToPtr(value, ptr, false);
            return buffer;
        }
    }
}
