use crate::serde::deserialize_hex_map;
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub type LayoutTypeId = u32;
type LayoutMapInner<'a> = HashMap<LayoutTypeId, Layout<'a>>;

/// Layout maps are usually loaded from rsz<game>.json files, and contain a mapping of type IDs to
/// type definitions. These are required in order to properly understand the contents of an RSZ
/// file.
#[derive(Debug, Deserialize)]
pub struct LayoutMap<'a>(
    #[serde(borrow, deserialize_with = "deserialize_hex_map")] LayoutMapInner<'a>,
);

impl<'a> LayoutMap<'a> {
    /// Parses the contents of a layout file. You must load the contents yourself, this method
    /// _only_ handles parsing.
    pub fn parse(input: &'a str) -> Result<Self> {
        serde_json::from_str(input).map_err(From::from)
    }

    /// Retrieves a type layout by ID.
    pub fn get_layout(&self, id: LayoutTypeId) -> Option<&Layout<'a>> {
        self.0.get(&id)
    }
}

/// Each layout contained in a layout file defines the original name of the field, the CRC value,
/// and an array of fields contained in the property (which may be empty).
#[derive(Debug, Deserialize)]
pub struct Layout<'a> {
    /// If the property defined by this layout contains any fields, this will be a collection of
    /// layout information for those fields.
    pub fields: Vec<LayoutField<'a>>,

    /// The original property name as defined within the engine.
    pub name: &'a str,
    pub crc: &'a str,
}

/// Defines a field owned by a property layout.
#[derive(Debug, Deserialize)]
pub struct LayoutField<'a> {
    /// The field's alignment. When parsing, the input stream _must_ be aligned by this value in
    /// order to find the correct bytes of the field.
    pub align: usize,

    /// The number of bytes the field occupies in the file.
    ///
    /// This is mostly redundant within the scope of this library, since the types we read fields
    /// into are themselves the expected size already.
    pub size: usize,

    /// A flag indicating whether this field is an array. If it is, the next 4 bytes are a
    /// signed 32-bit integer that indicate the length of the array, followed by the elements (whose
    /// type is defined by the field's [LayoutField::kind] property.
    #[serde(rename = "array")]
    pub is_array: bool,

    /// The name of the field.
    pub name: &'a str,

    /// Indicates that the field is a native type. In the context of this library, this field is not
    /// needed and is only included for completeness.
    #[serde(rename = "native")]
    pub is_native: bool,

    /// The original in-engine type name of the field.
    #[serde(rename = "original_type")]
    pub original_type_name: &'a str,

    /// The type contained in the field.
    #[serde(rename = "type")]
    pub kind: FieldKind,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("deserialize failed: {0}")]
    Deserialize(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub enum FieldKind {
    #[serde(rename = "Bool")]
    Boolean,
    F16,
    F32,
    F64,
    Guid,
    #[serde(rename = "Object")]
    InstanceRef,
    S8,
    S16,
    S32,
    S64,
    String,
    U8,
    U16,
    U32,
    U64,

    // --- All items below this line are not yet supported ---
    AABB,
    Capsule,
    Color,
    Cylinder,
    Data,
    DateTime,
    Float2,
    Float3,
    Float4,
    Frustum,
    GameObjectRef,
    Half4,
    Int2,
    Int3,
    Int4,
    KeyFrame,
    Mat4,
    OBB,
    Plane,
    Point,
    Position,
    Quaternion,
    Range,
    RangeI,
    Rect,
    Resource,
    RuntimeType,
    Size,
    Sphere,
    Struct,
    Uint2,
    UserData,
    Vec2,
    Vec3,
    Vec4,
}
