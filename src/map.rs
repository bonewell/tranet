use std::collections::HashMap;

pub type Time = i64;
pub type RouteIndex = usize;
pub type PlatformIndex = usize;
pub type SequenceNumber = usize;

#[derive(Debug)]
pub struct Point {
    pub lat: f64,
    pub lon: f64,
}

impl Point {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }
}

#[derive(Debug)]
pub struct Platform {
    pub point: Point,
    pub routes: Vec<RouteIndex>,
}

impl Platform {
    pub fn new(point: Point, routes: Vec<RouteIndex>) -> Self {
        Self { point, routes }
    }
}

#[derive(Debug)]
pub struct Trip {
    pub stops: Vec<Time>,
}

impl Trip {
    pub fn new(stops: Vec<Time>) -> Self {
        Self { stops }
    }
}

#[derive(Debug)]
pub struct Route {
    pub circle: bool,
    pub platforms: Vec<PlatformIndex>,
    pub trips: Vec<Trip>,
    pub ordinal: HashMap<PlatformIndex, SequenceNumber>,
}

impl Route {
    pub fn new(circle: bool, platforms: Vec<PlatformIndex>, trips: Vec<Trip>) -> Self {
        let ordinal = platforms
            .iter()
            .enumerate()
            .map(|(index, platform)| (*platform, index))
            .collect();
        Self {
            circle,
            platforms,
            trips,
            ordinal,
        }
    }

    pub fn before(&self, lhs: &PlatformIndex, rhs: &PlatformIndex) -> bool {
        self.ordinal[&lhs] < self.ordinal[&rhs]
    }
}

#[derive(Debug)]
pub struct Passage {
    pub to: PlatformIndex,
    pub time: Time,
}

impl Passage {
    pub fn new(to: PlatformIndex, time: Time) -> Self {
        Self { to, time }
    }
}

#[derive(Debug)]
pub struct PublicTransport {
    pub platforms: Vec<Platform>,
    pub routes: Vec<Route>,
    pub passages: Vec<Vec<Passage>>,
}
