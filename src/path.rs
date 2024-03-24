use std::fmt;

use geo_types::{Coord, Geometry, GeometryCollection, LineString};
use wkt::ToWkt;

use crate::map::{RouteIndex, Time};

#[derive(Debug)]
pub struct Part {
    points: Vec<Coord<f64>>,
    route: Option<RouteIndex>,
}

impl Part {
    pub fn new(points: Vec<Coord<f64>>, route: Option<RouteIndex>) -> Self {
        Self { points, route }
    }

    pub fn first(&self) -> &Coord<f64> {
        self.points.first().unwrap()
    }

    pub fn last(&self) -> &Coord<f64> {
        self.points.last().unwrap()
    }
}

#[derive(Debug)]
pub struct Path {
    pub parts: Vec<Part>,
    pub arrival: Time,
}

impl Path {
    pub fn new(parts: Vec<Part>, arrival: Time) -> Self {
        Self { parts, arrival }
    }

    pub fn first(&self) -> &Part {
        self.parts.first().unwrap()
    }

    pub fn last(&self) -> &Part {
        self.parts.last().unwrap()
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut routes = vec![];
        let mut collection = GeometryCollection::default();
        for part in &self.parts {
            routes.push(part.route);
            let line = LineString::new(part.points.clone());
            collection.0.push(Geometry::LineString(line));
            let point = (*part.points.last().unwrap()).into();
            collection.0.push(Geometry::Point(point));
        }
        write!(f, "{}", collection.to_wkt())
    }
}
