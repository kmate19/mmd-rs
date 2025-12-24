use std::{io::Read, slice::Iter};

use thiserror::Error;

use crate::{
    parser::{Parser, PmxParseable},
    pmx::{Globals, Pmx},
    types::{Index, PmxTextGroup},
    util::ReadExt,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Negative size encountered where positive expected")]
    NegativeSize,
    #[error("Invalid frame type")]
    InvalidFrameType,
    #[error(transparent)]
    Type(#[from] crate::types::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Frames {
    len: usize,
    inner: Vec<Frame>,
}

impl PmxParseable for Frames {
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

        debug_assert!(
            inner_vec.len() == size,
            "the parsed frame count does not match the expected size"
        );

        Ok(Self {
            len: size,
            inner: inner_vec,
        })
    }
}

impl Frames {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> Iter<'_, Frame> {
        self.inner.iter()
    }
}

#[derive(Debug)]
pub struct Frame {
    name: PmxTextGroup,
    special: bool,
    _frame_len: i32,
    frames: Option<Vec<FrameData>>,
}

#[derive(Debug)]
pub struct FrameData {
    _type: u8,
    pub inner: FrameDataInner,
}

#[derive(Debug)]
pub enum FrameDataInner {
    Bone(Index),
    Morph(Index),
}

impl Frame {
    pub fn name(&self) -> &PmxTextGroup {
        &self.name
    }

    pub fn special(&self) -> bool {
        self.special
    }

    pub fn frames(&self) -> Option<&Vec<FrameData>> {
        self.frames.as_ref()
    }
}

impl PmxParseable for Frame {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, globals: &Globals) -> Result<Self> {
        let name = parser.parse()?;

        let reader = &mut parser.reader;

        let special = reader.read_byte()? != 0;

        let frame_len = reader.read_i32_le()?;

        let frames = {
            if frame_len > 0 {
                let mut vec = Vec::with_capacity(frame_len as _);

                for _ in 0..frame_len {
                    let frame_type = reader.read_byte()?;

                    let data_inner = match frame_type {
                        0 => FrameDataInner::Bone(Index::create(
                            reader,
                            globals.bone_idx_size.try_into()?,
                            true,
                        )?),
                        1 => FrameDataInner::Morph(Index::create(
                            reader,
                            globals.morph_idx_size.try_into()?,
                            true,
                        )?),
                        _ => Err(Error::InvalidFrameType)?,
                    };

                    vec.push(FrameData {
                        _type: frame_type,
                        inner: data_inner,
                    })
                }
                Some(vec)
            } else {
                None
            }
        };

        Ok(Self {
            name,
            special,
            _frame_len: frame_len,
            frames,
        })
    }
}
