use std::collections::HashMap;

pub type PlatformIndex = usize;
pub type RouteIndex = usize;

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
    pub stops: Vec<i64>,
}

#[derive(Debug)]
pub struct Route {
    pub circle: bool,
    pub platforms: Vec<PlatformIndex>,
    pub trips: Vec<Trip>,
    pub ordinal: HashMap<PlatformIndex, usize>,
}

impl Route {
    pub fn before(&self, lhs: &PlatformIndex, rhs: &PlatformIndex) -> bool {
        self.ordinal[&lhs] < self.ordinal[&rhs]
    }
}

#[derive(Debug)]
pub struct Passage {
    pub to: PlatformIndex,
    pub time: i64,
}

impl Passage {
    pub fn new(to: PlatformIndex, time: i64) -> Self {
        Self { to, time }
    }
}

#[derive(Debug)]
pub struct PublicTransport {
    pub platforms: Vec<Platform>,
    pub routes: Vec<Route>,
    pub passages: Vec<Vec<Passage>>,
}
