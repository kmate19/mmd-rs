use core::fmt;

use std::{
    io::{BufReader, ErrorKind, Read},
    path::Path,
};

use thiserror::Error;

use crate::{
    bone, frame, joint, material, morph,
    parser::Parser,
    rb, surface, texture,
    types::{self, IndexSize, PmxTextGroup, TextEncoding},
    util::ReadExt,
    vertex,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("File had an invalid tag, did you input the correct file?")]
    InvalidTag,
    #[error("Invalid global variable amount, must be at least 8")]
    InvalidGlobalCount,
    #[error("Invalid Vec4 additional value, must be between 0 and 4 inclusive")]
    InvalidVec4Additional,
    #[error("Leftover bytes in file after parsing {amount} for version {version}")]
    LeftoverBytes { amount: usize, version: f32 },
    #[error("Unsupported file version: {version}")]
    UnsupportedVersion { version: f32 },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error parsing vertex: {0}")]
    Vertex(#[from] vertex::Error),
    #[error("PMX type error: {0}")]
    Type(#[from] types::Error),
    #[error("Surface error: {0}")]
    Surface(#[from] surface::Error),
    #[error("Texture error: {0}")]
    Texture(#[from] texture::Error),
    #[error("Material error: {0}")]
    Material(#[from] material::Error),

    #[error("Bone error: {0}")]
    Bone(#[from] bone::Error),
    #[error("Morph error: {0}")]
    Morph(#[from] morph::Error),
    #[error("Frame error: {0}")]
    Frame(#[from] frame::Error),
    #[error("Rigidbody error: {0}")]
    Rb(#[from] rb::Error),
    #[error("Joint error: {0}")]
    Joint(#[from] joint::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub use material::Error as MaterialError;
pub use surface::Error as SurfaceError;
pub use texture::Error as TextureError;
pub use types::Error as TypeError;
pub use vertex::Error as VertexError;

/// The main PMX structure representing a parsed PMX file.
/// Can be created using the `Pmx::open` function.
pub struct Pmx {
    header: Header,
    vertices: vertex::Vertices,
    surfaces: surface::Surfaces,
    textures: texture::Textures,
    materials: material::Materials,
    bones: bone::Bones,
    morphs: morph::Morphs,
    frames: frame::Frames,
    rigid_bodies: rb::RigidBodies,
    joints: joint::Joints,
}

impl fmt::Debug for Pmx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pmx")
            .field("header", &self.header)
            .field(
                "vertices",
                &format!(
                    "<truncated, print the field separately if you want to see raw contents> (size: {})",
                    self.vertices.len()
                ),
            )
            .field("surfaces", &format!(
                "<truncated, print the field separately if you want to see raw contents> (size: {})",
                self.surfaces.len()
            ))
            .field("textures", &self.textures)
            .field("materials", &format!(
                "<truncated, print the field separately if you want to see raw contents> (size: {})",
                self.materials.len()
            ))
            .field("bones", &format!(
                "<truncated, print the field separately if you want to see raw contents> (size: {})",
                self.bones.len()
            ))
            .field("morphs", &format!(
                "<truncated, print the field separately if you want to see raw contents> (size: {})",
                self.morphs.len()
            ))
            .field("frames", &format!(
                "<truncated, print the field separately if you want to see raw contents> (size: {})",
                self.morphs.len()
            ))
            .field("rigid_bodies", &format!(
                "<truncated, print the field separately if you want to see raw contents> (size: {})",
                self.rigid_bodies.len()
            ))
            .field("joints", &format!(
                "<truncated, print the field separately if you want to see raw contents> (size: {})",
                self.joints.len()
            ))
            .finish()
    }
}

impl Pmx {
    /// Open and parse a PMX file from the given path.
    /// ```no_run
    /// use mmd_rs::pmx::Pmx;
    ///
    /// let pmx = Pmx::open("path/to/model.pmx").expect("Failed to open PMX file");
    /// println!("Model name: {}", pmx.header().name());
    /// ```
    ///
    /// This function can fail from standard IO errors, as well as parsing errors related to the PMX file.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let fh = std::fs::File::open(path)?;
        let reader = BufReader::new(fh);
        let mut parser = Parser::new(reader);

        let header = parser.parse_header()?;

        if header.version != 2.0 && header.version != 2.1 {
            Err(Error::UnsupportedVersion {
                version: header.version,
            })?
        }

        let vertices = parser.parse()?;

        let surfaces = parser.parse()?;
        let textures = parser.parse()?;

        let materials = parser.parse()?;

        let bones = parser.parse()?;

        let morphs = parser.parse()?;

        let frames = parser.parse()?;

        let rigid_bodies = parser.parse()?;

        let joints = parser.parse()?;

        if header.version == 2.1 {
            // TODO(mate): but the soft body parsing here
        }

        match parser.reader.read_byte() {
            Err(err) if err.kind() == ErrorKind::UnexpectedEof => Ok(Pmx {
                header,
                vertices,
                surfaces,
                materials,
                textures,
                bones,
                morphs,
                frames,
                rigid_bodies,
                joints,
            }),
            Err(err) => Err(err)?,
            Ok(_) => Err(Error::LeftoverBytes {
                amount: parser.reader.bytes().count() + 1,
                version: header.version,
            })?,
        }
    }

    /// Get the PMX file header.
    ///
    /// The header contains metadata about the PMX file, including version information, global settings, and model names/comments.
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Get the PMX file vertices.
    ///
    /// A vertex contains information about a single point in 3D space, including its position, normal vector, UV coordinates, and skinning information.
    ///
    /// ```no_run
    /// use mmd_rs::pmx::Pmx;
    ///
    /// let pmx = Pmx::open("path/to/model.pmx").expect("Failed to open PMX file");
    ///
    /// for vertex in pmx.vertices().iter() {
    ///    println!("Vertex position: {:?}", vertex.pos());
    /// }
    /// ```
    pub fn vertices(&self) -> &vertex::Vertices {
        &self.vertices
    }

    /// Get the PMX file surfaces.
    ///
    /// A surface defines how vertices are connected to form the 3D model's geometry, typically represented as triangles.
    ///
    /// These can therefore be thought of regular indices in other 3D formats.
    ///
    /// ```no_run
    /// use mmd_rs::pmx::Pmx;
    ///
    /// let pmx = Pmx::open("path/to/model.pmx").expect("Failed to open PMX file");
    ///
    /// for surface in pmx.surfaces().iter() {
    ///   println!("Surface index: {:?}", surface.as_index());
    /// }
    pub fn surfaces(&self) -> &surface::Surfaces {
        &self.surfaces
    }

    /// Get the PMX file textures.
    ///
    /// A texture is an image applied to the surface of a 3D model to give it color and detail, internally this is just a file path to the texture image.
    ///
    /// ```no_run
    /// use mmd_rs::pmx::Pmx;
    ///
    /// let pmx = Pmx::open("path/to/model.pmx").expect("Failed to open PMX file");
    ///
    /// for texture in pmx.textures().iter() {
    ///    println!("Texture path: {}", texture.path().as_str());
    /// }
    pub fn textures(&self) -> &texture::Textures {
        &self.textures
    }

    /// Get the PMX file materials.
    ///
    /// A material defines the visual properties of a surface, like blending mode, edge color, etc.
    ///
    /// ```no_run
    /// use mmd_rs::pmx::Pmx;
    ///
    /// let pmx = Pmx::open("path/to/model.pmx").expect("Failed to open PMX file");
    ///
    /// for material in pmx.materials().iter() {
    ///    println!("Material name: {}", material.name());
    /// }
    pub fn materials(&self) -> &material::Materials {
        &self.materials
    }

    /// Get the PMX file bones.
    ///
    /// A bone is a part of the skeletal structure used for animating the 3D model, allowing for complex movements and deformations.
    ///
    /// ```no_run
    /// use mmd_rs::pmx::Pmx;
    ///
    /// let pmx = Pmx::open("path/to/model.pmx").expect("Failed to open PMX file");
    ///
    /// for bone in pmx.bones().iter() {
    ///   println!("Bone name: {}", bone.name().universal().as_str());
    /// }
    pub fn bones(&self) -> &bone::Bones {
        &self.bones
    }

    // TODO(mate): these are not really great examples to be honest

    /// Get the PMX file morphs.
    ///
    /// A morph is a predefined transformation that can be applied to the model to change its shape or appearance, such as facial expressions or muscle movements.
    ///
    /// ```no_run
    /// use mmd_rs::pmx::Pmx;
    ///
    /// let pmx = Pmx::open("path/to/model.pmx").expect("Failed to open PMX file");
    ///
    /// for morph in pmx.morphs().iter() {
    ///   println!("Morph name: {}", morph.name().universal().as_str());
    /// }
    pub fn morphs(&self) -> &morph::Morphs {
        &self.morphs
    }

    pub fn frames(&self) -> &frame::Frames {
        &self.frames
    }

    pub fn rigid_bodies(&self) -> &rb::RigidBodies {
        &self.rigid_bodies
    }

    pub fn joints(&self) -> &joint::Joints {
        &self.joints
    }
}

#[derive(Debug)]
pub struct Header {
    pub(crate) version: f32,
    pub(crate) globals: Globals,
    pub(crate) name: PmxTextGroup,
    pub(crate) comment: PmxTextGroup,
}

impl Header {
    pub fn version(&self) -> f32 {
        self.version
    }

    pub fn globals(&self) -> &Globals {
        &self.globals
    }

    pub fn name(&self) -> &PmxTextGroup {
        &self.name
    }

    pub fn comment(&self) -> &PmxTextGroup {
        &self.comment
    }
}

#[derive(Debug, Clone)]
pub struct Globals {
    pub(crate) encoding: TextEncoding,
    pub(crate) vec4_additional: u8,
    pub(crate) vert_idx_size: IndexSize,
    pub(crate) tex_idx_size: IndexSize,
    pub(crate) material_idx_size: IndexSize,
    pub(crate) bone_idx_size: IndexSize,
    pub(crate) morph_idx_size: IndexSize,
    pub(crate) rb_idx_size: IndexSize,
    /// Store additional fields here that we don't know the specific purpose of right now.
    #[allow(dead_code, reason = "we don't have a use for these yet")]
    pub(crate) additional: Option<Vec<i8>>,
}

impl Globals {
    pub(crate) fn from_bytes(r: &mut impl Read) -> Result<Self> {
        let global_count = r.read_i8()?;
        if global_count < 8 {
            Err(Error::InvalidGlobalCount)?
        }

        // note that global count is actually an i8, but since we already checked it's >= 8 we can assume its not negative.
        let mut globals = vec![0; global_count as usize];

        r.read_exact(&mut globals)?;

        match globals[1] {
            0..=4 => {}
            _ => Err(Error::InvalidVec4Additional)?,
        }

        let additional = if global_count > 8 {
            // because these are signed bytes in the spec, we need to convert them properly
            // note that we dont convert the rest even though theyre signed
            // but since they have a corresponding type with limited value, try_into will catch invalid values anyways
            Some(globals.split_off(8).into_iter().map(|b| b as i8).collect())
        } else {
            None
        };

        Ok(Self {
            encoding: globals[0].try_into()?,
            vec4_additional: globals[1],
            vert_idx_size: globals[2].try_into()?,
            tex_idx_size: globals[3].try_into()?,
            material_idx_size: globals[4].try_into()?,
            bone_idx_size: globals[5].try_into()?,
            morph_idx_size: globals[6].try_into()?,
            rb_idx_size: globals[7].try_into()?,
            additional,
        })
    }

    pub fn encoding(&self) -> TextEncoding {
        self.encoding
    }
}
