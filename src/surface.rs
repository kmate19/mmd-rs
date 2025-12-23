use std::io::Read;

use thiserror::Error;

use crate::{
    parser::{Parser, PmxParseable},
    pmx::{Globals, Pmx},
    types::Index,
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
pub struct Surfaces {
    len: usize,
    inner: Vec<Surface>,
}

impl PmxParseable for Surfaces {
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
            let surf = parser.parse::<Surface>()?;
            inner_vec.push(surf);
        }

        Ok(Self {
            len: size,
            inner: inner_vec,
        })
    }
}

impl Surfaces {
    pub fn len(&self) -> usize {
        let len = self.inner.len();
        debug_assert!(self.len == len);
        len
    }
}

#[derive(Debug)]
pub struct Surface {
    index: Index,
}

impl PmxParseable for Surface {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, globals: &Globals) -> Result<Self> {
        // NOTE(mate): not implementing our usual trait and pattern here because this type is simply a nightmare to generalize over
        let index = Index::create(&mut parser.reader, globals.vert_idx_size.try_into()?, false)?;

        Ok(Self { index })
    }
}
