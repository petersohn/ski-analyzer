use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

use serde::{Deserialize, Deserializer, Serializer};

use std::result::Result;

pub fn serialize<S>(
    time: &OffsetDateTime,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = time
        .format(&Iso8601::DEFAULT)
        .map_err(|e| serde::ser::Error::custom(e))?;
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    OffsetDateTime::parse(&s, &Iso8601::DEFAULT)
        .map_err(|e| serde::de::Error::custom(e))
}
