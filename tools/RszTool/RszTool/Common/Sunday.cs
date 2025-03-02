namespace RszTool.Common
{
    /// <summary>
    /// Sunday字符串搜索算法
    /// </summary>
    public class Sunday
    {
        public const byte WILDCARD = (byte)'*';

        /// <summary>查找表</summary>
        private readonly int[] bc = new int[256];
        /// <summary>模板串</summary>
        private byte[] pattern = [];
        /// <summary>是否支持通配符(*)</summary>
        private bool wildcard = false;

        public Sunday()
        {
            ((Span<int>)bc).Fill(-1);
        }

        public Sunday(byte[] pattern, bool wildcard = false)
        {
            Update(pattern, wildcard);
        }

        // 更新参数
        public void Update(byte[] pattern, bool wildcard = false)
        {
            this.pattern = pattern;
            this.wildcard = wildcard;
            // 构造表，记录模式串中每种字符最后出现的位置
            ((Span<int>)bc).Fill(-1);
            for (int i = 0; i < pattern.Length; i++)
            {
                bc[pattern[i]] = i;
            }
        }

        /// <summary>
        /// 搜索算法
        /// </summary>
        /// <param name="target">目标串</param>
        /// <param name="ordinal">找第几个匹配的结果</param>
        /// <returns></returns>
        public int Search(Span<byte> target, int start = 0, int end = -1, int ordinal = 1)
        {
            if (end == -1) end = target.Length;
            if (end > target.Length)
            {
                throw new ArgumentOutOfRangeException($"{nameof(end)} must <= {nameof(target)}.Length");
            }
            if (end < pattern.Length)
            {
                throw new ArgumentException($"{nameof(end)} must >= {nameof(pattern)}.Length");
            }
            if (start < 0 || start >= end)
            {
                throw new ArgumentOutOfRangeException(nameof(start), $"{nameof(start)} must >= 0 and < {nameof(end)}");
            }
            if (ordinal < 1)
            {
                throw new ArgumentOutOfRangeException(nameof(ordinal), $"{nameof(ordinal)} must >= 1");
            }

            if (pattern.Length <= 0 || end <= 0)
                return -1;

            int i = start, j = 0, k;
            int m = pattern.Length;

            while (i < end)
            {
                if (target[i] == pattern[j] || wildcard && pattern[j] == WILDCARD)
                {
                    if (j == pattern.Length - 1)
                    {
                        if (ordinal == 1)
                        {
                            return i - j;
                        }
                        --ordinal;
                        j = 0;
                    }
                    else
                    {
                        ++j;
                    }
                    ++i;
                }
                else
                {
                    // 下一个字符在模式串中的位置
                    k = bc[target[m]];
                    if (wildcard)
                    {
                        k = Math.Max(k, bc[WILDCARD]);
                    }
                    i = m - k;
                    j = 0;
                    m = i + pattern.Length;
                    if (m >= end)
                    {
                        break;
                    }
                }
            }

            return -1;
        }
    }


    public struct SearchParam
    {
        public long start = 0;
        public long end = -1;
        public int ordinal = 1;
        public bool wildcard = false;

        public SearchParam() {}
    }
}
