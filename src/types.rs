use std::collections::HashMap;

#[derive(Debug)]
pub struct Point {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug)]
pub struct Platform {
    pub point: Point,
    pub routes: Vec<i64>,
}

#[derive(Debug)]
pub struct Trip {
    pub stops: Vec<i64>,
}

#[derive(Debug)]
pub struct Route {
    pub circle: bool,
    pub platforms: Vec<i64>,
    pub trips: Vec<Trip>,
}

pub type Passage = (i64, i64);

#[derive(Debug)]
pub struct PublicTransport {
    pub platforms: Vec<Platform>,
    pub routes: Vec<Route>,
    pub passages: HashMap<Passage, i64>,
}
