use std::{io::Read, slice::Iter};

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
    #[error(transparent)]
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
            let tex = parser.parse()?;
            inner_vec.push(tex);
        }

        debug_assert!(
            inner_vec.len() == size,
            "the parsed texture count does not match the expected size"
        );

        Ok(Self {
            len: size,
            inner: inner_vec,
        })
    }
}

impl Textures {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> Iter<'_, Texture> {
        self.inner.iter()
    }
}

#[derive(Debug)]
pub struct Texture {
    path: PmxText,
}

impl Texture {
    pub fn path(&self) -> &PmxText {
        &self.path
    }
}

impl PmxParseable for Texture {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, _globals: &Globals) -> Result<Self> {
        let path = parser.parse()?;

        Ok(Self { path })
    }
}
