use std::io::Read;

use thiserror::Error;

use crate::{
    parser::{Parser, PmxParseable},
    pmx::{Globals, Pmx},
    types::PmxText,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Negative size encountered where positive expected")]
    NegativeSize,
    #[error(transparent)]
    Type(#[from] crate::types::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Textures {
    len: usize,
    inner: Vec<Texture>,
}

impl PmxParseable for Textures {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, _globals: &Globals) -> Result<Self> {
        let mut size_bytes = [0; 4];

        let reader = &mut parser.reader;

        reader.read_exact(&mut size_bytes)?;

        let size = i32::from_le_bytes(size_bytes);

        if size.is_negative() {
            Err(Error::NegativeSize)?
        }

        let size = size as usize;

        let mut inner_vec = Vec::with_capacity(size);

        for _ in 0..size {
            let tex = parser.parse::<Texture>()?;
            inner_vec.push(tex);
        }

        Ok(Self {
            len: size,
            inner: inner_vec,
        })
    }
}

impl Textures {
    pub fn len(&self) -> usize {
        let len = self.inner.len();
        debug_assert!(self.len == len);
        len
    }
}

#[derive(Debug)]
pub struct Texture {
    path: PmxText,
}

impl PmxParseable for Texture {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, _globals: &Globals) -> Result<Self> {
        let path = parser.parse::<PmxText>()?;

        Ok(Self { path })
    }
}
