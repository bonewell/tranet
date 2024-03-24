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

#[derive(Debug)]
pub struct Platform {
    pub point: Point,
    pub routes: Vec<RouteIndex>,
}

#[derive(Debug)]
pub struct Trip {
    pub stops: Vec<Time>,
}

#[derive(Debug)]
pub struct Route {
    pub circle: bool,
    pub platforms: Vec<PlatformIndex>,
    pub trips: Vec<Trip>,
    pub ordinal: HashMap<PlatformIndex, SequenceNumber>,
}

impl Route {
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
