use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};

pub fn ordered_map<S, K, V>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Ord + Serialize,
    V: Serialize,
{
    value
        .iter()
        .collect::<BTreeMap<_, _>>()
        .serialize(serializer)
}

pub fn optional_ordered_map<S, K, V>(
    value: &Option<HashMap<K, V>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Ord + Serialize,
    V: Serialize,
{
    let Some(value) = value else {
        return serializer.serialize_none();
    };

    let value: BTreeMap<_, _> = value.iter().collect();
    serializer.serialize_some(&value)
}

pub fn is_default<T>(value: &T) -> bool
where
    T: Default + PartialEq,
{
    *value == T::default()
}

pub fn zero_is_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Zero + Deserialize<'de>,
{
    let value = T::deserialize(deserializer)?;

    if value.is_zero() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

pub fn zero_or_negative_is_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Zero + Negative + Deserialize<'de>,
{
    let value = T::deserialize(deserializer)?;

    if value.is_zero() || value.is_negative() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

pub trait Zero {
    fn is_zero(&self) -> bool;
}

macro_rules! impl_zero {
    (
        $( $ty:ident $(,)? )+
    ) => {
        $(
            impl Zero for $ty {
                fn is_zero(&self) -> bool { *self == 0 }
            }
        )*
    };
}

impl_zero! {
    u8,
    i8,
    isize,
}

pub trait Negative {
    fn is_negative(&self) -> bool;
}

macro_rules! impl_negative {
    (
        $( $ty:ident $(,)? )+
    ) => {
        $(
            impl Negative for $ty {
                fn is_negative(&self) -> bool { *self < 0 }
            }
        )*
    };
}

impl_negative! {
    i8,
}
