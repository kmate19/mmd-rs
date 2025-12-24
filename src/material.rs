use std::{io::Read, slice::Iter};

use thiserror::Error;

use crate::util::f32_array_from_le_bytes;
use crate::{Vec3, Vec4};

use crate::{
    parser::{Parser, PmxParseable},
    pmx::{Globals, Pmx},
    types::{Flag, Index, PmxText, PmxTextGroup},
    util::ReadExt,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Negative size encountered where positive expected")]
    NegativeSize,
    #[error("Invalid blend mode value")]
    InvalidBlendMode,
    #[error("Invalid toon reference value")]
    InvalidToonRef,
    #[error(transparent)]
    Type(#[from] crate::types::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
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
        let size = parser.reader.read_i32_le()?;

        if size.is_negative() {
            Err(Error::NegativeSize)?
        }

        let size = size as usize;

        let mut inner_vec = Vec::with_capacity(size);

        for _ in 0..size {
            let mat = parser.parse()?;
            inner_vec.push(mat);
        }

        debug_assert!(
            inner_vec.len() == size,
            "the parsed material count does not match the expected size"
        );

        Ok(Self {
            len: size,
            inner: inner_vec,
        })
    }
}

impl Materials {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> Iter<'_, Material> {
        self.inner.iter()
    }
}

#[derive(Debug)]
pub struct Material {
    name: PmxTextGroup,
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
    toon_ref: ToonRef,
    toon: Toon,
    meta: PmxText,
    surface_count: i32,
}

impl PmxParseable for Material {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, globals: &Globals) -> Result<Self> {
        let name = parser.parse()?;

        let (diffuse, specular, specular_strength, ambient) = {
            let reader = &mut parser.reader;

            let diffuse: Vec4 = f32_array_from_le_bytes!(4, reader).into();
            let specular: Vec3 = f32_array_from_le_bytes!(3, reader).into();
            let specular_strength = reader.read_f32_le()?;
            let ambient: Vec3 = f32_array_from_le_bytes!(3, reader).into();

            (diffuse, specular, specular_strength, ambient)
        };

        let flags = parser.parse()?;

        let (edge_color, edge_scale, tex_idx, env_idx, env_blend, toon_ref, toon) = {
            let reader = &mut parser.reader;

            let edge_color: Vec4 = f32_array_from_le_bytes!(4, reader).into();
            let edge_scale = reader.read_f32_le()?;
            let tex_idx = Index::parse_texture(reader, globals.tex_idx_size)?;
            let env_idx = Index::parse_texture(reader, globals.tex_idx_size)?;
            let env_blend = reader.read_byte()?.try_into()?;
            let toon_ref = reader.read_byte()?.try_into()?;
            let toon = match toon_ref {
                ToonRef::Texture => {
                    let toon_idx = Index::parse_texture(reader, globals.tex_idx_size)?;
                    Toon::Texture(toon_idx)
                }
                ToonRef::Internal => Toon::Internal(reader.read_byte()?),
            };

            (
                edge_color, edge_scale, tex_idx, env_idx, env_blend, toon_ref, toon,
            )
        };

        let meta = parser.parse()?;

        let surface_count = parser.reader.read_i32_le()?;

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
            toon_ref,
            toon,
            meta,
            surface_count,
        })
    }
}

impl Material {
    pub fn env_blend(&self) -> &EnvBlendMode {
        &self.env_blend
    }

    pub fn name(&self) -> &PmxTextGroup {
        &self.name
    }

    pub fn diffuse(&self) -> Vec4 {
        self.diffuse
    }

    pub fn specular(&self) -> Vec3 {
        self.specular
    }

    pub fn specular_strength(&self) -> f32 {
        self.specular_strength
    }

    pub fn ambient(&self) -> Vec3 {
        self.ambient
    }

    pub fn flags(&self) -> &Flag {
        &self.flags
    }

    pub fn edge_color(&self) -> Vec4 {
        self.edge_color
    }

    pub fn edge_scale(&self) -> f32 {
        self.edge_scale
    }

    pub fn tex_idx(&self) -> &Index {
        &self.tex_idx
    }

    pub fn env_idx(&self) -> &Index {
        &self.env_idx
    }

    pub fn toon(&self) -> &Toon {
        &self.toon
    }

    pub fn meta(&self) -> &PmxText {
        &self.meta
    }

    pub fn surface_count(&self) -> i32 {
        self.surface_count
    }

    pub fn toon_ref(&self) -> &ToonRef {
        &self.toon_ref
    }
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

#[derive(Debug)]
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
