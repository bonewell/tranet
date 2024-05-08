use std::collections::{HashMap, HashSet};

use crate::map::{PlatformIndex, PublicTransport, Route, RouteIndex, Time, Trip};
use crate::path::{Part, Path};
use crate::platforms::Platforms;

type Marked = HashSet<PlatformIndex>;
type Routes = HashMap<RouteIndex, PlatformIndex>;

#[derive(Debug, Clone, PartialEq)]
struct Label<'a> {
    arrival: Time,
    from: Option<PlatformIndex>,
    route: Option<&'a Route>,
}

type Labels<'a> = Vec<Label<'a>>;

impl<'a> Label<'a> {
    fn new(arrival: Time, from: Option<PlatformIndex>, route: Option<&'a Route>) -> Self {
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

    fn dominate(&self, other: &Label) -> bool {
        self.arrival < other.arrival
    }
}

struct Vehicle<'a> {
    route: &'a Route,
    trip: Option<&'a Trip>,
    boarding: Option<PlatformIndex>,
}

impl<'a> Vehicle<'a> {
    fn new(route: &'a Route) -> Self {
        Self {
            route,
            boarding: None,
            trip: None,
        }
    }

    fn on_way(&self) -> bool {
        self.trip.is_some()
    }

    fn arrival(&self, platform: &PlatformIndex) -> Time {
        let ordinal = self.route.ordinal[platform];
        self.trip.unwrap().stops[ordinal]
    }

    fn make_label(&self, platform: &PlatformIndex) -> Label {
        Label::new(self.arrival(platform), self.boarding, Some(self.route))
    }

    fn update(&mut self, label: &Label) {
        if let Some(platform) = &label.from {
            let trip = self.route.try_catch(label.arrival, platform, self.trip);
            if let Some(next_trip) = trip {
                match self.trip {
                    Some(current_trip) if current_trip == next_trip => (),
                    Some(current_trip)
                        if self.route.is_seam(platform)
                            && is_same_vehicle(&current_trip, next_trip) =>
                    {
                        self.trip = Some(next_trip)
                    }
                    _ => {
                        self.boarding = Some(*platform);
                        self.trip = Some(next_trip);
                    }
                };
            }
        }
    }
}

fn is_same_vehicle(current_trip: &Trip, next_trip: &Trip) -> bool {
    current_trip.stops.last().unwrap() == next_trip.stops.first().unwrap()
}

pub struct Searcher<'a> {
    map: &'a PublicTransport,
    platforms: Platforms,
    target: Label<'a>,
    best: Labels<'a>,
    labels: Vec<Labels<'a>>,
}

impl<'a> Searcher<'a> {
    pub fn new(map: &'a PublicTransport, platforms: Platforms) -> Self {
        Self {
            map,
            platforms,
            target: Label::infinity(),
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
            let route = &self.map.routes[r];
            let mut vehicle = Vehicle::new(route);
            for pi in route.tail(p) {
                if vehicle.on_way() {
                    let label = vehicle.make_label(&pi);
                    if label.dominate(self.best_label(&pi)) {
                        self.best[pi] = label;
                        self.labels[round][pi] = self.best[pi].clone();
                        marked.insert(pi);
                        self.update_target(pi);
                    }
                }
                vehicle.update(&self.best[pi]);
            }
        }
        marked
    }

    fn best_label(&self, platform: &PlatformIndex) -> &Label {
        // With local and target pruning
        let label = &self.best[*platform];
        match self.target.dominate(label) {
            true => &self.target,
            false => label,
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

    fn make_part(&self, from: PlatformIndex, to: PlatformIndex, route: Option<&Route>) -> Part {
        let points = match route {
            Some(route) => route
                .range(from, to)
                .iter()
                .map(|p| (&self.map.platforms[*p].point).into())
                .collect(),
            None => vec![
                (&self.map.platforms[from].point).into(),
                (&self.map.platforms[to].point).into(),
            ],
        };
        let route_id = match route {
            Some(route) => Some(route.id),
            None => None,
        };
        Part::new(points, route_id)
    }

    fn update_target(&mut self, platform: PlatformIndex) {
        if let Some(duration) = self.platforms.to.get(&platform) {
            let arrival = &self.best[platform].arrival + duration;
            let new_target = Label::new(arrival, Some(platform), None);
            if new_target.dominate(&self.target) {
                self.target = new_target;
            }
        }
    }

    fn is_from(&self, platform: &Option<PlatformIndex>) -> bool {
        platform.is_some() && self.platforms.from.contains_key(&platform.unwrap())
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
        let route = Route::new(1, false, vec![], vec![]);
        let lhs = vec![Label::new(34, Some(2), Some(&route))];
        let rhs = vec![Label::new(34, Some(2), Some(&route))];
        assert!(is_similar(&lhs, &rhs, 0));
    }

    #[test]
    fn different_route() {
        let route1 = Route::new(1, false, vec![], vec![]);
        let route2 = Route::new(8, false, vec![], vec![]);
        let lhs = vec![Label::new(34, Some(2), Some(&route1))];
        let rhs = vec![Label::new(34, Some(2), Some(&route2))];
        assert!(!is_similar(&lhs, &rhs, 0));
    }

    #[test]
    fn different_platform() {
        let route = Route::new(8, false, vec![], vec![]);
        let lhs = vec![Label::new(34, Some(2), Some(&route))];
        let rhs = vec![Label::new(34, Some(3), Some(&route))];
        assert!(!is_similar(&lhs, &rhs, 0));
    }

    #[test]
    fn different_time() {
        let route = Route::new(8, false, vec![], vec![]);
        let lhs = vec![Label::new(34, Some(2), Some(&route))];
        let rhs = vec![Label::new(67, Some(2), Some(&route))];
        assert!(!is_similar(&lhs, &rhs, 0));
    }

    #[test]
    fn different() {
        let route1 = Route::new(1, false, vec![], vec![]);
        let route2 = Route::new(8, false, vec![], vec![]);
        let lhs = vec![Label::new(34, Some(2), Some(&route2))];
        let rhs = vec![Label::new(67, Some(3), Some(&route1))];
        assert!(!is_similar(&lhs, &rhs, 0));
    }
}

#[cfg(test)]
mod searcher {
    use super::*;

    use crate::{
        map::{Passage, Platform, Point},
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
                0,
                false,
                vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
                vec![Trip::new(1, vec![20, 25, 30, 35, 40, 45, 50, 55, 60, 65])],
            ),
            Route::new(
                1,
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
