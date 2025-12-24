use std::{io::Read, slice::Iter};

use thiserror::Error;

use crate::{
    parser::{Parser, PmxParseable},
    pmx::{Globals, Pmx},
    types::{Index, PmxTextGroup},
    util::{ReadExt, f32_array_from_le_bytes},
};

use crate::Vec3;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Negative size encountered where positive expected")]
    NegativeSize,
    #[error("Invalid joint type")]
    InvalidJointType,
    #[error(transparent)]
    Type(#[from] crate::types::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Joints {
    len: usize,
    inner: Vec<Joint>,
}

impl PmxParseable for Joints {
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
            "the parsed surface count does not match the expected size"
        );

        Ok(Self {
            len: size,
            inner: inner_vec,
        })
    }
}

impl Joints {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> Iter<'_, Joint> {
        self.inner.iter()
    }
}

#[derive(Debug)]
pub struct Joint {
    name: PmxTextGroup,
    typ: JointType,
    rb_index_a: Index,
    rb_index_b: Index,
    pos: Vec3,
    rotation: Vec3,
    pos_min: Vec3,
    pos_max: Vec3,
    rotation_min: Vec3,
    rotation_max: Vec3,
    spring_pos: Vec3,
    spring_rotation: Vec3,
}

#[derive(Debug)]
pub enum JointType {
    SpringSixDOF,
    SixDOF,
    P2P,
    ConeTwist,
    Slider,
    Hinge,
}

impl TryFrom<u8> for JointType {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        let ok = match value {
            0 => Self::SpringSixDOF,
            1 => Self::SixDOF,
            2 => Self::P2P,
            3 => Self::ConeTwist,
            4 => Self::Slider,
            5 => Self::Hinge,
            _ => Err(Error::InvalidJointType)?,
        };

        Ok(ok)
    }
}

impl PmxParseable for Joint {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, globals: &Globals) -> Result<Self> {
        let name = parser.parse()?;

        let reader = &mut parser.reader;

        let typ = reader.read_byte()?.try_into()?;
        let rb_index_a = Index::create(reader, globals.rb_idx_size.try_into()?, true)?;
        let rb_index_b = Index::create(reader, globals.rb_idx_size.try_into()?, true)?;
        let pos = f32_array_from_le_bytes!(3, reader).into();
        let rotation = f32_array_from_le_bytes!(3, reader).into();
        let pos_min = f32_array_from_le_bytes!(3, reader).into();
        let pos_max = f32_array_from_le_bytes!(3, reader).into();
        let rotation_min = f32_array_from_le_bytes!(3, reader).into();
        let rotation_max = f32_array_from_le_bytes!(3, reader).into();
        let spring_pos = f32_array_from_le_bytes!(3, reader).into();
        let spring_rotation = f32_array_from_le_bytes!(3, reader).into();

        Ok(Self {
            name,
            typ,
            rb_index_a,
            rb_index_b,
            pos,
            rotation,
            pos_min,
            pos_max,
            rotation_min,
            rotation_max,
            spring_pos,
            spring_rotation,
        })
    }
}
