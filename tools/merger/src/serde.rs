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

pub fn is_map_empty<K, V>(value: &HashMap<K, V>) -> bool
where
    K: Ord + Serialize,
    V: Serialize,
{
    value.is_empty()
}
