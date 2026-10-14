//! `serde_json::Value` <-> `google.protobuf.Struct` conversion. `buffa-types`
//! has no built-in serde_json interop, and several resources (library
//! options' flattened `attribute_values`, settings `values` maps) carry
//! arbitrary-shaped JSON that has no fixed proto schema — `Struct` is the
//! faithful protobuf representation of "arbitrary JSON object", so this is
//! shared conversion glue rather than one-off per resource.

use buffa_types::google::protobuf::value::Kind;
use buffa_types::google::protobuf::{ListValue, Struct, Value as BuffaValue};
use serde_json::{Map, Value as JsonValue};

pub fn json_to_struct(value: JsonValue) -> Struct {
    match value {
        JsonValue::Object(map) => Struct {
            fields: map.into_iter().map(|(k, v)| (k, json_to_buffa_value(v))).collect(),
        ..Default::default()
        },
        // Not an object — has no legal Struct representation; callers of
        // this function only ever pass an Object (see flatten_option), so
        // this arm exists only to make the match total.
        _ => Struct::default(),
    }
}

pub fn json_to_buffa_value(value: JsonValue) -> BuffaValue {
    match value {
        JsonValue::Null => BuffaValue::null(),
        JsonValue::Bool(b) => BuffaValue { kind: Some(Kind::BoolValue(b)) , ..Default::default() },
        JsonValue::Number(n) => {
            BuffaValue { kind: Some(Kind::NumberValue(n.as_f64().unwrap_or_default())) , ..Default::default() }
        }
        JsonValue::String(s) => BuffaValue { kind: Some(Kind::StringValue(s)) , ..Default::default() },
        JsonValue::Array(items) => BuffaValue {
            kind: Some(Kind::ListValue(Box::new(ListValue {
                values: items.into_iter().map(json_to_buffa_value).collect(),
            ..Default::default()
            }))),
        ..Default::default()
        },
        JsonValue::Object(map) => BuffaValue {
            kind: Some(Kind::StructValue(Box::new(Struct {
                fields: map.into_iter().map(|(k, v)| (k, json_to_buffa_value(v))).collect(),
            ..Default::default()
            }))),
        ..Default::default()
        },
    }
}

pub fn struct_to_json(value: Struct) -> JsonValue {
    let mut map = Map::new();
    for (k, v) in value.fields {
        map.insert(k, buffa_value_to_json(v));
    }
    JsonValue::Object(map)
}

pub fn buffa_value_to_json(value: BuffaValue) -> JsonValue {
    match value.kind {
        None | Some(Kind::NullValue(_)) => JsonValue::Null,
        Some(Kind::BoolValue(b)) => JsonValue::Bool(b),
        Some(Kind::NumberValue(n)) => {
            serde_json::Number::from_f64(n).map(JsonValue::Number).unwrap_or(JsonValue::Null)
        }
        Some(Kind::StringValue(s)) => JsonValue::String(s),
        Some(Kind::ListValue(list)) => {
            JsonValue::Array(list.values.into_iter().map(buffa_value_to_json).collect())
        }
        Some(Kind::StructValue(s)) => struct_to_json(*s),
    }
}
