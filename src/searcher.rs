use std::cmp::{self, Eq};
use std::collections::{HashMap, HashSet};

use geo_types::Coord;

use crate::map::{
    OrdinalNumber, PlatformIndex, Point, PublicTransport, Route, RouteIndex, Time, Trip,
};
use crate::path::{Part, Path};
use crate::platforms::Platforms;

type Marked = HashSet<PlatformIndex>;
type Routes = HashMap<RouteIndex, PlatformIndex>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Stop {
    platform: PlatformIndex,
    ordinal: Option<OrdinalNumber>,
}

impl Stop {
    fn new(platform: PlatformIndex, ordinal: Option<OrdinalNumber>) -> Self {
        Self { platform, ordinal }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Way {
    from: Stop,
    to: Stop,
    route: Option<RouteIndex>,
}

impl Way {
    fn new(from: Stop, to: Stop, route: Option<RouteIndex>) -> Self {
        Self { from, to, route }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Label {
    arrival: Time,
    way: Option<Way>,
}

type Labels = Vec<Label>;

impl Label {
    fn new(arrival: Time, way: Option<Way>) -> Self {
        Self { arrival, way }
    }

    fn infinity() -> Self {
        Self {
            arrival: Time::MAX,
            way: None,
        }
    }
}

struct Vehicle<'a> {
    index: RouteIndex,
    route: &'a Route,
    trip: Option<&'a Trip>,
    from: Option<OrdinalNumber>,
}

impl<'a> Vehicle<'a> {
    fn new(index: RouteIndex, route: &'a Route) -> Self {
        Self {
            index,
            route,
            from: None,
            trip: None,
        }
    }

    fn on_way(&self) -> bool {
        self.trip.is_some() && self.from.is_some()
    }

    fn arrival(&self, ordinal: OrdinalNumber) -> Time {
        self.trip.unwrap().stop(ordinal, self.route.circle)
    }

    fn make_stop(&self, ordinal: OrdinalNumber) -> Stop {
        Stop::new(self.route.platform(ordinal), Some(ordinal))
    }

    fn way(&self, to: OrdinalNumber) -> Way {
        Way::new(
            self.make_stop(self.from.unwrap()),
            self.make_stop(to),
            Some(self.index),
        )
    }

    fn update(&mut self, time: Time, ordinal: OrdinalNumber) {
        let trip = self.route.try_catch(time, ordinal, self.trip);
        if let Some(next_trip) = trip {
            match self.trip {
                Some(current_trip) if current_trip == next_trip => (),
                Some(current_trip)
                    if self.route.is_seam(ordinal) && is_same_vehicle(&current_trip, next_trip) =>
                {
                    self.trip = Some(next_trip)
                }
                _ => {
                    self.from = Some(ordinal);
                    self.trip = Some(next_trip);
                }
            };
        }
    }
}

fn is_same_vehicle(current_trip: &Trip, next_trip: &Trip) -> bool {
    current_trip.last() == next_trip.first()
}

pub struct Searcher<'a> {
    map: &'a PublicTransport,
    platforms: Platforms,
    arrival: Time,
    best: Labels,
    labels: Vec<Labels>,
}

impl<'a> Searcher<'a> {
    pub fn new(map: &'a PublicTransport, platforms: Platforms) -> Self {
        Self {
            map,
            platforms,
            arrival: Time::MAX,
            best: vec![Label::infinity(); map.platforms.len()],
            labels: vec![],
        }
    }

    pub fn ready(&self) -> bool {
        !self.platforms.from.is_empty() && !self.platforms.to.is_empty()
    }

    pub fn run(&mut self, departure: Time) -> Vec<Path> {
        let mut marked = self.init(departure);
        while !marked.is_empty() {
            self.round();
            let routes = self.accumulate(marked);
            marked = self.traverse(routes);
            marked.extend(self.transfer(&marked));
        }
        self.paths()
    }

    fn round(&mut self) {
        self.labels.push(self.labels.last().unwrap().clone());
    }

    fn init(&mut self, departure: Time) -> Marked {
        let mut marked = Marked::new();
        for (platform, duration) in &self.platforms.from {
            self.best[*platform] = Label::new(departure + duration, None);
            marked.insert(*platform);
        }
        self.labels = vec![self.best.clone()];
        marked
    }

    fn accumulate(&self, marked: Marked) -> Routes {
        let mut routes = Routes::new();
        for mp in marked {
            for r in &self.map.platforms[mp].routes {
                let route = &self.map.routes[*r];
                let op = routes.get(r);
                if op.is_none() || route.is_before(&mp, op.unwrap()) {
                    routes.insert(*r, mp);
                }
            }
        }
        routes
    }

    fn traverse(&mut self, routes: Routes) -> Marked {
        let round = self.labels.len() - 1;
        let mut marked = Marked::new();
        for (r, p) in routes {
            let route = &self.map.routes[r];
            let mut vehicle = Vehicle::new(r, route);
            for ordinal in route.tail(&p) {
                let platform = route.platform(ordinal);
                if vehicle.on_way() {
                    let arrival = vehicle.arrival(ordinal);
                    // With local and target pruning
                    let minimal = cmp::min(self.best[platform].arrival, self.arrival);
                    if arrival < minimal {
                        self.best[platform] = Label::new(arrival, Some(vehicle.way(ordinal)));
                        self.labels[round][platform] = self.best[platform].clone();
                        marked.insert(platform);
                        self.update(&platform, arrival);
                    }
                }
                let arrival = self.best[platform].arrival;
                vehicle.update(arrival, ordinal);
            }
        }
        marked
    }

    fn transfer(&mut self, marked: &Marked) -> Marked {
        let round = self.labels.len() - 1;
        let labels = &mut self.labels[round];
        let mut also_marked = Marked::new();
        for from in marked {
            for passage in &self.map.passages[*from] {
                let minimal = labels[passage.to].arrival;
                let arrival = labels[*from].arrival + passage.time;
                if arrival < minimal {
                    let from = Stop::new(*from, None);
                    let to = Stop::new(passage.to, None);
                    let way = Way::new(from, to, None);
                    self.best[passage.to] = Label::new(arrival, Some(way));
                    labels[passage.to] = self.best[passage.to].clone();
                    also_marked.insert(passage.to);
                }
            }
        }
        also_marked
    }

    fn paths(&self) -> Vec<Path> {
        let mut paths: Vec<Path> = Vec::new();
        for (k, labels) in self.labels.iter().enumerate() {
            for (platform, duration) in &self.platforms.to {
                if k > 0 && labels[*platform] == self.labels[k - 1][*platform] {
                    continue;
                }
                if let Some(p) = self.unwind(labels, *platform, *duration) {
                    paths.push(p);
                }
            }
        }
        paths
    }

    fn unwind(&self, labels: &Vec<Label>, platform: PlatformIndex, duration: Time) -> Option<Path> {
        if on_foot(labels, platform) {
            return None;
        }
        let mut parts: Vec<Part> = vec![];
        let mut way = labels[platform].way.as_ref();
        let mut from: Option<&PlatformIndex> = None;
        while way.is_some() {
            let w = way.unwrap();
            parts.push(self.make_part(w));
            from = Some(&w.from.platform);
            way = labels[w.from.platform].way.as_ref();
        }
        if !self.is_from(from) {
            return None;
        }
        let arrival = labels[platform].arrival + duration;
        parts.reverse();
        Some(Path::new(parts, arrival))
    }

    fn make_part(&self, way: &Way) -> Part {
        let mut points = vec![];
        if way.route.is_some() {
            let route = &self.map.routes[way.route.unwrap()];
            let range = route.range(way.from.ordinal.unwrap(), way.to.ordinal.unwrap());
            for p in range {
                points.push(make_point(&self.map.platforms[*p].point));
            }
        } else {
            points.push(make_point(&self.map.platforms[way.from.platform].point));
            points.push(make_point(&self.map.platforms[way.to.platform].point));
        }
        Part::new(points, way.route)
    }

    fn update(&mut self, platform: &PlatformIndex, arrival: Time) {
        if let Some(duration) = self.platforms.to.get(platform) {
            self.arrival = cmp::min(arrival + duration, self.arrival);
        }
    }

    fn is_from(&self, platform: Option<&PlatformIndex>) -> bool {
        platform.is_some() && self.platforms.from.contains_key(platform.unwrap())
    }
}

fn make_point(point: &Point) -> Coord<f64> {
    Coord {
        x: point.lon,
        y: point.lat,
    }
}

fn on_foot(labels: &Labels, platform: PlatformIndex) -> bool {
    match &labels[platform].way {
        Some(way) => way.route.is_none(),
        None => true,
    }
}

#[cfg(test)]
mod labels {
    use super::*;

    #[test]
    fn similar_infinity() {
        let lhs = Label::infinity();
        let rhs = Label::infinity();
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn similar() {
        let lhs = Label::new(
            34,
            Some(Way::new(
                Stop::new(0, Some(0)),
                Stop::new(2, Some(2)),
                Some(8),
            )),
        );
        let rhs = Label::new(
            34,
            Some(Way::new(
                Stop::new(0, Some(0)),
                Stop::new(2, Some(2)),
                Some(8),
            )),
        );
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn different_route() {
        let lhs = Label::new(
            34,
            Some(Way::new(
                Stop::new(0, Some(0)),
                Stop::new(2, Some(2)),
                Some(1),
            )),
        );
        let rhs = Label::new(
            34,
            Some(Way::new(
                Stop::new(0, Some(0)),
                Stop::new(2, Some(2)),
                Some(8),
            )),
        );
        assert_ne!(lhs, rhs);
    }

    #[test]
    fn different_platform() {
        let lhs = Label::new(
            34,
            Some(Way::new(
                Stop::new(0, Some(0)),
                Stop::new(2, Some(2)),
                Some(8),
            )),
        );
        let rhs = Label::new(
            34,
            Some(Way::new(
                Stop::new(1, Some(0)),
                Stop::new(2, Some(2)),
                Some(8),
            )),
        );
        assert_ne!(lhs, rhs);
    }

    #[test]
    fn different_ordinal() {
        let lhs = Label::new(
            34,
            Some(Way::new(
                Stop::new(0, Some(0)),
                Stop::new(2, Some(2)),
                Some(8),
            )),
        );
        let rhs = Label::new(
            34,
            Some(Way::new(
                Stop::new(0, Some(1)),
                Stop::new(2, Some(2)),
                Some(8),
            )),
        );
        assert_ne!(lhs, rhs);
    }

    #[test]
    fn different_time() {
        let lhs = Label::new(
            34,
            Some(Way::new(
                Stop::new(0, Some(0)),
                Stop::new(2, Some(2)),
                Some(8),
            )),
        );
        let rhs = Label::new(
            67,
            Some(Way::new(
                Stop::new(0, Some(0)),
                Stop::new(2, Some(2)),
                Some(8),
            )),
        );
        assert_ne!(lhs, rhs);
    }

    #[test]
    fn different() {
        let lhs = Label::new(
            34,
            Some(Way::new(
                Stop::new(0, Some(0)),
                Stop::new(2, Some(2)),
                Some(1),
            )),
        );
        let rhs = Label::new(
            67,
            Some(Way::new(
                Stop::new(1, Some(1)),
                Stop::new(2, Some(2)),
                Some(8),
            )),
        );
        assert_ne!(lhs, rhs);
    }
}

#[cfg(test)]
mod searcher {
    use super::*;

    use crate::{
        map::{Passage, Platform},
        platforms::Platforms,
    };

    fn platforms() -> Vec<Platform> {
        let mut platforms = vec![];
        for _ in 0..10 {
            platforms.push(Platform::new(Point::new(0.0, 0.0), vec![0]))
        }
        for _ in 0..10 {
            platforms.push(Platform::new(Point::new(0.0, 0.0), vec![1]))
        }
        platforms
    }

    fn routes() -> Vec<Route> {
        vec![
            Route::new(
                false,
                vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
                vec![Trip::new(1, vec![20, 25, 30, 35, 40, 45, 50, 55, 60, 65])],
            ),
            Route::new(
                false,
                vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19],
                vec![Trip::new(2, vec![25, 30, 35, 40, 45, 50, 55, 60, 65, 70])],
            ),
        ]
    }

    fn loop_routes() -> Vec<Route> {
        vec![
            Route::new(
                false,
                vec![0, 1, 2, 3, 4, 3, 6, 7, 8, 9],
                vec![Trip::new(1, vec![20, 25, 30, 35, 40, 45, 50, 55, 60, 65])],
            ),
            Route::new(
                false,
                vec![10, 11, 12, 13, 14, 15, 16, 15, 18, 19],
                vec![Trip::new(2, vec![25, 30, 35, 40, 45, 50, 55, 60, 65, 70])],
            ),
        ]
    }

    fn passages() -> Vec<Vec<Passage>> {
        let mut passages = vec![];
        for _ in 0..20 {
            passages.push(vec![]);
        }
        passages[2] = vec![Passage::new(0, 10), Passage::new(7, 20)];
        passages[15] = vec![Passage::new(10, 15), Passage::new(19, 30)];
        passages
    }

    #[test]
    fn accumulate_routes() {
        let map = PublicTransport::new(platforms(), routes(), passages());
        let platforms = Platforms::default();
        let searcher = Searcher::new(&map, platforms);
        let marked = Marked::from([11, 5, 2, 14]);

        let expected = Routes::from([(0, 2), (1, 11)]);
        assert_eq!(expected, searcher.accumulate(marked));
    }

    #[test]
    fn accumulate_loop_routes() {
        let map = PublicTransport::new(platforms(), loop_routes(), passages());
        let platforms = Platforms::default();
        let searcher = Searcher::new(&map, platforms);
        let marked = Marked::from([3, 4, 15, 16]);

        let expected = Routes::from([(0, 3), (1, 15)]);
        assert_eq!(expected, searcher.accumulate(marked));
    }

    #[test]
    fn do_transfer() {
        let map = PublicTransport::new(platforms(), routes(), passages());
        let platforms = Platforms::default();
        let mut searcher = Searcher::new(&map, platforms);
        let marked = Marked::from([2, 15]);

        let expected = Marked::from([0, 7, 10, 19]);
        searcher.labels = vec![vec![Label::infinity(); 20]];
        searcher.labels[0][2].arrival = 10;
        searcher.labels[0][15].arrival = 15;
        assert_eq!(expected, searcher.transfer(&marked));
    }

    #[test]
    fn do_traverse() {
        let map = PublicTransport::new(platforms(), routes(), passages());
        let platforms = Platforms::default();
        let mut searcher = Searcher::new(&map, platforms);
        let routes = Routes::from([(0, 2), (1, 11)]);
        searcher.best = vec![Label::infinity(); 20];
        searcher.best[2].arrival = 10;
        searcher.best[11].arrival = 15;
        searcher.best[8].arrival = 20;
        searcher.best[9].arrival = 30;
        searcher.best[18].arrival = 25;
        searcher.best[19].arrival = 35;
        searcher.labels = vec![searcher.best.clone(); 2];

        let expected = Marked::from([3, 4, 5, 6, 7, 12, 13, 14, 15, 16, 17]);
        assert_eq!(expected, searcher.traverse(routes));
    }

    #[test]
    fn do_loop_traverse() {
        let map = PublicTransport::new(platforms(), loop_routes(), passages());
        let platforms = Platforms::default();
        let mut searcher = Searcher::new(&map, platforms);
        let routes = Routes::from([(0, 3), (1, 15)]);
        searcher.best = vec![Label::new(1, None); 20];
        searcher.best[4] = Label::infinity();
        searcher.best[16] = Label::infinity();
        searcher.labels = vec![searcher.best.clone(); 2];

        let expected = Marked::from([4, 16]);
        assert_eq!(expected, searcher.traverse(routes));
    }
}
