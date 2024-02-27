use std::fmt;

use geo_types::{Coord, Geometry, GeometryCollection, LineString};
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

    pub fn first(&self) -> &Coord<f64> {
        self.points.first().unwrap()
    }

    pub fn last(&self) -> &Coord<f64> {
        self.points.last().unwrap()
    }
}

impl Path {
    pub fn new(parts: Vec<Part>) -> Self {
        Self { parts }
    }

    pub fn first(&self) -> &Part {
        self.parts.first().unwrap()
    }

    pub fn last(&self) -> &Part {
        self.parts.last().unwrap()
    }

    pub fn push(&mut self, part: Part) {
        self.parts.push(part);
    }

    pub fn concat(&mut self, path: Path) {
        self.parts.extend(path.parts);
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut collection = GeometryCollection::default();
        for part in &self.parts {
            let line = LineString::new(part.points.clone());
            collection.0.push(Geometry::LineString(line));
            let point = (*part.points.last().unwrap()).into();
            collection.0.push(Geometry::Point(point));
        }
        write!(f, "{}", collection.to_wkt())
    }
}
