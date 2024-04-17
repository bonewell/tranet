use std::cmp::{self, Eq};
use std::collections::{HashMap, HashSet};

use geo_types::Coord;

use crate::map::{PlatformIndex, Point, PublicTransport, Route, RouteIndex, Time, Trip};
use crate::path::{Part, Path};
use crate::platforms::Platforms;

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
            let mut boarding: Option<Boarding> = None;
            let route = &self.map.routes[r];
            for pi in route.tail(p) {
                let pi_ordinal = route.ordinal[&pi];
                if let Some(boarding) = &boarding {
                    let arrival = boarding.trip.stops[pi_ordinal];
                    // With local and target pruning
                    let minimal = cmp::min(self.best[pi].arrival, self.arrival);
                    if arrival < minimal {
                        self.best[pi] = Label::new(arrival, Some(boarding.platform), Some(r));
                        self.labels[round][pi] = self.best[pi].clone();
                        marked.insert(pi);
                        self.update(&pi, arrival);
                    }
                }
                let trip = self.try_catch(&pi, route, &boarding);
                if let Some(trip) = trip {
                    boarding = match &boarding {
                        Some(b) if trip == b.trip => boarding,
                        _ => Some(Boarding::new(pi, trip)),
                    };
                }
            }
        }
        marked
    }

    fn try_catch(
        &self,
        platform: &PlatformIndex,
        route: &'a Route,
        boarding: &Option<Boarding>,
    ) -> Option<&'a Trip> {
        let arrival = self.best[*platform].arrival;
        match boarding {
            Some(b) => route.try_catch(arrival, platform, Some(b.trip)),
            None => route.try_catch(arrival, platform, None),
        }
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
        if route.is_some() {
            let route = &self.map.routes[route.unwrap()];
            for p in route.range(from, to) {
                points.push(make_point(&self.map.platforms[p].point));
            }
        } else {
            points.push(make_point(&self.map.platforms[from].point));
            points.push(make_point(&self.map.platforms[to].point));
        }
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

    fn passages() -> Vec<Vec<Passage>> {
        let mut passages = vec![];
        for _ in 0..20 {
            passages.push(vec![]);
        }
        passages[2] = vec![Passage::new(0, 10), Passage::new(7, 20)];
        passages[15] = vec![Passage::new(10, 15), Passage::new(19, 30)];
        passages
    }

    fn map() -> PublicTransport {
        PublicTransport {
            platforms: platforms(),
            routes: routes(),
            passages: passages(),
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

    #[test]
    fn do_transfer() {
        let map = map();
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
        let map = map();
        let platforms = Platforms::default();
        let mut searcher = Searcher::new(&map, platforms);
        let routes = Routes::from([(0, 2), (1, 11)]);
        let expected = Marked::from([3, 4, 5, 6, 7, 12, 13, 14, 15, 16, 17]);
        searcher.best = vec![Label::infinity(); 20];
        searcher.best[2].arrival = 10;
        searcher.best[11].arrival = 15;
        searcher.best[8].arrival = 20;
        searcher.best[9].arrival = 30;
        searcher.best[18].arrival = 25;
        searcher.best[19].arrival = 35;
        searcher.labels = vec![searcher.best.clone(); 2];
        assert_eq!(expected, searcher.traverse(routes));
    }
}
