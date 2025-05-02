use serde::{Deserialize, Deserializer};

pub fn deserialize_untagged_enum_case_insensitive<'de, T, D>(
    deserializer: D,
) -> Result<T, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;
    T::deserialize(Value::String(
        String::deserialize(deserializer)?.to_lowercase(),
    ))
    .map_err(Error::custom)
}
