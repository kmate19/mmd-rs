use core::fmt;

use std::{
    io::{BufReader, Read},
    path::Path,
};

use thiserror::Error;

use crate::{
    material,
    parser::Parser,
    surface, texture,
    types::{self, PmxTextGroup, TextEncoding},
    vertex,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("File had an invalid tag, did you input the correct file?")]
    InvalidTag,
    #[error("Invalid global variable amount, must be at least 8")]
    InvalidGlobalCount,
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

        let vertices = parser.parse()?;

        let surfaces = parser.parse()?;
        let textures = parser.parse()?;

        let materials = parser.parse()?;

        Ok(Pmx {
            header,
            vertices,
            surfaces,
            materials,
            textures,
        })
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
    pub(crate) vert_idx_size: u8,
    pub(crate) tex_idx_size: u8,
    pub(crate) material_idx_size: u8,
    pub(crate) bone_idx_size: u8,
    pub(crate) morph_idx_size: u8,
    pub(crate) rb_idx_size: u8,
    /// Store additional fields here that we don't know the specific purpose of right now.
    pub(crate) additional: Option<Vec<u8>>,
}

impl Globals {
    pub(crate) fn from_bytes(r: &mut impl Read) -> Result<Self> {
        let mut global_count = [0; 1];

        r.read_exact(&mut global_count)?;

        if global_count[0] < 8 {
            Err(Error::InvalidGlobalCount)?
        }

        // note that global count is actually an i8, but since we already checked it's >= 8 we can assume its not negative.
        let mut globals = vec![0; global_count[0] as usize];

        r.read_exact(&mut globals)?;

        let additional = if global_count[0] > 8 {
            Some(globals.split_off(8))
        } else {
            None
        };

        Ok(Self {
            encoding: globals[0].try_into()?,
            vec4_additional: globals[1],
            vert_idx_size: globals[2],
            tex_idx_size: globals[3],
            material_idx_size: globals[4],
            bone_idx_size: globals[5],
            morph_idx_size: globals[6],
            rb_idx_size: globals[7],
            additional,
        })
    }

    pub fn encoding(&self) -> TextEncoding {
        self.encoding
    }
}
