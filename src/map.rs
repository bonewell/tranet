use std::collections::HashMap;

use geo_types::Coord;

pub type Time = i64;
pub type RouteIndex = usize;
pub type RouteId = i64;
pub type TripId = i64;
pub type PlatformIndex = usize;
pub type SequenceNumber = usize;

#[derive(Debug, Default)]
pub struct Point {
    pub lat: f64,
    pub lon: f64,
}

impl From<&Point> for Coord<f64> {
    fn from(point: &Point) -> Self {
        Self {
            x: point.lon,
            y: point.lat,
        }
    }
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
    id: TripId,
    pub stops: Vec<Time>,
}

impl Trip {
    pub fn new(id: TripId, stops: Vec<Time>) -> Self {
        Self { id, stops }
    }
}

impl PartialEq for Trip {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug)]
pub struct Route {
    pub id: RouteId,
    circle: bool,
    platforms: Vec<PlatformIndex>,
    trips: Vec<Trip>,
    pub ordinal: HashMap<PlatformIndex, SequenceNumber>,
}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Route {
    pub fn new(id: RouteId, circle: bool, platforms: Vec<PlatformIndex>, trips: Vec<Trip>) -> Self {
        let ordinal = platforms
            .iter()
            .enumerate()
            .map(|(index, platform)| (*platform, index))
            .collect();
        Self {
            id,
            circle,
            platforms,
            trips,
            ordinal,
        }
    }

    pub fn is_before(&self, lhs: &PlatformIndex, rhs: &PlatformIndex) -> bool {
        self.ordinal[&lhs] < self.ordinal[&rhs]
    }

    pub fn is_seam(&self, platform: &PlatformIndex) -> bool {
        self.circle && platform == self.platforms.last().unwrap()
    }

    fn has_earlier(&self, time: Time, platform: &PlatformIndex, trip: &Trip) -> bool {
        let ordinal = self.ordinal[platform];
        time < trip.stops[ordinal]
    }

    fn next_trip(&self, time: Time, platform: &PlatformIndex) -> Option<&Trip> {
        let ordinal = self.ordinal[platform];
        let i = self.trips.partition_point(|t| t.stops[ordinal] < time);
        self.trips.get(i)
    }

    fn try_catch_next_trip(
        &self,
        time: Time,
        platform: &PlatformIndex,
        current_trip: Option<&Trip>,
    ) -> Option<&Trip> {
        match current_trip {
            Some(trip) if !self.has_earlier(time, platform, trip) => None,
            _ => self.next_trip(time, platform),
        }
    }

    fn next_trip_on_seam(&self, trip: &Trip) -> Option<&Trip> {
        let platform = self.platforms.first().unwrap();
        let time = *trip.stops.last().unwrap();
        self.next_trip(time, platform)
    }

    fn try_catch_next_trip_on_seam(
        &self,
        time: Time,
        platform: &PlatformIndex,
        current_trip: Option<&Trip>,
    ) -> Option<&Trip> {
        match current_trip {
            Some(trip) => self.next_trip_on_seam(trip),
            None => {
                let trip = self.next_trip(time, platform);
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
        platform: &PlatformIndex,
        current_trip: Option<&Trip>,
    ) -> Option<&Trip> {
        match self.is_seam(platform) {
            true => self.try_catch_next_trip_on_seam(time, platform, current_trip),
            false => self.try_catch_next_trip(time, platform, current_trip),
        }
    }

    pub fn range(&self, from: PlatformIndex, to: PlatformIndex) -> Vec<PlatformIndex> {
        let from = self.ordinal[&from];
        let to = self.ordinal[&to];
        match self.circle {
            true if from > to => [&self.platforms[from..], &self.platforms[..=to]].concat(),
            _ => self.platforms[from..=to].to_vec(),
        }
    }

    pub fn tail(&self, from: PlatformIndex) -> Vec<PlatformIndex> {
        let ordinal = self.ordinal[&from];
        match self.circle {
            true => [&self.platforms[ordinal..], &self.platforms[..ordinal]].concat(),
            false => self.platforms[ordinal..].to_vec(),
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
        Route::new(0, false, vec![0, 1, 2], trips)
    }

    #[test]
    fn no_trip() {
        let route = route();
        let trip = route.try_catch(60, &0, None);
        assert!(trip.is_none());
    }

    #[test]
    fn yes_trip() {
        let route = route();
        let trip = route.try_catch(70, &1, None);
        assert!(trip.is_some());
        assert_eq!(2, trip.unwrap().id);
    }

    #[test]
    fn no_move_trip() {
        let route = route();
        let trip = &route.trips[0];
        let trip = route.try_catch(60, &1, Some(trip));
        assert!(trip.is_none());
    }

    fn circle_route() -> Route {
        let trips = vec![
            Trip::new(1, vec![10, 60, 70, 80]),
            Trip::new(2, vec![40, 90, 110, 120]),
            Trip::new(3, vec![80, 130, 140, 150]),
        ];
        Route::new(0, true, vec![0, 1, 2], trips)
    }

    #[test]
    fn catch_circle_trip() {
        let route = circle_route();
        let trip = route.try_catch(60, &1, None);
        assert!(trip.is_some());
        assert_eq!(1, trip.unwrap().id);
    }

    #[test]
    fn catch_circle_trip_on_seam() {
        let route = circle_route();
        let trip = route.try_catch(70, &2, None);
        assert!(trip.is_some());
        assert_eq!(3, trip.unwrap().id);
    }
}
