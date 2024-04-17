use std::collections::HashMap;

pub type Time = i64;
pub type RouteIndex = usize;
pub type PlatformIndex = usize;
pub type SequenceNumber = usize;

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
    pub stops: Vec<Time>,
}

impl Trip {
    pub fn new(id: i32, stops: Vec<Time>) -> Self {
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
    circle: bool,
    platforms: Vec<PlatformIndex>,
    trips: Vec<Trip>,
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

    pub fn is_before(&self, lhs: &PlatformIndex, rhs: &PlatformIndex) -> bool {
        self.ordinal[&lhs] < self.ordinal[&rhs]
    }

    pub fn is_seam(&self, platform: &PlatformIndex) -> bool {
        self.circle && platform == self.platforms.last().unwrap()
    }

    fn next_trip(&self, time: Time, platform: &PlatformIndex) -> Option<&Trip> {
        let ordinal = self.ordinal[platform];
        let i = self.trips.partition_point(|t| t.stops[ordinal] < time);
        self.trips.get(i)
    }

    fn next_circle_trip(&self, current_trip: &Trip) -> Option<&Trip> {
        let arrival = *current_trip.stops.last().unwrap();
        let platform = self.platforms.first().unwrap();
        self.next_trip(arrival, platform)
    }

    fn move_on(&self, arrival: Time, platform: &PlatformIndex, trip: &Trip) -> bool {
        let ordinal = self.ordinal[platform];
        let next_circle = self.is_seam(platform);
        let was_here_earlier = arrival < trip.stops[ordinal];
        !next_circle && !was_here_earlier
    }

    pub fn try_catch(
        &self,
        time: Time,
        platform: &PlatformIndex,
        trip: Option<&Trip>,
    ) -> Option<&Trip> {
        if let Some(trip) = trip {
            if self.move_on(time, platform, trip) {
                return None;
            }
            if self.is_seam(platform) {
                return self.next_circle_trip(trip);
            }
        }
        self.next_trip(time, platform)
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
        Route::new(false, vec![0, 1, 2], trips)
    }

    #[test]
    fn no_trip() {
        let route = route();
        let trip = route.next_trip(60, &0);
        assert!(trip.is_none());
    }

    #[test]
    fn yes_trip() {
        let route = route();
        let trip = route.next_trip(70, &1);
        assert!(trip.is_some());
        assert_eq!(90, trip.unwrap().stops[1]);
    }
}
