use std::cmp::{self, Eq};
use std::collections::{HashMap, HashSet};

use chrono::{Local, Timelike};
use geo_types::Coord;

use crate::map::{Point, PublicTransport, Trip};
use crate::path::{Part, Path};
use crate::platforms::{Platforms, Walking};

pub struct Raptor {
    map: PublicTransport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Label {
    arrival: i64,
    from: Option<usize>,
    route: Option<usize>,
}

type Labels = Vec<Label>;
type PlatformIndex = usize;
type Marked = HashSet<PlatformIndex>;
type RouteIndex = usize;
type Routes = HashMap<RouteIndex, PlatformIndex>;

impl Label {
    pub fn new(arrival: i64, from: Option<usize>, route: Option<usize>) -> Self {
        Self {
            arrival,
            from,
            route,
        }
    }

    pub fn infinity() -> Self {
        Self {
            arrival: i64::MAX,
            from: None,
            route: None,
        }
    }
}

impl Raptor {
    pub fn new(map: PublicTransport) -> Self {
        Self { map }
    }

    pub fn find_path(
        &self,
        start: geo_types::Point<f64>,
        finish: geo_types::Point<f64>,
    ) -> Vec<Path> {
        let platforms = Platforms::new(&self.map.platforms, Platforms::zone(&start));
        let from = platforms.find(&start);
        let to = platforms.find(&finish);
        if !from.is_empty() && !to.is_empty() {
            let paths = self.run(from, to);
            return complete(paths, start, finish);
        }
        vec![]
    }

    fn run(&self, from: Vec<Walking>, to: Vec<Walking>) -> Vec<Path> {
        let departure = Local::now().num_seconds_from_midnight() as i64;
        let (mut best, mut marked) = self.init(departure, &from);
        let mut labels = vec![best.clone()];
        let mut k = 1;
        while !marked.is_empty() {
            labels.push(labels[k - 1].clone());
            let routes = self.accumulate(marked);
            marked = self.traverse(k, routes, &to, &mut best, &mut labels);
            marked.extend(self.transfer(&marked, &mut labels[k]));
            k += 1;
        }
        self.paths(from, to, labels)
    }

    fn init(&self, departure: i64, from: &Vec<Walking>) -> (Labels, HashSet<usize>) {
        let mut best = vec![Label::infinity(); self.map.platforms.len()];
        let mut marked = Marked::new();
        for w in from {
            best[w.platform] = Label::new(departure + w.duration, None, None);
            marked.insert(w.platform);
        }
        (best, marked)
    }

    fn accumulate(&self, marked: Marked) -> Routes {
        let mut routes = Routes::new();
        for mp in marked {
            for r in &self.map.platforms[mp].routes {
                let route = &self.map.routes[*r];
                let op = routes.get(r);
                if op.is_none() || route.before(&mp, op.unwrap()) {
                    routes.insert(*r, mp);
                }
            }
        }
        routes
    }

    fn traverse(
        &self,
        round: usize,
        routes: Routes,
        to: &Vec<Walking>,
        best: &mut Labels,
        labels: &mut Vec<Labels>,
    ) -> Marked {
        let mut marked = Marked::new();
        for (r, p) in routes {
            let mut trip = None;
            let route = &self.map.routes[r];
            let ordinal = route.ordinal[&p];
            for pi in &route.platforms[ordinal..] {
                // TODO - closed routes?
                let pi_ordinal = route.ordinal[pi];
                if trip.is_some() {
                    let trip: &Trip = trip.unwrap();
                    let arrival = trip.stops[pi_ordinal];
                    let minimal = minimal(*pi, to, best);
                    if arrival < minimal {
                        best[*pi] = Label::new(arrival, Some(p), Some(r));
                        labels[round][*pi] = best[*pi].clone();
                        marked.insert(*pi);
                    }
                }
                let arrival = labels[round - 1][*pi].arrival;
                let earlier_trip = earlier_trip(arrival, ordinal, &route.trips);
                if earlier_trip.is_some() && arrival <= earlier_trip.unwrap().stops[pi_ordinal] {
                    trip = earlier_trip
                }
            }
        }
        marked
    }

    fn transfer(&self, marked: &Marked, labels: &mut Labels) -> Marked {
        let mut also_marked = Marked::new();
        for from in marked {
            for passage in &self.map.passages[*from] {
                let prev = labels[passage.to].arrival;
                let other = labels[*from].arrival + passage.time;
                if other < prev {
                    labels[passage.to] = Label::new(other, Some(*from), None);
                    also_marked.insert(passage.to);
                }
            }
        }
        also_marked
    }

    fn paths(&self, from: Vec<Walking>, to: Vec<Walking>, round_labels: Vec<Labels>) -> Vec<Path> {
        let mut paths: Vec<Path> = Vec::new();
        for (k, labels) in round_labels.iter().enumerate() {
            for t in &to {
                if on_foot(labels, t.platform) {
                    continue;
                }
                if k > 0 && is_similar(labels, &round_labels[k - 1], t.platform) {
                    continue;
                }
                let mut parts: Vec<Part> = vec![];
                let mut finish = Some(t.platform);
                let mut start = labels[finish.unwrap()].from;
                while start.is_some() && finish.is_some() {
                    let from = start.unwrap();
                    let to = finish.unwrap();
                    parts.push(self.make_part(from, to, labels[to].route));
                    finish = start;
                    start = labels[to].from;
                }
                if is_from(finish.unwrap(), &from) {
                    parts.reverse();
                    paths.push(Path::new(parts));
                }
            }
        }
        paths
    }

    fn make_part(&self, from: usize, to: usize, route: Option<usize>) -> Part {
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
        Part::new(points)
    }
}

fn make_point(point: &Point) -> Coord<f64> {
    Coord {
        x: point.lon,
        y: point.lat,
    }
}

fn complete(paths: Vec<Path>, from: geo_types::Point<f64>, to: geo_types::Point<f64>) -> Vec<Path> {
    let from = from.into();
    let to = to.into();
    let mut completed = Vec::new();
    for path in paths {
        completed.push(make_path(&from, &to, path));
    }
    completed
}

fn make_first_walking(from: &Coord<f64>, path: &Path) -> Part {
    let first = path.first().first();
    Part::new(vec![*from, *first])
}

fn make_last_walking(to: &Coord<f64>, path: &Path) -> Part {
    let last = path.last().last();
    Part::new(vec![*last, *to])
}

fn make_path(from: &Coord<f64>, to: &Coord<f64>, path: Path) -> Path {
    let first = make_first_walking(&from, &path);
    let last = make_last_walking(&to, &path);

    let mut completed = Path::new(vec![first]);
    completed.concat(path);
    completed.push(last);
    completed
}

fn earlier_trip<'a>(arrival: i64, ordinal: usize, trips: &'a Vec<Trip>) -> Option<&'a Trip> {
    let i = trips.partition_point(|t| t.stops[ordinal] < arrival);
    trips.get(i)
}

fn minimal(pi: usize, to: &Vec<Walking>, best: &Labels) -> i64 {
    let times: Vec<i64> = to.iter().map(|w| best[w.platform].arrival).collect();
    cmp::min(best[pi].arrival, *times.iter().min().unwrap())
}

fn on_foot(labels: &Labels, platform: usize) -> bool {
    labels[platform].route.is_none()
}

fn is_similar(lhs: &Labels, rhs: &Labels, platform: usize) -> bool {
    lhs[platform] == rhs[platform]
}

fn is_from(platform: usize, from: &Vec<Walking>) -> bool {
    from.iter().find(|w| w.platform == platform).is_some()
}
