use std::{io::Read, slice::Iter};

use thiserror::Error;

use crate::Vec3;
use crate::types::{Flag, Index};
use crate::util::f32_array_from_le_bytes;

use crate::{
    parser::{Parser, PmxParseable},
    pmx::{Globals, Pmx},
    types::PmxTextGroup,
    util::ReadExt,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Negative size encountered where positive expected")]
    NegativeSize,
    #[error(transparent)]
    Type(#[from] crate::types::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Bones {
    len: usize,
    inner: Vec<Bone>,
}

impl PmxParseable for Bones {
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
            "the parsed bone count does not match the expected size"
        );

        Ok(Self {
            len: size,
            inner: inner_vec,
        })
    }
}

impl Bones {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> Iter<'_, Bone> {
        self.inner.iter()
    }
}

#[repr(u8)]
pub enum BoneFlag {
    IndexedTailPos,
    Rotatable,
    Translatable,
    IsVisible,
    Enabled,
    IK,
}

#[repr(u8)]
pub enum InheritFlag {
    InheritRotation,
    InheritTranslation,
    FixedAxis,
    LocalCoordinate,
    PhysicsAfterDeform,
    ExternalParentDeform,
}

#[derive(Debug)]
pub struct Bone {
    name: PmxTextGroup,
    pos: Vec3,
    parent: Index,
    layer: i32,
    /// Bone flags first array
    /// Second array is inherit flags
    flags: [Flag; 2],
    tail_pos: TailPos,
    inherit: Option<InheritBone>,
    fixed_axis: Option<FixedAxisBone>,
    local: Option<LocalBone>,
    external_parent: Option<Index>,
    ik: Option<BoneIK>,
}

impl Bone {
    pub fn has_bone_flag(&self, flag: BoneFlag) -> bool {
        self.flags[0].get_state(flag as u8)
    }

    pub fn has_inherit_flag(&self, flag: InheritFlag) -> bool {
        self.flags[1].get_state(flag as u8)
    }

    pub fn name(&self) -> &PmxTextGroup {
        &self.name
    }

    pub fn pos(&self) -> Vec3 {
        self.pos
    }

    pub fn parent(&self) -> &Index {
        &self.parent
    }

    pub fn layer(&self) -> i32 {
        self.layer
    }

    pub fn flags(&self) -> &[Flag; 2] {
        &self.flags
    }

    pub fn tail_pos(&self) -> &TailPos {
        &self.tail_pos
    }

    pub fn inherit(&self) -> Option<&InheritBone> {
        self.inherit.as_ref()
    }

    pub fn fixed_axis(&self) -> Option<&FixedAxisBone> {
        self.fixed_axis.as_ref()
    }

    pub fn local(&self) -> Option<&LocalBone> {
        self.local.as_ref()
    }

    pub fn external_parent(&self) -> Option<&Index> {
        self.external_parent.as_ref()
    }

    pub fn ik(&self) -> Option<&BoneIK> {
        self.ik.as_ref()
    }
}

#[derive(Debug)]
pub struct InheritBone {
    pub parent: Index,
    pub parent_influence: f32,
}

#[derive(Debug)]
pub struct FixedAxisBone {
    pub axis: Vec3,
}

#[derive(Debug)]
pub struct LocalBone {
    pub x: Vec3,
    pub z: Vec3,
}

#[derive(Debug)]
pub struct BoneIK {
    pub target: Index,
    pub loop_count: i32,
    pub limit_rad: f32,
    // This field is only so that we remember to parse it; it could be inferred from the links option.
    _link_count: i32,
    pub links: Option<Vec<BoneIKLink>>,
}

#[derive(Debug)]
pub struct BoneIKLink {
    pub index: Index,
    // This field is only so that we remember to parse it; it could be inferred from the limit option.
    _limits: bool,
    pub limit: Option<IKLimit>,
}

#[derive(Debug)]
pub struct IKLimit {
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Debug)]
pub enum TailPos {
    Offset(Vec3),
    Bone(Index),
}

impl PmxParseable for Bone {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, globals: &Globals) -> Result<Self> {
        let name = parser.parse()?;

        let (pos, parent, layer) = {
            let reader = &mut parser.reader;
            let pos = f32_array_from_le_bytes!(3, reader).into();
            let parent = Index::parse_bone(reader, globals.bone_idx_size)?;
            let layer = reader.read_i32_le()?;

            (pos, parent, layer)
        };

        // TODO(mate): just use bitflags
        // Bone Flags
        // Bones can have the following bit flags:

        // Bit index	Name	Set effect	Version
        // 0	Indexed tail position	Is the tail position a vec3 or bone index	2.0
        // 1	Rotatable	Enables rotation	2.0
        // 2	Translatable	Enables translation (shear)	2.0
        // 3	Is visible	???	2.0
        // 4	Enabled	???	2.0
        // 5	IK	Use inverse kinematics (physics)	2.0
        // --------- 2. array ---------
        // 0	Inherit rotation	Rotation inherits from another bone	2.0
        // 1	Inherit translation	Translation inherits from another bone	2.0
        // 2	Fixed axis	The bone's shaft is fixed in a direction	2.0
        // 3	Local co-ordinate	???	2.0
        // 4	Physics after deform	???	2.0
        // 5	External parent deform	???	2.0
        let flags: [Flag; 2] = [parser.parse()?, parser.parse()?];

        let reader = &mut parser.reader;

        let tail_pos = if flags[0].get_state(0) {
            TailPos::Bone(Index::parse_bone(reader, globals.bone_idx_size)?)
        } else {
            TailPos::Offset(f32_array_from_le_bytes!(3, reader).into())
        };

        let inherit = if flags[1].get_state(0) || flags[1].get_state(1) {
            Some(InheritBone {
                parent: Index::parse_bone(reader, globals.bone_idx_size)?,
                parent_influence: reader.read_f32_le()?,
            })
        } else {
            None
        };

        let fixed_axis = if flags[1].get_state(2) {
            Some(FixedAxisBone {
                axis: f32_array_from_le_bytes!(3, reader).into(),
            })
        } else {
            None
        };

        let local = if flags[1].get_state(3) {
            Some(LocalBone {
                x: f32_array_from_le_bytes!(3, reader).into(),
                z: f32_array_from_le_bytes!(3, reader).into(),
            })
        } else {
            None
        };

        let external_parent = if flags[1].get_state(5) {
            Some(Index::parse_bone(reader, globals.bone_idx_size)?)
        } else {
            None
        };

        let ik = if flags[0].get_state(5) {
            let target = Index::parse_bone(reader, globals.bone_idx_size)?;
            let loop_count = reader.read_i32_le()?;
            let limit_rad = reader.read_f32_le()?;
            let link_count = reader.read_i32_le()?;

            let links = if link_count > 0 {
                let mut v = Vec::with_capacity(link_count as _);

                for _ in 0..link_count {
                    let index = Index::parse_bone(reader, globals.bone_idx_size)?;

                    let limits = reader.read_byte()? != 0;

                    let limit = if limits {
                        Some(IKLimit {
                            min: f32_array_from_le_bytes!(3, reader).into(),
                            max: f32_array_from_le_bytes!(3, reader).into(),
                        })
                    } else {
                        None
                    };
                    v.push(BoneIKLink {
                        index,
                        _limits: limits,
                        limit,
                    });
                }
                Some(v)
            } else {
                None
            };
            Some(BoneIK {
                target,
                loop_count,
                limit_rad,
                _link_count: link_count,
                links,
            })
        } else {
            None
        };

        Ok(Self {
            name,
            pos,
            parent,
            layer,
            flags,
            inherit,
            external_parent,
            fixed_axis,
            tail_pos,
            local,
            ik,
        })
    }
}
