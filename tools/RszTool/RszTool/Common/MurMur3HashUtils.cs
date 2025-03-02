using System.Runtime.InteropServices;
using System.Text;

namespace RszTool.Common
{
    public class MurMur3HashUtils
    {
        private static uint MurMur3Hash(ReadOnlySpan<byte> bytes)
        {
            const uint c1 = 0xcc9e2d51;
            const uint c2 = 0x1b873593;
            const uint seed = 0xffffffff;

            uint h1 = seed;
            uint k1;

            for (int i = 0; i < bytes.Length; i += 4)
            {
                int chunkLength = Math.Min(4, bytes.Length - i);
                k1 = chunkLength switch
                {
                    4 => (uint)(bytes[i] | bytes[i + 1] << 8 | bytes[i + 2] << 16 | bytes[i + 3] << 24),
                    3 => (uint)(bytes[i] | bytes[i + 1] << 8 | bytes[i + 2] << 16),
                    2 => (uint)(bytes[i] | bytes[i + 1] << 8),
                    1 => bytes[i],
                    _ => 0
                };
                k1 *= c1;
                k1 = Rotl32(k1, 15);
                k1 *= c2;
                h1 ^= k1;
                if (chunkLength == 4)
                {
                    h1 = Rotl32(h1, 13);
                    h1 = h1 * 5 + 0xe6546b64;
                }
            }

            h1 ^= (uint)bytes.Length;
            h1 = Fmix(h1);

            return h1;
        }

        private static uint Rotl32(uint x, byte r)
        {
            return (x << r) | (x >> (32 - r));
        }

        private static uint Fmix(uint h)
        {
            h ^= h >> 16;
            h *= 0x85ebca6b;
            h ^= h >> 13;
            h *= 0xc2b2ae35;
            h ^= h >> 16;
            return h;
        }

        public static uint GetHash(string text)
        {
            return MurMur3Hash(MemoryMarshal.AsBytes(text.AsSpan()));
        }

        public static uint GetAsciiHash(string text)
        {
            return MurMur3Hash(Encoding.ASCII.GetBytes(text));
        }
    }
}
