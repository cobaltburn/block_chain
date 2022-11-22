extern crate core;

use std::env;
use std::fs::File;
use crate::chain::Chain;

mod chain;

fn main() {
    let mut args = env::args();
    let mut file_name: String = String::from("");
    let mut chain: Chain;
    while let Some(arg) = args.next() {
        if arg == "~f" {
            file_name = args.next().expect("No argument found");
        }
    }

    let blocks_added= if file_name != "" {
        let file = File::open(file_name).expect("Invalid file argument");
        chain = Chain::from(file);
        10
    } else {
        chain = Chain::new();
        9
    };
    for _ in 0..blocks_added {
        chain.add_block();
    }
}
