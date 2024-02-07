use std::{env, fs::File, io::Read, process::exit};

use serde_pickle as pickle;

pub mod types;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Usage: pickle [filename]");
        exit(1);
    }

    let reader: Box<dyn Read> = Box::new(File::open(&args[1]).expect("filename is not specified"));
    let decoded: pickle::Value =
        pickle::value_from_reader(reader, Default::default()).expect("Can not parse file");
    let tranet: types::PublicTransport = decoded.into();
    print!("{:?}", tranet);
}
