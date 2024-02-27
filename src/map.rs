use std::collections::HashMap;

#[derive(Debug)]
pub struct Point {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug)]
pub struct Platform {
    pub point: Point,
    pub routes: Vec<usize>,
}

#[derive(Debug)]
pub struct Trip {
    pub stops: Vec<i64>,
}

#[derive(Debug)]
pub struct Route {
    pub circle: bool,
    pub platforms: Vec<usize>,
    pub trips: Vec<Trip>,
    pub ordinal: HashMap<usize, usize>,
}

impl Route {
    pub fn before(&self, lhs: &usize, rhs: &usize) -> bool {
        self.ordinal[&lhs] < self.ordinal[&rhs]
    }
}

#[derive(Debug)]
pub struct Passage {
    pub to: usize,
    pub time: i64,
}

impl Passage {
    pub fn new(to: usize, time: i64) -> Self {
        Self { to, time }
    }
}

#[derive(Debug)]
pub struct PublicTransport {
    pub platforms: Vec<Platform>,
    pub routes: Vec<Route>,
    pub passages: Vec<Vec<Passage>>,
}
