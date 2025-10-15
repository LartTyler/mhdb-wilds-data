use crate::check_magic;
use crate::layout::{FieldKind, LayoutField, LayoutMap};
use crate::rsz::{Error, Result};
use crate::types::mat4::Mat4;
use crate::types::vec3::Vec3;
use half::f16;
use serde::Serialize;
use std::any::type_name;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use strum_macros::EnumTryAs;
use uuid::Uuid;
use zerocopy::{FromBytes, KnownLayout};

/// The data content of an RSZ document.
#[derive(Debug)]
pub struct Content {
    /// A collection of all object instances contained in the document.
    pub items: Items,

    /// A collection of root objects contained in the document. If you need to iterate over the
    /// document, this is your starting point.
    pub objects: Items,
}

impl Content {
    pub const MAGIC: u32 = 0x5A5352;

    pub fn parse<T: RszStream>(data: &mut T, layout: &LayoutMap) -> Result<Self> {
        let header = data.next_byte_section::<Header>()?;
        check_magic!(Self::MAGIC, header.magic);

        let mut roots: Vec<ObjectReference> =
            Vec::with_capacity(header.object_count.unsigned_abs() as usize);

        for _ in 0..header.object_count {
            roots.push(data.next_byte_section::<ObjectReference>()?);
        }

        log::debug!("Found {} root object(s)", roots.len());

        let mut instances = Vec::with_capacity(header.instance_count.unsigned_abs() as usize);
        data.seek(header.instance_offset as usize)?;

        for _ in 0..header.instance_count {
            instances.push(data.next_byte_section::<Instance>()?);
        }

        log::debug!("Found {} instance(s)", instances.len());

        let mut items = Vec::with_capacity(instances.len());
        data.seek(header.data_offset as usize)?;

        for (index, type_info) in instances.iter().enumerate() {
            let Some(layout) = layout.get_layout(type_info.type_id) else {
                return Err(Error::UnknownLayoutId(type_info.type_id));
            };

            log::debug!("Found type {} at index = {index}", layout.name);

            items.push(Rc::new(Item {
                name: layout.name.to_owned(),
                fields: layout
                    .fields
                    .iter()
                    .map(|v| data.next_field(v, &items))
                    .collect::<Result<Vec<_>>>()?,
            }))
        }

        Ok(Self {
            objects: Items(
                roots
                    .into_iter()
                    .map(|root| items[root.index.unsigned_abs() as usize].clone())
                    .collect(),
            ),
            items: Items(items),
        })
    }
}

#[derive(Debug, FromBytes, KnownLayout)]
#[repr(C, packed)]
pub struct Header {
    pub magic: u32,
    pub version: u32,
    pub object_count: i32,
    pub instance_count: i32,
    pub userdata_count: i32,
    pub _reserved: u32,
    pub instance_offset: u64,
    pub data_offset: u64,
    pub userdata_offset: u64,
}

#[derive(Debug, FromBytes, KnownLayout)]
#[repr(C, packed)]
pub struct Instance {
    pub type_id: u32,
    pub crc: u32,
}

#[derive(Debug, FromBytes, KnownLayout)]
#[repr(C, packed)]
pub struct ObjectReference {
    pub index: i32,
}

#[derive(Debug)]
pub struct Items(Vec<Rc<Item>>);

impl Deref for Items {
    type Target = Vec<Rc<Item>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Items {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub fields: Vec<Field>,
}

impl Item {
    pub fn extract_field(&self, index: usize) -> Option<Item> {
        self.fields.get(index).map(|field| Item {
            name: self.name.clone(),
            fields: vec![field.clone()],
        })
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Serialize, EnumTryAs, Clone)]
#[serde(untagged)]
pub enum Value {
    Array(Values),
    Boolean(bool),
    F16(f16),
    F32(f32),
    F64(f64),
    Guid(Uuid),
    Object(Rc<Item>),
    S8(i8),
    S16(i16),
    S32(i32),
    S64(i64),
    String(String),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Vec3(Vec3),
    Mat4(Mat4),
}

#[derive(Debug, Clone)]
pub struct Values(Vec<Value>);

impl Deref for Values {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// This trait should be implemented by any type that needs to act as a data source for an RSZ file.
pub trait RszStream {
    /// Returns a copy of the stream, offset such that the current position of this stream is
    /// treated as the first byte in the copy of the stream.
    fn as_relative(&self) -> Self;

    /// Returns the relative position within the stream. For streams in which
    /// [RszStream::as_relative] as never been called, this value should be equal to
    /// [RszStream::position_absolute].
    fn position(&self) -> usize;

    /// Returns the absolute byte position within the stream, regardless of whether
    /// [RszStream::as_relative] has ever been called. This should always be the true position
    /// within the underlying stream.
    fn position_absolute(&self) -> usize;

    /// Shifts the stream forward to the target position.
    fn seek(&mut self, pos: usize) -> Result<()>;

    /// Skips `len` bytes by moving the stream position forward by the given amount.
    fn skip(&mut self, len: usize) -> Result<()>;

    /// Attempts to parse the next series of bytes as defined in the provided [LayoutField].
    fn next_field(&mut self, layout: &LayoutField, loaded_objects: &[Rc<Item>]) -> Result<Field>;

    /// Aligns the stream to the given value.
    fn align(&mut self, align: usize) -> Result<()>;

    /// Attempts to parse the next series of bytes using
    /// [zerocopy](https://docs.rs/zerocopy/latest/zerocopy). Any type supported by `zerocopy`
    /// should be parsable using this method.
    fn next_byte_section<T>(&mut self) -> Result<T>
    where
        T: FromBytes + KnownLayout;
}

/// An RSZ data stream over an in-memory slice of bytes.
pub struct SliceStream<'a> {
    data: &'a [u8],

    /// The current position in the stream.
    position: usize,

    /// A sum of any offsets applied to this stream using the [RszStream::as_relative()] function.
    ///
    /// This property is exclusively used for debugging and logging.
    offset: usize,
}

impl SliceStream<'_> {
    fn eof(&self) -> bool {
        self.position > self.data.len()
    }
}

impl<'a> From<&'a [u8]> for SliceStream<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self {
            data: value,
            position: 0,
            offset: 0,
        }
    }
}

impl<'a> From<(&'a [u8], usize)> for SliceStream<'a> {
    fn from((data, offset): (&'a [u8], usize)) -> Self {
        Self {
            data,
            offset,
            position: 0,
        }
    }
}

impl RszStream for SliceStream<'_> {
    fn as_relative(&self) -> Self {
        log::debug!(
            "Creating relative stream, base_position = 0x{:X} (0x{:X})",
            self.position,
            self.position_absolute(),
        );

        Self::from((&self.data[self.position..], self.position_absolute()))
    }

    fn position(&self) -> usize {
        self.position
    }

    fn position_absolute(&self) -> usize {
        self.position + self.offset
    }

    fn seek(&mut self, pos: usize) -> Result<()> {
        self.position = pos;

        log::trace!(
            "Seeking to 0x{pos:X} (abs = 0x{:X})",
            self.position_absolute()
        );

        if self.eof() {
            Err(Error::UnexpectedEof(self.position, self.data.len()))
        } else {
            Ok(())
        }
    }

    fn skip(&mut self, len: usize) -> Result<()> {
        log::trace!("Skipping {len} bytes");
        self.seek(self.position + len)
    }

    fn next_field(&mut self, layout: &LayoutField, loaded_objects: &[Rc<Item>]) -> Result<Field> {
        layout.parse(self, loaded_objects)
    }

    fn align(&mut self, align: usize) -> Result<()> {
        log::trace!(
            "Aligning to {align}, start = 0x{:X} (0x{:X})",
            self.position,
            self.position_absolute()
        );

        let delta = self.position % align;

        // We're already aligned, nothing to do.
        if delta == 0 {
            log::trace!(">> Already aligned.");
            return Ok(());
        }

        self.position += align - (self.position % align);

        log::trace!(
            ">> end = 0x{:X} (0x{:X})",
            self.position,
            self.position_absolute()
        );

        if self.eof() {
            Err(Error::UnexpectedEof(self.position, self.data.len()))
        } else {
            Ok(())
        }
    }

    fn next_byte_section<T>(&mut self) -> Result<T>
    where
        T: FromBytes + KnownLayout,
    {
        log::trace!(
            "Reading {} @ 0x{:X} (0x{:X})",
            type_name::<T>(),
            self.position,
            self.position_absolute()
        );

        let result = T::read_from_prefix(&self.data[self.position..])
            .map(|(v, _)| v)
            .map_err(|_| {
                Error::InvalidSection(format!("{}, len = {}", type_name::<T>(), size_of::<T>()))
            })?;

        self.skip(size_of::<T>())?;

        Ok(result)
    }
}

/// This trait should be implemented by any type that should be parsable via
/// [RszStream::next_field()].
trait ParseField {
    /// The main entrypoint for parsing a field using this trait. Implementations should handle
    /// arrays and any initial alignments in this method.
    fn parse<T: RszStream>(&self, data: &mut T, loaded_objects: &[Rc<Item>]) -> Result<Field>;

    /// Parses the actual field represented by the type implementing this trait. Implementations
    /// should only handle parsing the field's underlying type, and should rely on
    /// [ParseField::parse()] for any initial alignment or special case handling (such as arrays).
    fn parse_value<T>(&self, data: &mut T, loaded_objects: &[Rc<Item>]) -> Result<Value>
    where
        T: RszStream;
}

impl ParseField for LayoutField<'_> {
    fn parse<T: RszStream>(&self, data: &mut T, loaded_objects: &[Rc<Item>]) -> Result<Field> {
        log::debug!(
            "Parsing field \"{}\"; kind = {:?}, original_name = {}, is_array = {}",
            self.name,
            self.kind,
            self.original_type_name,
            self.is_array
        );

        let value = if self.is_array {
            data.align(4)?;

            let element_count = data.next_byte_section::<i32>()?.unsigned_abs();
            log::debug!(">> Field is array, len = {element_count}");

            let mut elements = Vec::with_capacity(element_count as usize);

            if element_count > 0 {
                data.align(self.align)?;

                for _ in 0..element_count {
                    elements.push(self.parse_value(data, loaded_objects)?);
                }
            }

            Value::Array(Values(elements))
        } else {
            data.align(self.align)?;
            self.parse_value(data, loaded_objects)?
        };

        log::debug!("value = {value:?}");

        Ok(Field {
            name: self.name.to_owned(),
            value,
        })
    }

    fn parse_value<T: RszStream>(
        &self,
        data: &mut T,
        loaded_objects: &[Rc<Item>],
    ) -> Result<Value> {
        match self.kind {
            FieldKind::Boolean => {
                let byte = data.next_byte_section::<u8>()?;
                Ok(Value::Boolean(byte != 0))
            }
            FieldKind::F16 => data.next_byte_section::<f16>().map(Value::F16),
            FieldKind::F32 => data.next_byte_section::<f32>().map(Value::F32),
            FieldKind::F64 => data.next_byte_section::<f64>().map(Value::F64),
            FieldKind::Guid => {
                let bytes = data.next_byte_section::<[u8; 16]>()?;
                Ok(Value::Guid(Uuid::from_bytes_le(bytes)))
            }
            FieldKind::InstanceRef => {
                let target_index = data.next_byte_section::<i32>()?;
                log::debug!(">> Reading instance ref, target_index = {target_index}");

                match loaded_objects.get(target_index.unsigned_abs() as usize) {
                    Some(target) => Ok(Value::Object(target.clone())),
                    None => Err(Error::ObjectNotFound(target_index)),
                }
            }
            FieldKind::S8 => data.next_byte_section::<i8>().map(Value::S8),
            FieldKind::S16 => data.next_byte_section::<i16>().map(Value::S16),
            FieldKind::S32 => data.next_byte_section::<i32>().map(Value::S32),
            FieldKind::S64 => data.next_byte_section::<i64>().map(Value::S64),
            FieldKind::String => {
                let length = data.next_byte_section::<i32>()? * 2;
                log::debug!(">> Reading string, len = {length}");

                let mut value = String::with_capacity(length.unsigned_abs() as usize);

                for _ in 0..length {
                    let c: char = data.next_byte_section::<u8>()?.into();

                    if c == '\0' {
                        continue;
                    }

                    value.push(c);
                }

                Ok(Value::String(value))
            }
            FieldKind::U8 => data.next_byte_section::<u8>().map(Value::U8),
            FieldKind::U16 => data.next_byte_section::<u16>().map(Value::U16),
            FieldKind::U32 => data.next_byte_section::<u32>().map(Value::U32),
            FieldKind::U64 => data.next_byte_section::<u64>().map(Value::U64),
            FieldKind::Vec3 => data.next_byte_section::<Vec3>().map(Value::Vec3),
            FieldKind::Mat4 => data.next_byte_section::<Mat4>().map(Value::Mat4),
            other => Err(Error::UnsupportedFieldKind(other)),
        }
    }
}
