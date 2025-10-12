use serde::Serialize;
use zerocopy::{FromBytes, KnownLayout};

#[derive(Debug, KnownLayout, FromBytes, Clone, PartialOrd, PartialEq, Serialize)]
#[repr(C, packed)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    #[serde(skip)]
    _dummy: f32,
}
