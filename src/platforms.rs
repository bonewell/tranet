use std::collections::HashMap;

use geo_types::Point;

use crate::map::{Platform, PlatformIndex, Time};

pub type Walking = HashMap<PlatformIndex, Time>;

#[derive(Default)]
pub struct Platforms {
    pub from: Walking,
    pub to: Walking,
}

impl Platforms {
    pub fn new(
        platforms: &Vec<Platform>,
        start: geo_types::Point<f64>,
        finish: geo_types::Point<f64>,
    ) -> Self {
        let from = find(start, platforms);
        let to = find(finish, platforms);
        Self { from, to }
    }

    pub fn from(start: Walking, finish: Walking) -> Self {
        Self {
            from: start,
            to: finish,
        }
    }
}

fn find(point: Point<f64>, platforms: &Vec<Platform>) -> Walking {
    let zone = zone(&point);
    let point = to_utm_point(&point, zone);
    let mut near = Walking::new();
    for (index, platform) in platforms.iter().enumerate() {
        let platform = Point::new(platform.point.lon, platform.point.lat);
        let platform = to_utm_point(&platform, zone);
        if is_near(&point, &platform) {
            near.insert(index, duration(&point, &platform));
        }
    }
    near
}

fn zone(point: &Point<f64>) -> u8 {
    utm::lat_lon_to_zone_number(point.y(), point.x())
}

fn to_utm_point(point: &Point<f64>, zone: u8) -> Point<f64> {
    let (x, y, _) = utm::to_utm_wgs84(point.y(), point.x(), zone);
    Point::new(x, y)
}

fn is_near(lhs: &Point<f64>, rhs: &Point<f64>) -> bool {
    let dx = lhs.x() - rhs.x();
    let dy = lhs.y() - rhs.y();
    let d2 = dx * dx + dy * dy;
    let r = 1000.0;
    d2 < r * r
}

fn duration(from: &Point<f64>, to: &Point<f64>) -> Time {
    let dx = from.x() - to.x();
    let dy = from.y() - to.y();
    let d2 = dx * dx + dy * dy;
    let speed: f64 = 5000.0 / 3600.0;
    (d2.sqrt() / speed).round() as Time
}

#[cfg(test)]
mod point {
    use super::*;

    use geo_types::coord;

    #[test]
    fn near() {
        let lhs = coord! { x: 5.0, y: 5.0 };
        let rhs = coord! { x: 1000.0, y: 5.0 };
        assert!(is_near(&lhs.into(), &rhs.into()));
    }

    #[test]
    fn far() {
        let lhs = coord! { x: 5.0, y: 5.0 };
        let rhs = coord! { x: 1005.0, y: 5.0 };
        assert!(!is_near(&lhs.into(), &rhs.into()));
    }

    #[test]
    fn calc_duration() {
        let lhs = coord! { x: 5.0, y: 5.0 };
        let rhs = coord! { x: 1005.0, y: 5.0 };
        assert_eq!(720, duration(&lhs.into(), &rhs.into()));
    }
}
