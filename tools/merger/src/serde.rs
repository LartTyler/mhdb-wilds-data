use serde::{Serialize, Serializer};
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

pub fn is_map_empty<K, V>(value: &HashMap<K, V>) -> bool
where
    K: Ord + Serialize,
    V: Serialize,
{
    value.is_empty()
}
