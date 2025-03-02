using System.Numerics;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace RszTool.via
{
    public static class ValueTypeUtils
    {
        public static void CheckIndex(int index, int length)
        {
            if (index < 0 || index >= length) throw new IndexOutOfRangeException($"Index must be 0..{length-1}, got {index}");
        }

        public static IndexOutOfRangeException IndexError(int index, int length)
        {
            return new IndexOutOfRangeException($"Index must be 0..{length-1}, got {index}");
        }
    }


    public struct Int2
    {
        public int x;
        public int y;

        public readonly override string ToString()
        {
            return $"<{x}, {y}>";
        }

        public int this[int index]
        {
            readonly get
            {
                ValueTypeUtils.CheckIndex(index, 2);
                return index == 0 ? x : y;
            }
            set
            {
                ValueTypeUtils.CheckIndex(index, 2);
                if (index == 0) x = value;
                else y = value;
            }
        }
    }


    public struct Int3
    {
        public int x;
        public int y;
        public int z;

        public readonly override string ToString()
        {
            return $"<{x}, {y}, {z}>";
        }

        public int this[int index]
        {
            readonly get => index switch {
                0 => x,
                1 => y,
                2 => z,
                _ => throw ValueTypeUtils.IndexError(index, 3)
            };
            set
            {
                switch (index)
                {
                    case 0: x = value; break;
                    case 1: y = value; break;
                    case 2: z = value; break;
                    default: throw ValueTypeUtils.IndexError(index, 3);
                }
            }
        }
    }


    public struct Int4
    {
        public int x;
        public int y;
        public int z;
        public int w;

        public readonly override string ToString()
        {
            return $"<{x}, {y}, {z}, {w}>";
        }

        public int this[int index]
        {
            readonly get => index switch {
                0 => x,
                1 => y,
                2 => z,
                3 => w,
                _ => throw ValueTypeUtils.IndexError(index, 4)
            };
            set
            {
                switch (index)
                {
                    case 0: x = value; break;
                    case 1: y = value; break;
                    case 2: z = value; break;
                    case 3: w = value; break;
                    default: throw ValueTypeUtils.IndexError(index, 4);
                }
            }
        }
    }


    public struct Uint2
    {
        public uint x;
        public uint y;

        public readonly override string ToString()
        {
            return $"<{x}, {y}>";
        }

        public uint this[int index]
        {
            readonly get
            {
                ValueTypeUtils.CheckIndex(index, 2);
                return index == 0 ? x : y;
            }
            set
            {
                ValueTypeUtils.CheckIndex(index, 2);
                if (index == 0) x = value;
                else y = value;
            }
        }
    }


    public struct Uint3
    {
        public uint x;
        public uint y;
        public uint z;

        public readonly override string ToString()
        {
            return $"<{x}, {y}, {z}>";
        }

        public uint this[int index]
        {
            readonly get => index switch {
                0 => x,
                1 => y,
                2 => z,
                _ => throw ValueTypeUtils.IndexError(index, 3)
            };
            set
            {
                switch (index)
                {
                    case 0: x = value; break;
                    case 1: y = value; break;
                    case 2: z = value; break;
                    default: throw ValueTypeUtils.IndexError(index, 3);
                }
            }
        }
    }


    public struct Uint4
    {
        public uint x;
        public uint y;
        public uint z;
        public uint w;

        public readonly override string ToString()
        {
            return $"<{x}, {y}, {z}, {w}>";
        }

        public uint this[int index]
        {
            readonly get => index switch {
                0 => x,
                1 => y,
                2 => z,
                3 => w,
                _ => throw ValueTypeUtils.IndexError(index, 4)
            };
            set
            {
                switch (index)
                {
                    case 0: x = value; break;
                    case 1: y = value; break;
                    case 2: z = value; break;
                    case 3: w = value; break;
                    default: throw ValueTypeUtils.IndexError(index, 4);
                }
            }
        }
    }


    // Size=8
    public struct Range
    {
        public float r;
        public float s;

        public readonly override string ToString()
        {
            return $"Range({r}, {s})";
        }

        public static explicit operator Range(Vector2 vector)
        {
            return new Range { r = vector.X, s = vector.Y };
        }

        public static explicit operator Vector2(Range range)
        {
            return new Vector2(range.r, range.s);
        }

        public float this[int index]
        {
            readonly get
            {
                ValueTypeUtils.CheckIndex(index, 2);
                return index == 0 ? r : s;
            }
            set
            {
                ValueTypeUtils.CheckIndex(index, 2);
                if (index == 0) r = value;
                else s = value;
            }
        }
    }


    // Size=8
    public struct RangeI
    {
        public int r;
        public int s;

        public readonly override string ToString()
        {
            return $"Range({r}, {s})";
        }

        public int this[int index]
        {
            readonly get
            {
                ValueTypeUtils.CheckIndex(index, 2);
                return index == 0 ? r : s;
            }
            set
            {
                ValueTypeUtils.CheckIndex(index, 2);
                if (index == 0) r = value;
                else s = value;
            }
        }
    }


    // Size=4
    public struct Color
    {
        public uint rgba;

        public readonly string Hex()
        {
            return rgba.ToString("X8");
        }

        public readonly override string ToString()
        {
            return Hex();
        }

        /// <summary>
        /// Parse a hex string
        /// </summary>
        /// <param name="hex"></param>
        /// <returns></returns>
        public static Color Parse(string hex)
        {
            return new Color { rgba = uint.Parse(hex, System.Globalization.NumberStyles.HexNumber) };
        }
    }


    // Size=64
    public struct mat4 {
        public float m00;
        public float m01;
        public float m02;
        public float m03;
        public float m10;
        public float m11;
        public float m12;
        public float m13;
        public float m20;
        public float m21;
        public float m22;
        public float m23;
        public float m30;
        public float m31;
        public float m32;
        public float m33;

        public float this[int index]
        {
            get
            {
                ValueTypeUtils.CheckIndex(index, 16);
                ref float ptr = ref m00;
                return Unsafe.Add(ref ptr, index);
            }
            set
            {
                ValueTypeUtils.CheckIndex(index, 16);
                ref float ptr = ref m00;
                Unsafe.Add(ref ptr, index) = value;
            }
        }

        /* public float this[int row, int col]
        {
            get => this[row * 4 + col];
            set => this[row * 4 + col] = value;
        } */
    }


    [StructLayout(LayoutKind.Explicit, Size = 80)]
    public struct OBB
    {
        [FieldOffset(0)]
        private mat4 coord;
        [FieldOffset(64)]
        private Vector3 extent;

        public mat4 Coord { readonly get => coord; set => coord = value; }
        public Vector3 Extent { readonly get => extent; set => extent = value; }
    }


    // Size=16
    public struct Sphere
    {
        public Vector3 pos;
        public float r;

        public Vector3 Pos { readonly get => pos; set => pos = value; }
        public float R { readonly get => r; set => r = value; }

        public readonly override string ToString()
        {
            return $"Sphere({pos}, {r})";
        }
    }


    [StructLayout(LayoutKind.Explicit, Size = 32)]
    public struct AABB
    {
        [FieldOffset(0)]
        public Vector3 minpos;
        [FieldOffset(16)]
        public Vector3 maxpos;

        public Vector3 Minpos { readonly get => minpos; set => minpos = value; }
        public Vector3 Maxpos { readonly get => maxpos; set => maxpos = value; }
    }


    [StructLayout(LayoutKind.Explicit, Size = 48)]
    public struct Capsule
    {
        [FieldOffset(0)]
        public Vector3 p0;
        [FieldOffset(16)]
        public Vector3 p1;
        [FieldOffset(32)]
        public float r;

        public Vector3 P0 { readonly get => p0; set => p0 = value; }
        public Vector3 P1 { readonly get => p1; set => p1 = value; }
        public float R { readonly get => r; set => r = value; }
    }


    [StructLayout(LayoutKind.Explicit, Size = 48)]
    public struct Area
    {
        [FieldOffset(0)]
        public Vector2 p0;
        [FieldOffset(8)]
        public Vector2 p1;
        [FieldOffset(16)]
        public Vector2 p2;
        [FieldOffset(24)]
        public Vector2 p3;
        [FieldOffset(32)]
        public float height;
        [FieldOffset(36)]
        public float bottom;

        public Vector2 P0 { readonly get => p0; set => p0 = value; }
        public Vector2 P1 { readonly get => p1; set => p1 = value; }
        public Vector2 P2 { readonly get => p2; set => p2 = value; }
        public Vector2 P3 { readonly get => p3; set => p3 = value; }
        public float Height { readonly get => height; set => height = value; }
        public float Bottom { readonly get => bottom; set => bottom = value; }
    }


    public struct Postion
    {
        public double x;
        public double y;
        public double z;

        public double this[int index]
        {
            readonly get => index switch {
                0 => x,
                1 => y,
                2 => z,
                _ => throw ValueTypeUtils.IndexError(index, 3)
            };
            set
            {
                switch (index)
                {
                    case 0: x = value; break;
                    case 1: y = value; break;
                    case 2: z = value; break;
                    default: throw ValueTypeUtils.IndexError(index, 3);
                }
            }
        }
    }


    // Size=32
    public struct TaperedCapsule
    {
        private Vector4 vertexRadiusA;
        private Vector4 vertexRadiusB;

        public Vector4 VertexRadiusA { readonly get => vertexRadiusA; set => vertexRadiusA = value; }
        public Vector4 VertexRadiusB { readonly get => vertexRadiusB; set => vertexRadiusB = value; }
    }


    // Size=32
    public struct Cone
    {
        public Vector3 p0;
        public float r0;
        public Vector3 p1;
        public float r1;
    }


    [StructLayout(LayoutKind.Explicit, Size = 32)]
    public struct Line
    {
        [FieldOffset(0)]
        public Vector3 from;
        [FieldOffset(16)]
        public Vector3 dir;
    }


    [StructLayout(LayoutKind.Explicit, Size = 32)]
    public struct LineSegment
    {
        [FieldOffset(0)]
        public Vector3 start;
        [FieldOffset(16)]
        public Vector3 end;
    }


    // Size=16
    public struct Plane
    {
        public Vector3 normal;
        public float dist;
    }


    // Size=4
    public struct PlaneXZ
    {
        public float dist;
    }


    // Size=8
    public struct Point
    {
        public float x;
        public float y;
    }


    // Size=8
    public struct Size
    {
        public float w;
        public float h;

        public float this[int index]
        {
            readonly get
            {
                ValueTypeUtils.CheckIndex(index, 2);
                return index == 0 ? w : h;
            }
            set
            {
                ValueTypeUtils.CheckIndex(index, 2);
                if (index == 0) w = value;
                else h = value;
            }
        }
    }


    [StructLayout(LayoutKind.Explicit, Size = 32)]
    public struct Ray
    {
        [FieldOffset(0)]
        public Vector3 from;
        [FieldOffset(16)]
        public Vector3 dir;
    }


    // Size=16
    public struct RayY
    {
        public Vector3 from;
        public float dir;
    }


    [StructLayout(LayoutKind.Explicit, Size = 32)]
    public struct Segment
    {
        [FieldOffset(0)]
        public Vector4 from;
        [FieldOffset(16)]
        public Vector3 dir;
    }


    [StructLayout(LayoutKind.Explicit, Size = 48)]
    public struct Triangle
    {
        [FieldOffset(0)]
        public Vector3 p0;
        [FieldOffset(16)]
        public Vector3 p1;
        [FieldOffset(32)]
        public Vector3 p2;
    }


    [StructLayout(LayoutKind.Explicit, Size = 48)]
    public struct Cylinder
    {
        [FieldOffset(0)]
        public Vector3 p0;
        [FieldOffset(16)]
        public Vector3 p1;
        [FieldOffset(32)]
        public float r;
    }


    [StructLayout(LayoutKind.Explicit, Size = 32)]
    public struct Ellipsoid
    {
        [FieldOffset(0)]
        public Vector3 pos;
        [FieldOffset(16)]
        public Vector3 r;
    }


    // Size=32
    public struct Torus
    {
        public Vector3 pos;
        public float r;
        public Vector3 axis;
        public float cr;
    }


    // Size=16
    public struct Rect
    {
        public float l;
        public float t;
        public float r;
        public float b;

        public float this[int index]
        {
            get
            {
                ValueTypeUtils.CheckIndex(index, 4);
                ref float ptr = ref l;
                return Unsafe.Add(ref ptr, index);
            }
            set
            {
                ValueTypeUtils.CheckIndex(index, 4);
                ref float ptr = ref l;
                Unsafe.Add(ref ptr, index) = value;
            }
        }
    }


    // Size=32
    public struct Rect3D
    {
        public Vector3 normal;
        public float sizeW;
        public Vector3 center;
        public float sizeH;
    }


    // Size=96
    public struct Frustum
    {
        public Plane plane0;
        public Plane plane1;
        public Plane plane2;
        public Plane plane3;
        public Plane plane4;
        public Plane plane5;
    }


    // Size=16
    public struct KeyFrame
    {
        public float value;
        public uint time_type;
        public uint inNormal;
        public uint outNormal;
    }


#pragma warning disable CS8981
    public struct sfix
    {
        public int v;
    }
#pragma warning restore CS8981


    public struct Sfix2
    {
        public sfix x;
        public sfix y;
    }


    public struct Sfix3
    {
        public sfix x;
        public sfix y;
        public sfix z;
    }


    public struct Sfix4
    {
        public sfix x;
        public sfix y;
        public sfix z;
        public sfix w;
    }
}
