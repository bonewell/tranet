use geo_types::Coord;

use crate::map::{Point, PublicTransport};
use crate::path::{Part, Path};
use crate::platforms::Platforms;

pub struct Raptor<'a> {
    map: &'a PublicTransport,
}

impl<'a> Raptor<'a> {
    pub fn new(map: &'a PublicTransport) -> Self {
        Self { map }
    }

    pub fn find_path(&self, from: &geo_types::Point<f64>, to: &geo_types::Point<f64>) -> Vec<Path> {
        let mut paths: Vec<Path> = Vec::new();
        let platforms = Platforms::new(&self.map.platforms, Platforms::zone(&from));
        for f in platforms.find(&from) {
            for t in platforms.find(&to) {
                paths.push(self.make_path(f, t, &from.clone().into(), &to.clone().into()));
            }
        }
        paths
    }

    fn make_path(&self, f: usize, t: usize, from: &Coord<f64>, to: &Coord<f64>) -> Path {
        let part1: Part = Part::new(vec![from.clone(), make_point(&self.map.platforms[f].point)]);
        let part2: Part = Part::new(vec![
            make_point(&self.map.platforms[f].point),
            make_point(&self.map.platforms[t].point),
        ]);
        let part3: Part = Part::new(vec![make_point(&self.map.platforms[t].point), to.clone()]);
        Path::new(vec![part1, part2, part3])
    }
}

fn make_point(point: &Point) -> Coord<f64> {
    Coord {
        x: point.lon,
        y: point.lat,
    }
}
