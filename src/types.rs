use serde_pickle as pickle;

#[derive(Debug)]
pub struct Platform {
    id: i32,
    directions: Vec<i32>,
}

#[derive(Debug)]
pub struct PublicTransport {
    platforms: Vec<Platform>,
}

impl From<pickle::Value> for PublicTransport {
    fn from(value: pickle::Value) -> Self {
        PublicTransport { platforms: vec![] }
    }
}
