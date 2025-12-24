use std::{io::Read, slice::Iter};

use thiserror::Error;

use crate::{
    parser::{Parser, PmxParseable},
    pmx::{Globals, Pmx},
    types::Index,
    util::ReadExt,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Negative size encountered where positive expected")]
    NegativeSize,
    #[error("Invalid surface size: must be a multiple of 3 and must match the advertised count")]
    InvalidSize,
    #[error(transparent)]
    Type(#[from] crate::types::Error),
    #[error(transparent)]
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
        let size = parser.reader.read_i32_le()?;

        if size.is_negative() {
            Err(Error::NegativeSize)?
        }

        let size = size as usize;

        let mut inner_vec = Vec::with_capacity(size);

        for _ in 0..size {
            let surf = parser.parse()?;
            inner_vec.push(surf);
        }

        if inner_vec.len() % 3 != 0 || inner_vec.len() != size {
            Err(Error::InvalidSize)?; // or some other appropriate error
        }

        Ok(Self {
            len: size,
            inner: inner_vec,
        })
    }
}

impl Surfaces {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> Iter<'_, Surface> {
        self.inner.iter()
    }
}

#[derive(Debug)]
pub struct Surface {
    index: Index,
}

impl Surface {
    pub fn as_index(&self) -> i32 {
        self.index.value()
    }
}

impl PmxParseable for Surface {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, globals: &Globals) -> Result<Self> {
        let index = Index::parse_vertex(&mut parser.reader, globals.vert_idx_size)?;

        Ok(Self { index })
    }
}
