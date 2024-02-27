use geo_types::Point;

use crate::map::Platform;

pub struct Platforms<'a> {
    platforms: &'a Vec<Platform>,
    zone: u8,
}

#[derive(Debug)]
pub struct Walking {
    pub platform: usize,
    pub duration: i64,
}

impl Walking {
    fn new(platform: usize, duration: i64) -> Self {
        Self { platform, duration }
    }
}

impl<'a> Platforms<'a> {
    pub fn new(platforms: &'a Vec<Platform>, zone: u8) -> Self {
        Self { platforms, zone }
    }

    pub fn zone(point: &Point<f64>) -> u8 {
        utm::lat_lon_to_zone_number(point.y(), point.x())
    }

    fn to_utm_point(&self, point: &Point<f64>) -> Point<f64> {
        let (x, y, _) = utm::to_utm_wgs84(point.y(), point.x(), self.zone);
        Point::new(x, y)
    }

    pub fn find(&self, point: &Point<f64>) -> Vec<Walking> {
        let mut near = Vec::new();
        let point = self.to_utm_point(point);
        for (index, platform) in self.platforms.iter().enumerate() {
            let platform = Point::new(platform.point.lon, platform.point.lat);
            let platform = self.to_utm_point(&platform);
            if is_near(&point, &platform) {
                let walking = Walking::new(index, duration(&point, &platform));
                near.push(walking);
            }
        }
        near
    }
}

fn is_near(lhs: &Point<f64>, rhs: &Point<f64>) -> bool {
    let dx = lhs.x() - rhs.x();
    let dy = lhs.y() - rhs.y();
    let d2 = dx * dx + dy * dy;
    let r = 1000.0;
    d2 < r * r
}

fn duration(from: &Point<f64>, to: &Point<f64>) -> i64 {
    let dx = from.x() - to.x();
    let dy = from.y() - to.y();
    let d2 = dx * dx + dy * dy;
    let speed: f64 = 5000.0 / 3600.0;
    (d2.sqrt() / speed).round() as i64
}
