use std::collections::BTreeMap;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

use geo_types::Point;
use serde_pickle::{value_from_reader, HashableValue, Value};
use wkt::TryFromWkt;

use crate::map;
use crate::map::{Passage, Platform, PublicTransport, Route, Trip};

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

fn make_vec_of_index(value: &Value) -> Vec<usize> {
    value_to_vec(&value, &vec![])
        .iter()
        .map(|v| make_index(v))
        .collect()
}

fn make_usize(value: &HashableValue) -> usize {
    hashvalue_to_i64(&value) as usize
}

fn make_index(value: &Value) -> usize {
    value_to_i64(&value) as usize
}

fn make_time(value: &Value) -> i64 {
    value_to_f64(&value) as i64
}

fn make_vec_of_passage(value: &Value, size: usize) -> Vec<Vec<Passage>> {
    let mut passages: Vec<Vec<Passage>> = vec![];
    passages.resize_with(size, Default::default);
    let default = BTreeMap::new();
    for (from, v) in value_to_dict(&value, &default).iter() {
        for (to, time) in value_to_dict(&v, &default).iter() {
            passages[make_usize(&from)].push(Passage::new(make_usize(&to), make_time(time)));
        }
    }
    passages
}

impl From<&Value> for map::Point {
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
        map::Point::new(lat, lon)
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
        let routes = make_vec_of_index(
            platform
                .get(&HashableValue::String(String::from("routes")))
                .unwrap_or(&Value::None),
        );
        Platform::new(point, routes)
    }
}

impl From<&Value> for Trip {
    fn from(value: &Value) -> Self {
        let stops = make_vec_of_i64(&value);
        static mut ID: i64 = 0;
        unsafe {
            let id = ID;
            ID += 1;
            Trip::new(id, stops)
        }
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
        let platforms = make_vec_of_index(
            route
                .get(&HashableValue::String(String::from("platforms")))
                .unwrap_or(&Value::None),
        );
        let platforms = match circle {
            true => platforms[..platforms.len() - 1].to_vec(),
            false => platforms.to_vec(),
        };
        let trips = make_vec(
            route
                .get(&HashableValue::String(String::from("trips")))
                .unwrap_or(&Value::None),
        );
        static mut ID: i64 = 0;
        unsafe {
            let id = ID;
            ID += 1;
            Route::new(id, circle, platforms, trips)
        }
    }
}

impl From<&Value> for PublicTransport {
    fn from(value: &Value) -> Self {
        let default = vec![];
        let fields = value_to_vec(&value, &default);
        let platforms = make_vec(fields.get(0).unwrap_or(&Value::None));
        let routes = make_vec(fields.get(1).unwrap_or(&Value::None));
        let passages = make_vec_of_passage(fields.get(2).unwrap_or(&Value::None), platforms.len());
        Self {
            platforms,
            routes,
            passages,
        }
    }
}

pub fn read_map(filename: &String) -> PublicTransport {
    let reader: Box<dyn Read> = Box::new(File::open(&filename).expect("Can not open map"));
    let decoded: Value = value_from_reader(reader, Default::default()).expect("Can not parse map");
    (&decoded).into()
}

fn parse_points(line: &String) -> (Point<f64>, Point<f64>) {
    let points: Vec<&str> = line.split(',').collect();
    (
        Point::try_from_wkt_str(points[0]).unwrap(),
        Point::try_from_wkt_str(points[1]).unwrap(),
    )
}

pub fn read_points(filename: &String) -> Vec<(Point<f64>, Point<f64>)> {
    let mut points = Vec::new();
    let file = File::open(filename).expect("Can not open points");
    for line in BufReader::new(file).lines().flatten() {
        points.push(parse_points(&line));
    }
    points
}
