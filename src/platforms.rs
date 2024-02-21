use geo_types::Point;

use crate::map::Platform;

pub struct Platforms<'a> {
    platforms: &'a Vec<Platform>,
    zone: u8,
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

    pub fn find(&self, point: &Point<f64>) -> Vec<usize> {
        let point = self.to_utm_point(point);
        let mut near: Vec<usize> = Vec::new();
        for (i, p) in self.platforms.iter().enumerate() {
            let location = self.to_utm_point(&Point::new(p.point.lon, p.point.lat));
            if is_near(&point, &location) {
                near.push(i);
            }
        }
        near
    }
}

fn is_near(lhs: &Point<f64>, rhs: &Point<f64>) -> bool {
    let dx = lhs.x() - rhs.x();
    let dy = lhs.y() - rhs.y();
    let d2 = dx * dx + dy * dy;
    let r = 3000.0;
    d2 < r * r
}
