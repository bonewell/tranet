use std::cmp::{self, Eq};
use std::collections::{HashMap, HashSet};

use chrono::{Local, Timelike};
use geo_types::Coord;

use crate::map::{PlatformIndex, Point, PublicTransport, Route, RouteIndex, Time, Trip};
use crate::path::{Part, Path};
use crate::platforms::Platforms;

type GeoPoint = geo_types::Point<f64>;

pub struct Raptor {
    map: PublicTransport,
}

impl Raptor {
    pub fn new(map: PublicTransport) -> Self {
        Self { map }
    }

    pub fn find_path(&self, start: GeoPoint, finish: GeoPoint) -> Vec<Path> {
        let platforms = Platforms::new(&self.map.platforms, start, finish);
        let mut searcher = Searcher::new(&self.map, platforms);
        if searcher.ready() {
            let paths = searcher.run();
            return complete(paths, start, finish);
        }
        vec![]
    }
}

type Marked = HashSet<PlatformIndex>;
type Routes = HashMap<RouteIndex, PlatformIndex>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Label {
    arrival: Time,
    from: Option<PlatformIndex>,
    route: Option<RouteIndex>,
}

type Labels = Vec<Label>;

impl Label {
    fn new(arrival: Time, from: Option<PlatformIndex>, route: Option<RouteIndex>) -> Self {
        Self {
            arrival,
            from,
            route,
        }
    }

    fn infinity() -> Self {
        Self {
            arrival: Time::MAX,
            from: None,
            route: None,
        }
    }
}

struct Boarding<'a> {
    platform: PlatformIndex,
    trip: &'a Trip,
}

impl<'a> Boarding<'a> {
    fn new(platform: PlatformIndex, trip: &'a Trip) -> Self {
        Self { platform, trip }
    }
}

struct Searcher<'a> {
    map: &'a PublicTransport,
    platforms: Platforms,
    arrival: Time,
    best: Labels,
    labels: Vec<Labels>,
}

impl<'a> Searcher<'a> {
    fn new(map: &'a PublicTransport, platforms: Platforms) -> Self {
        Self {
            map,
            platforms,
            arrival: Time::MAX,
            best: vec![Label::infinity(); map.platforms.len()],
            labels: vec![],
        }
    }

    fn ready(&self) -> bool {
        !self.platforms.from.is_empty() && !self.platforms.to.is_empty()
    }

    fn run(&mut self) -> Vec<Path> {
        let departure = Local::now().num_seconds_from_midnight() as Time;
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
            self.best[*platform] = Label::new(departure + duration, None, None);
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
                if route.circle {
                    continue;
                }
                let op = routes.get(r);
                if op.is_none() || route.before(&mp, op.unwrap()) {
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
            let mut boarding: Option<Boarding> = None;
            let route = &self.map.routes[r];
            let ordinal = route.ordinal[&p];
            // TODO - closed routes?
            for pi in &route.platforms[ordinal..] {
                let pi_ordinal = route.ordinal[pi];
                if let Some(boarding) = &boarding {
                    let arrival = boarding.trip.stops[pi_ordinal];
                    // With local and target pruning
                    let minimal = cmp::min(self.best[*pi].arrival, self.arrival);
                    if arrival < minimal {
                        self.best[*pi] = Label::new(arrival, Some(boarding.platform), Some(r));
                        self.labels[round][*pi] = self.best[*pi].clone();
                        marked.insert(*pi);
                        self.update(pi, arrival);
                    }
                }
                let arrival = self.labels[round - 1][*pi].arrival;
                boarding = match boarding {
                    None => try_catch(arrival, pi, route),
                    Some(b) if arrival < b.trip.stops[pi_ordinal] => try_catch(arrival, pi, route),
                    _ => boarding,
                }
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
                    self.best[passage.to] = Label::new(arrival, Some(*from), None);
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
                if k > 0 && is_similar(labels, &self.labels[k - 1], *platform) {
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
        let mut finish = Some(platform);
        let mut start = labels[finish.unwrap()].from;
        while start.is_some() && finish.is_some() {
            let from = start.unwrap();
            let to = finish.unwrap();
            parts.push(self.make_part(from, to, labels[to].route));
            finish = start;
            start = labels[from].from;
        }
        if !self.is_from(&finish) {
            return None;
        }
        let arrival = labels[platform].arrival + duration;
        parts.reverse();
        Some(Path::new(parts, arrival))
    }

    fn make_part(&self, from: PlatformIndex, to: PlatformIndex, route: Option<RouteIndex>) -> Part {
        let mut points = vec![];
        points.push(make_point(&self.map.platforms[from].point));
        if route.is_some() {
            let route = &self.map.routes[route.unwrap()];
            let from = route.ordinal[&from];
            let to = route.ordinal[&to];
            for p in &route.platforms[from..=to] {
                points.push(make_point(&self.map.platforms[*p].point));
            }
        }
        points.push(make_point(&self.map.platforms[to].point));
        Part::new(points, route)
    }

    fn update(&mut self, platform: &PlatformIndex, arrival: Time) {
        if let Some(duration) = self.platforms.to.get(platform) {
            self.arrival = arrival + duration;
        }
    }

    fn is_from(&self, platform: &Option<PlatformIndex>) -> bool {
        platform.is_some() && self.platforms.from.contains_key(&platform.unwrap())
    }
}

fn make_point(point: &Point) -> Coord<f64> {
    Coord {
        x: point.lon,
        y: point.lat,
    }
}

fn complete(paths: Vec<Path>, from: GeoPoint, to: GeoPoint) -> Vec<Path> {
    let from = from.into();
    let to = to.into();
    let mut completed = Vec::new();
    for path in paths {
        completed.push(make_path(&from, &to, path));
    }
    completed.sort_by(|a, b| a.arrival.cmp(&b.arrival));
    completed
}

fn make_first_walking(from: &Coord<f64>, path: &Path) -> Part {
    let first = path.first().first();
    Part::new(vec![*from, *first], None)
}

fn make_last_walking(to: &Coord<f64>, path: &Path) -> Part {
    let last = path.last().last();
    Part::new(vec![*last, *to], None)
}

fn make_path(from: &Coord<f64>, to: &Coord<f64>, path: Path) -> Path {
    let first = make_first_walking(&from, &path);
    let last = make_last_walking(&to, &path);
    let mut parts = vec![];
    parts.push(first);
    parts.extend(path.parts);
    parts.push(last);
    Path::new(parts, path.arrival)
}

fn try_catch<'a>(
    departure: Time,
    platform: &PlatformIndex,
    route: &'a Route,
) -> Option<Boarding<'a>> {
    let ordinal = route.ordinal[platform];
    let i = route
        .trips
        .partition_point(|t| t.stops[ordinal] < departure);
    match route.trips.get(i) {
        Some(trip) => Some(Boarding::new(*platform, &trip)),
        _ => None,
    }
}

fn on_foot(labels: &Labels, platform: PlatformIndex) -> bool {
    labels[platform].route.is_none()
}

fn is_similar(lhs: &Labels, rhs: &Labels, platform: PlatformIndex) -> bool {
    lhs[platform] == rhs[platform]
}

#[cfg(test)]
mod labels {
    use super::*;

    #[test]
    fn similar_infinity() {
        let lhs = vec![Label::infinity()];
        let rhs = vec![Label::infinity()];
        assert!(is_similar(&lhs, &rhs, 0));
    }

    #[test]
    fn similar() {
        let lhs = vec![Label::new(34, Some(2), Some(8))];
        let rhs = vec![Label::new(34, Some(2), Some(8))];
        assert!(is_similar(&lhs, &rhs, 0));
    }

    #[test]
    fn different_route() {
        let lhs = vec![Label::new(34, Some(2), Some(1))];
        let rhs = vec![Label::new(34, Some(2), Some(8))];
        assert!(!is_similar(&lhs, &rhs, 0));
    }

    #[test]
    fn different_platform() {
        let lhs = vec![Label::new(34, Some(2), Some(8))];
        let rhs = vec![Label::new(34, Some(3), Some(8))];
        assert!(!is_similar(&lhs, &rhs, 0));
    }

    #[test]
    fn different_time() {
        let lhs = vec![Label::new(34, Some(2), Some(8))];
        let rhs = vec![Label::new(67, Some(2), Some(8))];
        assert!(!is_similar(&lhs, &rhs, 0));
    }

    #[test]
    fn different() {
        let lhs = vec![Label::new(34, Some(2), Some(8))];
        let rhs = vec![Label::new(67, Some(3), Some(1))];
        assert!(!is_similar(&lhs, &rhs, 0));
    }
}

#[cfg(test)]
mod utils {
    use crate::{map::Route, map::Trip, raptor::try_catch};

    fn route() -> Route {
        let trips = vec![
            Trip::new(vec![10, 60, 70]),
            Trip::new(vec![30, 90, 100]),
            Trip::new(vec![50, 110, 120]),
        ];
        Route::new(false, vec![0, 1, 2], trips)
    }

    #[test]
    fn no_trip() {
        let route = route();
        let boarding = try_catch(60, &0, &route);
        assert!(boarding.is_none());
    }

    #[test]
    fn catch_trip() {
        let route = route();
        let boarding = try_catch(70, &1, &route);
        assert!(boarding.is_some());
        assert_eq!(90, boarding.unwrap().trip.stops[1]);
    }
}

#[cfg(test)]
mod searcher {
    use crate::{
        map::{Platform, Point, PublicTransport, Route},
        platforms::Platforms,
        raptor::{Marked, Routes, Searcher},
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
            Route::new(false, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9], vec![]),
            Route::new(false, vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19], vec![]),
        ]
    }

    fn map() -> PublicTransport {
        PublicTransport {
            platforms: platforms(),
            routes: routes(),
            passages: vec![],
        }
    }

    #[test]
    fn accumulate_routes() {
        let map = map();
        let platforms = Platforms::default();
        let searcher = Searcher::new(&map, platforms);
        let marked = Marked::from([11, 5, 2, 14]);
        let expected = Routes::from([(0, 2), (1, 11)]);
        assert_eq!(expected, searcher.accumulate(marked));
    }
}
