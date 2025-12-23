use std::path::PathBuf;

use mmd_rs::pmx::Pmx;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_pmx_file>", args[0]);
        std::process::exit(1);
    }

    let path = PathBuf::from(args[1].clone());

    let pmx = Pmx::open(&path).unwrap();

    dbg!(&pmx);
}
