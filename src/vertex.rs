use std::{io::Read, slice::Iter};

use thiserror::Error;

use crate::{Vec2, Vec3, Vec4};

use crate::{
    parser::{Parser, PmxParseable},
    pmx::{Globals, Pmx},
    types::{Index, IndexSize},
    util::{ReadExt, f32_array_from_le_bytes},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("The index size mismatched")]
    IndexSizeMismatch,
    #[error("Negative size encountered where positive expected")]
    NegativeSize,
    #[error("Invalid weight deform type encountered")]
    InvalidWeightDeformType,
    #[error(transparent)]
    Type(#[from] crate::types::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Vertices {
    inner: Vec<Vertex>,
    len: usize,
}

// TODO(mate): lot of repeating on these container types, theyre basically all the same code
impl PmxParseable for Vertices {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, _globals: &Globals) -> Result<Self> {
        let size = parser.reader.read_i32_le()?;

        if size.is_negative() {
            Err(Error::NegativeSize)?
        }

        let size = size as usize;

        let mut inner_vec = Vec::with_capacity(size);

        for _ in 0..size {
            let vert = parser.parse()?;
            inner_vec.push(vert);
        }

        debug_assert!(
            inner_vec.len() == size,
            "the parsed vertex count does not match the expected size"
        );

        Ok(Self {
            inner: inner_vec,
            len: size,
        })
    }
}

impl Vertices {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> Iter<'_, Vertex> {
        self.inner.iter()
    }
}

#[derive(Debug)]
pub struct Vertex {
    pos: Vec3,
    normal: Vec3,
    uv: Vec2,
    extra_vec4: Option<Vec<Vec4>>,
    weight_deform: WeightDeform,
    edge_scale: f32,
}

impl Vertex {
    pub fn pos(&self) -> Vec3 {
        self.pos
    }

    pub fn normal(&self) -> Vec3 {
        self.normal
    }

    pub fn uv(&self) -> Vec2 {
        self.uv
    }

    pub fn extra_vec4(&self) -> Option<&Vec<Vec4>> {
        self.extra_vec4.as_ref()
    }

    pub fn weight_deform(&self) -> &WeightDeform {
        &self.weight_deform
    }

    pub fn edge_scale(&self) -> f32 {
        self.edge_scale
    }
}

impl PmxParseable for Vertex {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, globals: &Globals) -> Result<Self> {
        let reader = &mut parser.reader;

        let pos = f32_array_from_le_bytes!(3, reader).into();

        let normal = f32_array_from_le_bytes!(3, reader).into();

        let uv = f32_array_from_le_bytes!(2, reader).into();

        let vec4s = if globals.vec4_additional != 0 {
            let mut v = Vec::with_capacity(globals.vec4_additional as _);

            for _ in 0..globals.vec4_additional {
                v.push(f32_array_from_le_bytes!(4, reader).into());
            }

            Some(v)
        } else {
            None
        };

        let weight_deform_type = reader.read_byte()?;

        let weight_deform =
            WeightDeform::create(reader, weight_deform_type, globals.bone_idx_size)?;

        let edge_scale = reader.read_f32_le()?;

        Ok(Self {
            pos,
            normal,
            uv,
            extra_vec4: vec4s,
            weight_deform,
            edge_scale,
        })
    }
}

#[derive(Debug)]
pub enum WeightDeform {
    // ver 2.0
    Bdef1 {
        index: Index,
    },
    // ver 2.0
    Bdef2 {
        indices: [Index; 2],
        // Only 1 actual weight is stored in the file, the other is calculated from it
        weights: [f32; 2],
    },
    // ver 2.0
    Bdef4 {
        indices: [Index; 4],
        weights: [f32; 4],
    },
    /// Spherical deform blending
    // ver 2.0
    Sdef {
        indices: [Index; 2],
        // Only 1 actual weight is stored in the file, the other is calculated from it
        weights: [f32; 2],
        // these fields are unsure?
        c: Vec3,
        r0: Vec3,
        r1: Vec3,
    },
    /// Dual quaternion deform blending
    // unsure if this is correct also
    // ver 2.1
    Qdef {
        indices: [Index; 4],
        weights: [f32; 4],
    },
}

impl WeightDeform {
    fn create(reader: &mut impl Read, typ: u8, bone_index_size: IndexSize) -> Result<Self> {
        match typ {
            0 => {
                let index = Index::parse_bone(reader, bone_index_size)?;

                Ok(WeightDeform::Bdef1 { index })
            }
            1 => {
                let indices = [
                    Index::parse_bone(reader, bone_index_size)?,
                    Index::parse_bone(reader, bone_index_size)?,
                ];

                let mut weights = [0.0; 2];

                // We only need to read 4 bytes here, because the 2nd weight is not in the file
                // but calculated from the first weight
                let mut weights_bytes = [0; std::mem::size_of::<f32>()];
                reader.read_exact(&mut weights_bytes)?;

                let chunks = weights_bytes.as_chunks::<4>().0;

                weights[0] = f32::from_le_bytes(chunks[0]);
                weights[1] = 1.0 - weights[0];

                Ok(WeightDeform::Bdef2 { indices, weights })
            }
            2 => {
                let indices = [
                    Index::parse_bone(reader, bone_index_size)?,
                    Index::parse_bone(reader, bone_index_size)?,
                    Index::parse_bone(reader, bone_index_size)?,
                    Index::parse_bone(reader, bone_index_size)?,
                ];

                let weights = f32_array_from_le_bytes!(4, reader);

                Ok(WeightDeform::Bdef4 { indices, weights })
            }
            3 => {
                let indices = [
                    Index::parse_bone(reader, bone_index_size)?,
                    Index::parse_bone(reader, bone_index_size)?,
                ];

                let mut weights = [0.0; 2];
                // We only need to read 4 bytes here, because the 2nd weight is not in the file
                // but calculated from the first weight
                let mut weights_bytes = [0; std::mem::size_of::<f32>()];
                reader.read_exact(&mut weights_bytes)?;

                let chunks = weights_bytes.as_chunks::<4>().0;

                weights[0] = f32::from_le_bytes(chunks[0]);
                weights[1] = 1.0 - weights[0];

                let mut c = Vec3::default();
                let mut r0 = Vec3::default();
                let mut r1 = Vec3::default();

                for i in 0..3 {
                    let vec: Vec3 = f32_array_from_le_bytes!(3, reader).into();

                    match i {
                        0 => c = vec,
                        1 => r0 = vec,
                        2 => r1 = vec,
                        _ => unreachable!(),
                    }
                }

                Ok(WeightDeform::Sdef {
                    indices,
                    weights,
                    c,
                    r0,
                    r1,
                })
            }
            4 => {
                let indices = [
                    Index::parse_bone(reader, bone_index_size)?,
                    Index::parse_bone(reader, bone_index_size)?,
                    Index::parse_bone(reader, bone_index_size)?,
                    Index::parse_bone(reader, bone_index_size)?,
                ];

                let weights = f32_array_from_le_bytes!(4, reader);

                Ok(WeightDeform::Qdef { indices, weights })
            }
            _ => Err(Error::InvalidWeightDeformType)?,
        }
    }
}
