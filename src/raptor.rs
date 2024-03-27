use geo_types::Coord;

use crate::map::PublicTransport;
use crate::path::{Part, Path};
use crate::platforms::Platforms;
use crate::searcher::Searcher;

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
