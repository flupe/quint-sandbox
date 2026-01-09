// This module provides helpers for ITF JSON serialization

use serde::{Serializer, ser::SerializeMap};
use std::collections::HashMap;
use std::marker::PhantomData;
use serde_with::{SerializeAs, ser::SerializeAsWrap};
use num_bigint::BigInt;

pub struct ItfBigInt;
pub struct ItfMap<KA, VA>(PhantomData<(KA, VA)>);

impl SerializeAs<BigInt> for ItfBigInt {
    fn serialize_as<S>(val: &BigInt, s: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        let mut map = s.serialize_map(Some(1))?;
        map.serialize_entry("#bigint", &val.to_string())?;
        map.end()
    }
}

impl<K, V, KA, VA> SerializeAs<HashMap<K, V>> for ItfMap<KA, VA>
    where
        KA: SerializeAs<K>,
        VA: SerializeAs<V>, {
    fn serialize_as<S>(map: &HashMap<K, V>, s: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        let pairs: Vec<(SerializeAsWrap<K, KA>, SerializeAsWrap<V, VA>)> =
                map.iter()
                   .map(|(k, v)| (SerializeAsWrap::new(k), SerializeAsWrap::new(v)))
                   .collect();

        let mut outer = s.serialize_map(Some(1))?;
        outer.serialize_entry("#map", &pairs)?;
        outer.end()
    }
}
