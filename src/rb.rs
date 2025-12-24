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
    #[error("Invalid physics mode type")]
    InvalidPhysicsMode,
    #[error("Invalid shape")]
    InvalidShape,
    #[error(transparent)]
    Type(#[from] crate::types::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct RigidBodies {
    len: usize,
    inner: Vec<RigidBody>,
}

impl PmxParseable for RigidBodies {
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

        // TODO(mate): these shouldnt be debug asserts probably, just make it a regular check
        debug_assert!(
            inner_vec.len() == size,
            "the parsed rb count does not match the expected size"
        );

        Ok(Self {
            len: size,
            inner: inner_vec,
        })
    }
}

impl RigidBodies {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> Iter<'_, RigidBody> {
        self.inner.iter()
    }
}

#[derive(Debug)]
pub struct RigidBody {
    name: PmxTextGroup,
    related_bone_index: Index,
    group_id: u8,
    non_collision_group: i16,
    shape: Shape,
    shape_size: Vec3,
    shape_pos: Vec3,
    shape_rotation: Vec3,
    mass: f32,
    move_attentuation: f32,
    rotation_damping: f32,
    repulsion: f32,
    friction: f32,
    physics_mode: PhysicsMode,
}

impl RigidBody {
    pub fn name(&self) -> &PmxTextGroup {
        &self.name
    }
}

#[derive(Debug)]
pub enum Shape {
    Sphere,
    Box,
    Capsule,
}

impl TryFrom<u8> for Shape {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        let ok = match value {
            0 => Self::Sphere,
            1 => Self::Box,
            2 => Self::Capsule,
            _ => Err(Error::InvalidShape)?,
        };

        Ok(ok)
    }
}

#[derive(Debug)]
pub enum PhysicsMode {
    Bone,
    Physics,
    PhysicsBone,
}

impl TryFrom<u8> for PhysicsMode {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        let ok = match value {
            0 => Self::Bone,
            1 => Self::Physics,
            2 => Self::PhysicsBone,
            _ => Err(Error::InvalidPhysicsMode)?,
        };

        Ok(ok)
    }
}

impl PmxParseable for RigidBody {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, globals: &Globals) -> Result<Self> {
        let name = parser.parse()?;

        let reader = &mut parser.reader;

        let related_bone_index = Index::parse_bone(reader, globals.bone_idx_size)?;
        let group_id = reader.read_byte()?;
        let non_collision_group = reader.read_i16_le()?;
        let shape = reader.read_byte()?.try_into()?;
        let shape_size = f32_array_from_le_bytes!(3, reader).into();
        let shape_pos = f32_array_from_le_bytes!(3, reader).into();
        let shape_rotation = f32_array_from_le_bytes!(3, reader).into();
        let mass = reader.read_f32_le()?;
        let move_attentuation = reader.read_f32_le()?;
        let rotation_damping = reader.read_f32_le()?;
        let repulsion = reader.read_f32_le()?;
        let friction = reader.read_f32_le()?;
        let physics_mode = reader.read_byte()?.try_into()?;

        Ok(Self {
            name,
            related_bone_index,
            group_id,
            non_collision_group,
            shape,
            shape_size,
            shape_pos,
            shape_rotation,
            mass,
            move_attentuation,
            rotation_damping,
            repulsion,
            friction,
            physics_mode,
        })
    }
}
