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

    #[cfg(debug_assertions)]
    {
        dbg!(&pmx);
        #[cfg(feature = "ui")]
        mmd_rs::ui::start(std::sync::Arc::new(pmx));
    }
}
