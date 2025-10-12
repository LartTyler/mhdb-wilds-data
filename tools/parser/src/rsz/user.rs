use crate::check_magic;
use crate::layout::LayoutMap;
use crate::rsz::content::{Content, RszStream, SliceStream};
use crate::rsz::Result;
use std::fs;
use std::path::Path;
use zerocopy::{FromBytes, KnownLayout};

#[derive(Debug)]
pub struct User {
    pub content: Content,
}

impl User {
    pub const MAGIC: u32 = 0x525355;

    pub fn load(path: &Path, layout: &LayoutMap) -> Result<Self> {
        log::info!("Loading USER document, path = {path:?}");

        let data = fs::read(path)?;
        Self::parse(&mut SliceStream::from(data.as_slice()), layout)
    }

    pub fn parse<T: RszStream>(data: &mut T, layout: &LayoutMap) -> Result<Self> {
        let header = data.next_byte_section::<Header>()?;
        check_magic!(Self::MAGIC, header.magic);

        let offset = header.data_offset;
        log::trace!("Seeking to RSZ document, offset = 0x{offset:X}");
        data.seek(offset as usize)?;

        Ok(Self {
            content: Content::parse(&mut data.as_relative(), &layout)?,
        })
    }
}

#[derive(Debug, FromBytes, KnownLayout)]
#[repr(C, packed)]
pub struct Header {
    magic: u32,
    resource_count: i32,
    userdata_count: i32,
    info_count: i32,
    resource_offset: u64,
    userdata_offset: u64,
    data_offset: u64,
}
