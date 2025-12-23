use std::{io::Read, slice::Iter};

use thiserror::Error;

use crate::util::f32_array_from_le_bytes;
use crate::{Vec3, Vec4};

use crate::{
    parser::{Parser, PmxParseable},
    pmx::{Globals, Pmx},
    types::{Index, PmxTextGroup},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Negative size encountered where positive expected")]
    NegativeSize,
    #[error("Invalid morph type encountered")]
    InvalidMorphType,
    #[error(transparent)]
    Type(#[from] crate::types::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Morphs {
    len: usize,
    inner: Vec<Morph>,
}

impl PmxParseable for Morphs {
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
            let surf = parser.parse()?;
            inner_vec.push(surf);
        }

        debug_assert!(
            inner_vec.len() == size,
            "the parsed Morph count does not match the expected size"
        );

        Ok(Self {
            len: size,
            inner: inner_vec,
        })
    }
}

impl Morphs {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> Iter<'_, Morph> {
        self.inner.iter()
    }
}

#[derive(Debug)]
pub struct Morph {
    name: PmxTextGroup,
    panel_type: u8,
    _morph_type: u8,
    _offset_len: i32,
    offset_data: Option<Vec<OffsetData>>,
}

impl Morph {
    pub fn name(&self) -> &PmxTextGroup {
        &self.name
    }

    pub fn offset_data(&self) -> Option<&Vec<OffsetData>> {
        self.offset_data.as_ref()
    }

    pub fn panel_type(&self) -> u8 {
        self.panel_type
    }
}

#[derive(Debug)]
pub enum OffsetData {
    Group(GroupMorph),
    Vertex(VertexMorph),
    Bone(BoneMorph),
    UV(UVMorph),
    UVExt1(UVMorph),
    UVExt2(UVMorph),
    UVExt3(UVMorph),
    UVExt4(UVMorph),
    Material(MaterialMorph),
    Flip(FlipMorph),
    Impulse(ImpulseMorph),
}

#[derive(Debug)]
pub struct GroupMorph {
    // morph index in this case
    pub index: Index,
    pub influence: f32,
}

#[derive(Debug)]
pub struct VertexMorph {
    // vertex index
    pub index: Index,
    pub translation: Vec3,
}

#[derive(Debug)]
pub struct BoneMorph {
    // bone index
    pub index: Index,
    pub translation: Vec3,
    pub rotation: Vec4,
}

#[derive(Debug)]
pub struct UVMorph {
    // vertex index
    pub index: Index,
    pub uv_offset: Vec4,
}

#[derive(Debug)]
pub struct MaterialMorph {
    // material index
    pub index: Index,
    pub operation: u8,
    pub diffuse: Vec4,
    pub specular: Vec3,
    pub specular_power: f32,
    pub ambient: Vec3,
    pub edge_color: Vec4,
    pub edge_size: f32,
    pub texture_add: Vec4,
    pub sphere_add: Vec4,
    pub toon_add: Vec4,
}

#[derive(Debug)]
pub struct FlipMorph {
    // morph index
    pub index: Index,
    pub influence: f32,
}

#[derive(Debug)]
pub struct ImpulseMorph {
    // rigid body index
    pub index: Index,
    pub is_local: bool,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
}

impl PmxParseable for Morph {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, globals: &Globals) -> Result<Self> {
        let name = parser.parse()?;

        let panel_type = {
            let mut byte = [0; 1];
            parser.reader.read_exact(&mut byte)?;
            byte[0]
        };

        // dbg!(&panel_type);

        let morph_type = {
            let mut byte = [0; 1];
            parser.reader.read_exact(&mut byte)?;
            byte[0]
        };

        // dbg!(&morph_type);

        let offset_len = {
            let mut len_bytes = [0; 4];
            parser.reader.read_exact(&mut len_bytes)?;
            i32::from_le_bytes(len_bytes)
        };

        // dbg!(&offset_len);

        let reader = &mut parser.reader;

        let offset_data = if offset_len > 0 {
            let mut data = Vec::with_capacity(offset_len as _);

            for _ in 0..offset_len {
                match morph_type {
                    0 => {
                        // Group Morph

                        let morph = GroupMorph {
                            index: Index::create(reader, globals.morph_idx_size.try_into()?, true)?,
                            influence: {
                                let mut bytes = [0; 4];
                                reader.read_exact(&mut bytes)?;
                                f32::from_le_bytes(bytes)
                            },
                        };

                        data.push(OffsetData::Group(morph));
                    }
                    1 => {
                        // Vertex Morph

                        let morph = VertexMorph {
                            index: Index::create(reader, globals.vert_idx_size.try_into()?, false)?,
                            translation: f32_array_from_le_bytes!(3, reader).into(),
                        };

                        data.push(OffsetData::Vertex(morph));
                    }
                    2 => {
                        // Bone Morph

                        let morph = BoneMorph {
                            index: Index::create(reader, globals.bone_idx_size.try_into()?, true)?,
                            translation: f32_array_from_le_bytes!(3, reader).into(),
                            rotation: f32_array_from_le_bytes!(4, reader).into(),
                        };

                        data.push(OffsetData::Bone(morph));
                    }
                    3..=7 => {
                        // UV Morph

                        let morph = UVMorph {
                            index: Index::create(reader, globals.vert_idx_size.try_into()?, false)?,
                            uv_offset: f32_array_from_le_bytes!(4, reader).into(),
                        };

                        data.push(OffsetData::UV(morph));
                    }
                    8 => {
                        // Material Morph

                        let morph = MaterialMorph {
                            index: Index::create(
                                reader,
                                globals.material_idx_size.try_into()?,
                                true,
                            )?,
                            operation: {
                                let mut byte = [0; 1];
                                reader.read_exact(&mut byte)?;
                                byte[0]
                            },
                            diffuse: f32_array_from_le_bytes!(4, reader).into(),
                            specular: f32_array_from_le_bytes!(3, reader).into(),
                            specular_power: {
                                let mut bytes = [0; 4];
                                reader.read_exact(&mut bytes)?;
                                f32::from_le_bytes(bytes)
                            },
                            ambient: f32_array_from_le_bytes!(3, reader).into(),
                            edge_color: f32_array_from_le_bytes!(4, reader).into(),
                            edge_size: {
                                let mut bytes = [0; 4];
                                reader.read_exact(&mut bytes)?;
                                f32::from_le_bytes(bytes)
                            },
                            texture_add: f32_array_from_le_bytes!(4, reader).into(),
                            sphere_add: f32_array_from_le_bytes!(4, reader).into(),
                            toon_add: f32_array_from_le_bytes!(4, reader).into(),
                        };

                        data.push(OffsetData::Material(morph));
                    }
                    9 => {
                        // Flip Morph

                        let morph = FlipMorph {
                            index: Index::create(reader, globals.morph_idx_size.try_into()?, true)?,
                            influence: {
                                let mut bytes = [0; 4];
                                reader.read_exact(&mut bytes)?;
                                f32::from_le_bytes(bytes)
                            },
                        };

                        data.push(OffsetData::Flip(morph));
                    }
                    10 => {
                        // Impulse Morph

                        let morph = ImpulseMorph {
                            index: Index::create(reader, globals.rb_idx_size.try_into()?, true)?,
                            is_local: {
                                let mut byte = [0; 1];
                                reader.read_exact(&mut byte)?;
                                byte[0] != 0
                            },
                            velocity: f32_array_from_le_bytes!(3, reader).into(),
                            angular_velocity: f32_array_from_le_bytes!(3, reader).into(),
                        };

                        data.push(OffsetData::Impulse(morph));
                    }
                    _ => Err(Error::InvalidMorphType)?,
                }
            }
            Some(data)
        } else {
            None
        };

        // dbg!(&offset_data);

        Ok(Self {
            name,
            panel_type,
            _morph_type: morph_type,
            _offset_len: offset_len,
            offset_data,
        })
    }
}
