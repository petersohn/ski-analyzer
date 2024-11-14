use crate::utils::result::extract_option_result;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

pub fn serialize<S: Serializer>(
    time: &Option<OffsetDateTime>,
    s: S,
) -> Result<S::Ok, S::Error> {
    let time_str = extract_option_result(time.map(|t| {
        t.format(&Iso8601::DEFAULT)
            .map_err(|e| serde::ser::Error::custom(e))
    }))?;
    time_str.serialize(s)
}

pub fn deserialize<'de, D>(
    deserializer: D,
) -> Result<Option<OffsetDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    extract_option_result(s.map(|ss| {
        OffsetDateTime::parse(&ss, &Iso8601::DEFAULT)
            .map_err(|e| serde::de::Error::custom(e))
    }))
}
