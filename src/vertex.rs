use std::io::Read;

use thiserror::Error;

use crate::{
    parser::{Parser, PmxParseable},
    pmx::{Globals, Pmx},
    types::{Index, IndexSize, Vec2, Vec3, Vec4, f32_array_from_le_bytes},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("The index size mismatched")]
    IndexSizeMismatch,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Negative size encountered where positive expected")]
    NegativeSize,
    #[error("Invalid weight deform type encountered")]
    InvalidWeightDeformType,
    #[error(transparent)]
    Type(#[from] crate::types::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Vertices {
    inner: Vec<Vertex>,
    size: usize,
}

impl PmxParseable for Vertices {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, _globals: &Globals) -> Result<Self> {
        let mut size = [0; 4];

        let reader = &mut parser.reader;

        reader.read_exact(&mut size)?;

        let size = i32::from_le_bytes(size);

        if size.is_negative() {
            Err(Error::NegativeSize)?
        }

        let size = size as usize;

        let mut inner_vec = Vec::with_capacity(size);

        for _ in 0..size {
            let vert = parser.parse::<Vertex>()?;
            inner_vec.push(vert);
        }

        Ok(Self {
            inner: inner_vec,
            size,
        })
    }
}

impl Vertices {
    pub fn len(&self) -> usize {
        let len = self.inner.len();
        debug_assert!(self.size == len);
        len
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.inner
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

        let mut weight_deform_type = [0; 1];

        reader.read_exact(&mut weight_deform_type)?;

        let size: IndexSize = globals.vert_idx_size.try_into()?;

        let weight_deform = WeightDeform::create(reader, weight_deform_type[0], size, true)?;

        let mut edge_scale = [0; 4];

        reader.read_exact(&mut edge_scale)?;

        let edge_scale = f32::from_le_bytes(edge_scale);

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
    pub fn create(
        reader: &mut impl Read,
        typ: u8,
        size: IndexSize,
        index_sign: bool,
    ) -> Result<Self> {
        match typ {
            0 => {
                let index = Index::create(reader, size, index_sign)?;

                Ok(WeightDeform::Bdef1 { index })
            }
            1 => {
                let indices = [
                    Index::create(reader, size, index_sign)?,
                    Index::create(reader, size, index_sign)?,
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
                    Index::create(reader, size, index_sign)?,
                    Index::create(reader, size, index_sign)?,
                    Index::create(reader, size, index_sign)?,
                    Index::create(reader, size, index_sign)?,
                ];

                let weights = f32_array_from_le_bytes!(4, reader);

                Ok(WeightDeform::Bdef4 { indices, weights })
            }
            3 => {
                let indices = [
                    Index::create(reader, size, index_sign)?,
                    Index::create(reader, size, index_sign)?,
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
                    Index::create(reader, size, index_sign)?,
                    Index::create(reader, size, index_sign)?,
                    Index::create(reader, size, index_sign)?,
                    Index::create(reader, size, index_sign)?,
                ];

                let weights = f32_array_from_le_bytes!(4, reader);

                Ok(WeightDeform::Qdef { indices, weights })
            }
            _ => Err(Error::InvalidWeightDeformType)?,
        }
    }
}
