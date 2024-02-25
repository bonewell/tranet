use std::fmt;

use geo_types::{Coord, Geometry, GeometryCollection, LineString, Point};
use wkt::ToWkt;

pub struct Part {
    points: Vec<Coord<f64>>,
}

pub struct Path {
    parts: Vec<Part>,
}

impl Part {
    pub fn new(points: Vec<Coord<f64>>) -> Self {
        Self { points }
    }
}

impl Path {
    pub fn new(parts: Vec<Part>) -> Self {
        Self { parts }
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut collection = GeometryCollection::default();
        for part in &self.parts {
            let line = LineString::new(part.points.clone());
            collection.0.push(Geometry::LineString(line));
            let point: Point<f64> = (*part.points.last().unwrap()).into();
            collection.0.push(Geometry::Point(point));
        }
        write!(f, "{}", collection.to_wkt())
    }
}
