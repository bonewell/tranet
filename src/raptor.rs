use geo_types::Point;

use crate::map::PublicTransport;
use crate::platforms::Platforms;

pub struct Raptor<'a> {
    map: &'a PublicTransport,
}

impl<'a> Raptor<'a> {
    pub fn new(map: &'a PublicTransport) -> Self {
        Self { map }
    }

    pub fn find_path(&self, from: &Point<f64>, to: &Point<f64>) {
        let platforms = Platforms::new(&self.map.platforms, Platforms::zone(&from));
        let _from = platforms.find(&from);
        let _to = platforms.find(&to);
    }
}
