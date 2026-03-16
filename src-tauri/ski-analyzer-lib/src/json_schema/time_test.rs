use crate::json_schema::time::OffsetDateTimeDef;
use crate::utils::time_ser;
use jsonschema::Validator;
use schemars::schema_for;
use serde::Serialize;
use time::OffsetDateTime;

#[derive(Serialize)]
struct TimeWrapper(#[serde(with = "time_ser")] OffsetDateTime);

#[test]
fn validate_offset_date_time_schema() {
    let schema = schema_for!(OffsetDateTimeDef);
    let schema_value = serde_json::to_value(&schema).unwrap();
    let validator = Validator::new(&schema_value).unwrap();

    let datetime = OffsetDateTime::from_unix_timestamp(1700000000).unwrap();
    let wrapper = TimeWrapper(datetime);
    let json = serde_json::to_value(&wrapper).unwrap();
    eprintln!("JSON: {:?}", json);
    eprintln!("Schema: {:?}", schema_value);

    assert!(validator.validate(&json).is_ok());
}
