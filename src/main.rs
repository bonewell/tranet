use std::{env, process::exit};

mod map;
mod path;
mod platforms;
mod raptor;
mod reader;
mod searcher;

use crate::raptor::Raptor;
use crate::reader::{read_map, read_points};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        println!("Usage: tranet [map] [points]");
        exit(1);
    }

    let raptor = Raptor::new(read_map(&args[1]));
    for (start, finish) in read_points(&args[2]) {
        for path in raptor.find_path(start, finish) {
            println!("{}", path);
        }
        println!();
    }
}
