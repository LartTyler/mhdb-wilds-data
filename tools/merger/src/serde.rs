use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

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

pub fn ordered_set<S, V>(value: &HashSet<V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    V: Ord + Serialize,
{
    value.iter().collect::<BTreeSet<_>>().serialize(serializer)
}

pub fn is_default<T>(value: &T) -> bool
where
    T: Default + PartialEq,
{
    *value == T::default()
}
