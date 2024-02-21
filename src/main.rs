use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Lines, Read},
    process::exit,
};

use geo_types::Point;
use serde_pickle as pickle;
use wkt::TryFromWkt;

mod map;
mod platforms;
mod reader;

use crate::map::PublicTransport;
use crate::platforms::Platforms;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        println!("Usage: pickle [map] [points]");
        exit(1);
    }

    let map = read_map(&args[1]);
    for line in read_points(&args[2]).flatten() {
        let (from, to) = parse_points(&line);
        let platforms = Platforms::new(&map.platforms, Platforms::zone(&from));
        let from = platforms.find(&from);
        let to = platforms.find(&to);
        println!("{:?} -> {:?}", from, to);
        // let paths = find_path(from, to);
    }
}

fn read_map(filename: &String) -> PublicTransport {
    let reader: Box<dyn Read> = Box::new(File::open(&filename).expect("Can not open map"));
    let decoded: pickle::Value =
        pickle::value_from_reader(reader, Default::default()).expect("Can not parse map");
    (&decoded).into()
}

fn read_points(filename: &String) -> Lines<BufReader<File>> {
    let file = File::open(filename).expect("Can not open points");
    BufReader::new(file).lines()
}

fn parse_points(line: &String) -> (Point<f64>, Point<f64>) {
    let points: Vec<&str> = line.split(',').collect();
    (
        Point::try_from_wkt_str(points[0]).unwrap(),
        Point::try_from_wkt_str(points[1]).unwrap(),
    )
}
