use std::path::PathBuf;

use mmd_rs::pmx::Pmx;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let path = PathBuf::from(args[1].clone());

    let pmx = Pmx::open(&path);

    match pmx {
        Ok(pmx) => {
            dbg!(&pmx);
        }
        Err(err) => match err {
            mmd_rs::pmx::Error::InvalidTag => todo!(),
            mmd_rs::pmx::Error::InvalidGlobalCount => todo!(),
            mmd_rs::pmx::Error::Io(error) => todo!(),
            mmd_rs::pmx::Error::Vertex(error) => todo!(),
            mmd_rs::pmx::Error::Type(error) => todo!(),
            mmd_rs::pmx::Error::Surface(error) => todo!(),
            mmd_rs::pmx::Error::Texture(error) => todo!(),
            mmd_rs::pmx::Error::Material(error) => todo!(),
        },
    }
}
