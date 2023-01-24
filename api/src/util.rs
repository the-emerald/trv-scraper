use serde::{Deserialize, Deserializer};

// h/t: meetmangukiya (https://gist.github.com/meetmangukiya/40cad17bcb7d3196d33b072a3500fac7)
pub mod u256_fromstr_radix_10 {
    use ethers_core::types::U256;
    use serde::{de::Visitor, Deserializer, Serializer};
    use std::fmt;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Helper;

        impl<'de> Visitor<'de> for Helper {
            type Value = U256;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                U256::from_dec_str(value).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(Helper)
    }

    pub fn serialize<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&value)
    }
}

// https://stackoverflow.com/a/69458453
pub fn object_empty_or_error_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    for<'a> T: Deserialize<'a>,
{
    #[derive(Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    struct Empty {}

    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    enum Aux<T> {
        T(T),
        Empty(Empty),
        Null,
    }

    match Deserialize::deserialize(deserializer) {
        Ok(d) => match d {
            Aux::T(t) => Ok(Some(t)),
            Aux::Empty(_) | Aux::Null => Ok(None),
        },
        Err(_) => Ok(None),
    }
}
