use geo_types::Coord;

use crate::map::{Point, PublicTransport};
use crate::path::{Part, Path};
use crate::platforms::{Platforms, Walking};

pub struct Raptor {
    map: PublicTransport,
}

impl Raptor {
    pub fn new(map: PublicTransport) -> Self {
        Self { map }
    }

    pub fn find_path(&self, from: geo_types::Point<f64>, to: geo_types::Point<f64>) -> Vec<Path> {
        let platforms = Platforms::new(&self.map.platforms, Platforms::zone(&from));
        let paths = self.run(platforms.find(&from), platforms.find(&to));
        complete(paths, from, to)
    }

    fn run(&self, from: Vec<Walking>, to: Vec<Walking>) -> Vec<Path> {
        let mut paths: Vec<Path> = Vec::new();
        for f in from {
            for t in &to {
                let part = Part::new(vec![
                    make_point(&self.map.platforms[f.platform].point),
                    make_point(&self.map.platforms[t.platform].point),
                ]);
                paths.push(Path::new(vec![part]));
            }
        }
        paths
    }
}

fn make_point(point: &Point) -> Coord<f64> {
    Coord {
        x: point.lon,
        y: point.lat,
    }
}

fn complete(paths: Vec<Path>, from: geo_types::Point<f64>, to: geo_types::Point<f64>) -> Vec<Path> {
    let from: Coord<f64> = from.into();
    let to: Coord<f64> = to.into();
    let mut completed: Vec<Path> = Vec::new();
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
