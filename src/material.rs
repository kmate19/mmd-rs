use std::io::Read;

use thiserror::Error;

use crate::{
    parser::{Parser, PmxParseable},
    pmx::{Globals, Pmx},
    types::{Flag, Index, PmxText, Vec3, Vec4, vec_from_bytes},
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
    env_blend: EnvironmentBlend,
    toon: Toon,
    meta: PmxText,
    surface_count: i32,
}

#[derive(Debug)]
pub enum Toon {
    Texture(Index),
    Internal(u8),
}

#[derive(Debug)]
pub enum EnvironmentBlend {
    None,
    Multiply,
    Add,
    Additional(Vec4),
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

        let reader = &mut parser.reader;

        let diffuse: Vec4 = vec_from_bytes!(Vec4, reader);
        let specular: Vec3 = vec_from_bytes!(Vec3, reader);

        unimplemented!();
    }
}
