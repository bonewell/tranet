use std::{env, process::exit};

mod map;
mod platforms;
mod raptor;
mod reader;

use crate::raptor::Raptor;
use crate::reader::{read_map, read_points};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        println!("Usage: pickle [map] [points]");
        exit(1);
    }

    let map = read_map(&args[1]);
    let raptor = Raptor::new(&map);
    for (from, to) in read_points(&args[2]) {
        let paths = raptor.find_path(&from, &to);
        println!("{:?}", paths);
    }
}
