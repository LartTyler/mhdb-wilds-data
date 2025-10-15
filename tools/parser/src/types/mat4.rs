use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use std::fmt::{Debug, Formatter};
use zerocopy::{FromBytes, KnownLayout};

#[derive(KnownLayout, FromBytes)]
#[repr(C, packed)]
pub struct Mat4([Mat4Element; 4]);

impl Debug for Mat4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Mat4(")?;
        self.0.fmt(f)?;
        f.write_str(")")
    }
}

impl Clone for Mat4 {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Serialize for Mat4 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;

        for element in &self.0 {
            seq.serialize_element(element)?;
        }

        seq.end()
    }
}

#[derive(Debug, FromBytes, KnownLayout, Clone, Serialize)]
#[repr(C, packed)]
pub struct Mat4Element {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}
