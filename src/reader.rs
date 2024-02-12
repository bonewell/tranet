use std::collections::{BTreeMap, HashMap};

use serde_pickle::{HashableValue, Value};

use crate::types::{Passage, Platform, Point, PublicTransport, Route, Trip};

fn value_to_vec<'a>(value: &'a Value, default: &'a Vec<Value>) -> &'a Vec<Value> {
    match value {
        Value::Tuple(items) => items,
        Value::List(items) => items,
        _ => default,
    }
}

fn value_to_dict<'a>(
    value: &'a Value,
    default: &'a BTreeMap<HashableValue, Value>,
) -> &'a BTreeMap<HashableValue, Value> {
    match value {
        Value::Dict(items) => items,
        _ => default,
    }
}

fn value_to_i64(value: &Value) -> i64 {
    match value {
        Value::I64(i) => *i,
        _ => 0,
    }
}

fn hashvalue_to_i64(value: &HashableValue) -> i64 {
    match value {
        HashableValue::I64(i) => *i,
        _ => 0,
    }
}

fn value_to_f64(value: &Value) -> f64 {
    match value {
        Value::F64(f) => *f,
        _ => 0.0,
    }
}

fn value_to_bool(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        _ => false,
    }
}

fn make_vec<T: for<'a> std::convert::From<&'a Value>>(value: &Value) -> Vec<T> {
    value_to_vec(&value, &vec![])
        .iter()
        .map(|v| v.into())
        .collect()
}

fn make_vec_of_i64(value: &Value) -> Vec<i64> {
    value_to_vec(&value, &vec![])
        .iter()
        .map(|v| value_to_i64(v))
        .collect()
}

fn make_passage(from: &HashableValue, to: &HashableValue) -> Passage {
    (hashvalue_to_i64(&from), hashvalue_to_i64(&to))
}

fn make_time(value: &Value) -> i64 {
    value_to_f64(&value) as i64
}

fn make_map(value: &Value) -> HashMap<Passage, i64> {
    let default = BTreeMap::new();
    let mut map = HashMap::new();
    for (from, v) in value_to_dict(&value, &default).iter() {
        for (to, time) in value_to_dict(&v, &default).iter() {
            map.insert(make_passage(from, to), make_time(time));
        }
    }
    map
}

impl From<&Value> for Point {
    fn from(value: &Value) -> Self {
        let default = BTreeMap::new();
        let point = value_to_dict(&value, &default);
        let lat = value_to_f64(
            point
                .get(&HashableValue::String(String::from("lat")))
                .unwrap_or(&Value::None),
        );
        let lon = value_to_f64(
            point
                .get(&HashableValue::String(String::from("lon")))
                .unwrap_or(&Value::None),
        );
        Self { lat, lon }
    }
}

impl From<&Value> for Platform {
    fn from(value: &Value) -> Self {
        let default = BTreeMap::new();
        let platform = value_to_dict(&value, &default);
        let point = platform
            .get(&HashableValue::String(String::from("point")))
            .unwrap_or(&Value::None)
            .into();
        let routes = make_vec_of_i64(
            platform
                .get(&HashableValue::String(String::from("routes")))
                .unwrap_or(&Value::None),
        );
        Self { point, routes }
    }
}

impl From<&Value> for Trip {
    fn from(value: &Value) -> Self {
        let stops = make_vec_of_i64(&value);
        Self { stops }
    }
}

impl From<&Value> for Route {
    fn from(value: &Value) -> Self {
        let default = BTreeMap::new();
        let route = value_to_dict(&value, &default);
        let circle = value_to_bool(
            route
                .get(&HashableValue::String(String::from("circle")))
                .unwrap_or(&Value::None),
        );
        let platforms = make_vec_of_i64(
            route
                .get(&HashableValue::String(String::from("platforms")))
                .unwrap_or(&Value::None),
        );
        let trips = make_vec(
            route
                .get(&HashableValue::String(String::from("trips")))
                .unwrap_or(&Value::None),
        );
        Self {
            circle,
            platforms,
            trips,
        }
    }
}

impl From<&Value> for PublicTransport {
    fn from(value: &Value) -> Self {
        let default = vec![];
        let fields = value_to_vec(&value, &default);
        let platforms = make_vec(fields.get(0).unwrap_or(&Value::None));
        let routes = make_vec(fields.get(1).unwrap_or(&Value::None));
        let passages = make_map(fields.get(2).unwrap_or(&Value::None));
        Self {
            platforms,
            routes,
            passages,
        }
    }
}
