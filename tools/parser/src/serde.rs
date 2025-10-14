use crate::rsz::content::{Item, Items, Value, Values};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::collections::HashMap;

pub(crate) fn deserialize_hex_map<'de, D, V>(deserializer: D) -> Result<HashMap<u32, V>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    let map: HashMap<Cow<'de, str>, V> = HashMap::deserialize(deserializer)?;

    map.into_iter()
        .map(|(k, v)| {
            u32::from_str_radix(&k, 16)
                .map(|key| (key, v))
                .map_err(serde::de::Error::custom)
        })
        .collect()
}

impl Serialize for Items {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.len() == 1 {
            self[0].serialize(serializer)
        } else {
            (*self).serialize(serializer)
        }
    }
}

impl Serialize for Item {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if is_transparent_collection(self) {
            let Value::Array(ref values) = self.fields[0].value else {
                panic!("Transparent collections must always be an array.");
            };

            values.serialize(serializer)
        } else if is_transparent_object(self) {
            let Value::Object(ref item) = self.fields[0].value else {
                panic!("Transparent objects must always be an object.")
            };

            item.fields[0].value.serialize(serializer)
        } else {
            let mut map = serializer.serialize_map(Some(self.fields.len()))?;

            for field in self.fields.iter() {
                if is_transparent_value(&field.value) {
                    let Value::Object(ref value) = field.value else {
                        panic!(
                            "Transparent fields must always be an object with exactly one field."
                        );
                    };

                    map.serialize_entry(&field.name, &value.fields[0].value)?;
                } else {
                    map.serialize_entry(&field.name, &field.value)?;
                }
            }

            map.end()
        }
    }
}

fn is_transparent_collection(item: &Item) -> bool {
    item.fields.len() == 1 && matches!(item.fields[0].value, Value::Array(_))
}

fn is_transparent_object(item: &Item) -> bool {
    item.fields.len() == 1
        && matches!(item.fields[0].value, Value::Object(ref inner) if is_transparent_collection(inner))
}

fn is_transparent_value(value: &Value) -> bool {
    matches!(
        value,
        Value::Object(v) if v.fields.len() == 1
    )
}

impl Serialize for Values {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        let is_transparent = is_transparent_values_collection(self);

        for value in self.iter() {
            if is_transparent {
                let Value::Object(inner) = value else {
                    panic!(
                        "Members of a transparent value collection must be objects with exactly one field."
                    );
                };

                seq.serialize_element(&inner.fields[0].value)?;
            } else {
                seq.serialize_element(value)?;
            }
        }

        seq.end()
    }
}

fn is_transparent_values_collection(values: &Values) -> bool {
    !values.is_empty() && is_transparent_value(&values[0])
}
