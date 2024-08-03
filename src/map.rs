use std::{collections::HashMap, ops::Range};

pub type Time = i64;
pub type RouteIndex = usize;
pub type PlatformIndex = usize;
pub type OrdinalNumber = usize;

#[derive(Debug, Default)]
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
    id: i32,
    stops: Vec<Time>,
}

impl Trip {
    pub fn new(id: i32, stops: Vec<Time>) -> Self {
        Self { id, stops }
    }

    pub fn stop(&self, ordinal: OrdinalNumber, circle: bool) -> Time {
        let length = match circle {
            true => self.stops.len() - 1,
            false => self.stops.len(),
        };
        self.stops[ordinal % length]
    }

    pub fn first(&self) -> &Time {
        self.stops.first().unwrap()
    }

    pub fn last(&self) -> &Time {
        self.stops.last().unwrap()
    }
}

impl PartialEq for Trip {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug)]
pub struct Route {
    pub circle: bool,
    platforms: Vec<PlatformIndex>,
    trips: Vec<Trip>,
    ordinal: HashMap<PlatformIndex, OrdinalNumber>,
}

impl Route {
    pub fn new(circle: bool, platforms: Vec<PlatformIndex>, trips: Vec<Trip>) -> Self {
        let ordinal = platforms
            .iter()
            .enumerate()
            .rev()
            .map(|(index, platform)| (*platform, index))
            .collect();
        let platforms = match circle {
            true => [platforms.as_slice(), platforms.as_slice()].concat(),
            false => platforms,
        };
        Self {
            circle,
            platforms,
            trips,
            ordinal,
        }
    }

    fn seam(&self) -> OrdinalNumber {
        (self.platforms.len() / 2) - 1
    }

    pub fn platform(&self, ordinal: OrdinalNumber) -> PlatformIndex {
        self.platforms[ordinal]
    }

    pub fn is_before(&self, lhs: &PlatformIndex, rhs: &PlatformIndex) -> bool {
        self.ordinal[lhs] < self.ordinal[rhs]
    }

    pub fn is_seam(&self, ordinal: OrdinalNumber) -> bool {
        self.circle && ordinal == self.seam()
    }

    fn has_earlier(&self, time: Time, ordinal: OrdinalNumber, trip: &Trip) -> bool {
        time < trip.stop(ordinal, self.circle)
    }

    fn next_trip(&self, time: Time, ordinal: OrdinalNumber) -> Option<&Trip> {
        let i = self
            .trips
            .partition_point(|t| t.stop(ordinal, self.circle) < time);
        self.trips.get(i)
    }

    fn try_catch_next_trip(
        &self,
        time: Time,
        ordinal: OrdinalNumber,
        current_trip: Option<&Trip>,
    ) -> Option<&Trip> {
        match current_trip {
            Some(trip) if !self.has_earlier(time, ordinal, trip) => None,
            _ => self.next_trip(time, ordinal),
        }
    }

    fn next_trip_on_seam(&self, trip: &Trip) -> Option<&Trip> {
        let time = *trip.last();
        self.next_trip(time, 0)
    }

    fn try_catch_next_trip_on_seam(
        &self,
        time: Time,
        current_trip: Option<&Trip>,
    ) -> Option<&Trip> {
        match current_trip {
            Some(trip) => self.next_trip_on_seam(trip),
            None => {
                let trip = self.next_trip(time, self.seam());
                match trip {
                    Some(trip) => self.next_trip_on_seam(trip),
                    None => None,
                }
            }
        }
    }

    pub fn try_catch(
        &self,
        time: Time,
        ordinal: OrdinalNumber,
        current_trip: Option<&Trip>,
    ) -> Option<&Trip> {
        match self.is_seam(ordinal) {
            true => self.try_catch_next_trip_on_seam(time, current_trip),
            false => self.try_catch_next_trip(time, ordinal, current_trip),
        }
    }

    pub fn range(&self, from: OrdinalNumber, to: OrdinalNumber) -> &[PlatformIndex] {
        &self.platforms[from..=to]
    }

    pub fn tail(&self, platform: &PlatformIndex) -> Range<OrdinalNumber> {
        let from = self.ordinal[platform];
        match self.circle {
            true => from..(from + self.platforms.len() / 2),
            false => from..self.platforms.len(),
        }
    }
}

#[derive(Debug, Clone)]
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

impl PublicTransport {
    pub fn new(platforms: Vec<Platform>, routes: Vec<Route>, passages: Vec<Vec<Passage>>) -> Self {
        Self {
            platforms,
            routes,
            passages,
        }
    }
}

#[cfg(test)]
mod trip {
    use super::*;

    fn route() -> Route {
        let trips = vec![
            Trip::new(1, vec![10, 60, 70]),
            Trip::new(2, vec![30, 90, 100]),
            Trip::new(3, vec![50, 110, 120]),
        ];
        Route::new(false, vec![0, 1, 2], trips)
    }

    #[test]
    fn no_trip() {
        let route = route();
        let trip = route.try_catch(60, 0, None);
        assert!(trip.is_none());
    }

    #[test]
    fn yes_trip() {
        let route = route();
        let trip = route.try_catch(70, 1, None);
        assert!(trip.is_some());
        assert_eq!(2, trip.unwrap().id);
    }

    #[test]
    fn no_move_trip() {
        let route = route();
        let trip = &route.trips[0];
        let trip = route.try_catch(60, 1, Some(trip));
        assert!(trip.is_none());
    }

    fn circle_route() -> Route {
        let trips = vec![
            Trip::new(1, vec![10, 60, 70, 80]),
            Trip::new(2, vec![40, 90, 110, 120]),
            Trip::new(3, vec![80, 130, 140, 150]),
        ];
        Route::new(true, vec![0, 1, 2], trips)
    }

    #[test]
    fn catch_circle_trip() {
        let route = circle_route();
        let trip = route.try_catch(60, 1, None);
        assert!(trip.is_some());
        assert_eq!(1, trip.unwrap().id);
    }

    #[test]
    fn catch_circle_trip_on_seam() {
        let route = circle_route();
        let trip = route.try_catch(70, 2, None);
        assert!(trip.is_some());
        assert_eq!(3, trip.unwrap().id);
    }

    #[test]
    fn route_without_loop() {
        let route = route();
        assert!(route.is_before(&1, &2));
    }

    fn loop_route() -> Route {
        let trips = vec![
            Trip::new(1, vec![10, 60, 70, 80]),
            Trip::new(2, vec![30, 90, 100, 110]),
            Trip::new(3, vec![50, 110, 120, 130]),
        ];
        Route::new(false, vec![0, 1, 2, 1], trips)
    }

    #[test]
    fn route_with_loop() {
        let route = loop_route();
        assert!(route.is_before(&1, &2));
    }

    #[test]
    fn route_with_loop_reverse() {
        let route = loop_route();
        assert!(!route.is_before(&2, &1));
    }
}
