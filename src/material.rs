use std::io::Read;

use thiserror::Error;

use crate::{
    parser::{Parser, PmxParseable},
    pmx::{Globals, Pmx},
    types::{Flag, Index, PmxText, Vec3, Vec4, f32_array_from_le_bytes},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Negative size encountered where positive expected")]
    NegativeSize,
    #[error(transparent)]
    Type(#[from] crate::types::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid blend mode value")]
    InvalidBlendMode,
    #[error("Invalid toon reference value")]
    InvalidToonRef,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Materials {
    len: usize,
    inner: Vec<Material>,
}

impl PmxParseable for Materials {
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
            let mat = parser.parse::<Material>()?;
            inner_vec.push(mat);
        }

        Ok(Self {
            len: size,
            inner: inner_vec,
        })
    }
}

impl Materials {
    pub fn len(&self) -> usize {
        let len = self.inner.len();
        debug_assert!(self.len == len);
        len
    }
}

#[derive(Debug)]
pub struct Material {
    name: Name,
    diffuse: Vec4,
    specular: Vec3,
    specular_strength: f32,
    ambient: Vec3,
    flags: Flag,
    edge_color: Vec4,
    edge_scale: f32,
    tex_idx: Index,
    env_idx: Index,
    env_blend: EnvBlendMode,
    toon: Toon,
    meta: PmxText,
    surface_count: i32,
}

#[derive(Debug)]
pub enum EnvBlendMode {
    None,
    Multiply,
    Add,
    // Environment blend mode 3 will use the first additional vec4 to map the environment
    // texture, using just the X and Y values as the texture UV.
    // It is mapped as an additional texture layer.
    // This may conflict with other uses for the first additional vec4.
    Additional,
}

impl TryFrom<u8> for EnvBlendMode {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(EnvBlendMode::None),
            1 => Ok(EnvBlendMode::Multiply),
            2 => Ok(EnvBlendMode::Add),
            3 => Ok(EnvBlendMode::Additional),
            _ => Err(Error::InvalidBlendMode),
        }
    }
}

pub enum ToonRef {
    Texture,
    Internal,
}

impl TryFrom<u8> for ToonRef {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(ToonRef::Texture),
            1 => Ok(ToonRef::Internal),
            _ => Err(Error::InvalidToonRef),
        }
    }
}

// Toon value will be a texture index much like the standard texture and environment texture
// indexes unless the Toon reference byte is equal to 1,
// in which case Toon value will be a byte that references a set of 10 internal toon textures
// (Most implementations will use "toon01.bmp" to "toon10.bmp" as the internal textures,
// see the reserved names for Textures above).
#[derive(Debug)]
pub enum Toon {
    Texture(Index),
    Internal(u8),
}

#[derive(Debug)]
struct Name {
    local: PmxText,
    universal: PmxText,
}

impl PmxParseable for Material {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, globals: &Globals) -> Result<Self> {
        let name = {
            let local = parser.parse::<PmxText>()?;
            let universal = parser.parse::<PmxText>()?;
            Name { local, universal }
        };

        let (diffuse, specular, specular_strength, ambient) = {
            let reader = &mut parser.reader;

            let diffuse: Vec4 = f32_array_from_le_bytes!(4, reader).into();
            let specular: Vec3 = f32_array_from_le_bytes!(3, reader).into();

            let specular_strength = {
                let mut buf = [0; 4];
                reader.read_exact(&mut buf)?;
                f32::from_le_bytes(buf)
            };

            let ambient: Vec3 = f32_array_from_le_bytes!(3, reader).into();

            (diffuse, specular, specular_strength, ambient)
        };

        let flags = parser.parse::<Flag>()?;

        let (edge_color, edge_scale, tex_idx, env_idx, env_blend, toon) = {
            let reader = &mut parser.reader;

            let edge_color: Vec4 = f32_array_from_le_bytes!(4, reader).into();

            let edge_scale = {
                let mut buf = [0; 4];
                reader.read_exact(&mut buf)?;
                f32::from_le_bytes(buf)
            };

            let tex_idx = Index::create(reader, globals.tex_idx_size.try_into()?, true)?;

            let env_idx = Index::create(reader, globals.tex_idx_size.try_into()?, true)?;

            let env_blend = {
                let mut buf = [0; 1];
                reader.read_exact(&mut buf)?;
                EnvBlendMode::try_from(buf[0])?
            };

            let toon_ref = {
                let mut buf = [0; 1];
                reader.read_exact(&mut buf)?;
                ToonRef::try_from(buf[0])?
            };

            let toon = match toon_ref {
                ToonRef::Texture => {
                    let toon_idx = Index::create(reader, globals.tex_idx_size.try_into()?, true)?;
                    Toon::Texture(toon_idx)
                }
                ToonRef::Internal => {
                    let mut buf = [0; 1];
                    reader.read_exact(&mut buf)?;
                    Toon::Internal(buf[0])
                }
            };

            (edge_color, edge_scale, tex_idx, env_idx, env_blend, toon)
        };

        let meta = parser.parse::<PmxText>()?;

        let reader = &mut parser.reader;

        let surface_count = {
            let mut buf = [0; 4];
            reader.read_exact(&mut buf)?;
            i32::from_le_bytes(buf)
        };

        Ok(Self {
            name,
            diffuse,
            specular,
            specular_strength,
            ambient,
            flags,
            edge_color,
            edge_scale,
            tex_idx,
            env_idx,
            env_blend,
            toon,
            meta,
            surface_count,
        })
    }
}
