use std::{io::Read, slice::Iter};

use thiserror::Error;

use crate::{
    parser::{Parser, PmxParseable},
    pmx::{Globals, Pmx},
    types::{Flag, Index, PmxTextGroup},
    util::ReadExt,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Negative size encountered where positive expected")]
    NegativeSize,
    #[error("Invalid aerodynamics model type")]
    InvalidAerodynamicsModelType,
    #[error(transparent)]
    Type(#[from] crate::types::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct SoftBodies {
    len: usize,
    inner: Vec<SoftBody>,
}

impl PmxParseable for SoftBodies {
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
            "the parsed soft body count does not match the expected size"
        );

        Ok(Self {
            len: size,
            inner: inner_vec,
        })
    }
}

impl SoftBodies {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> Iter<'_, SoftBody> {
        self.inner.iter()
    }
}

#[repr(u8)]
pub enum SoftBodyFlag {
    BLink,
    ClusterCreation,
    LinkCrossing,
}

#[derive(Debug)]
pub struct SoftBody {
    name: PmxTextGroup,
    shape: Shape,
    material_index: Index,
    group_id: i8,
    non_collision_mask: i16,
    flag: Flag,
    b_link_distance: i32,
    num_clusters: i32,
    total_mass: f32,
    collision_margin: f32,
    // note that this is an i32 internally
    aerodynamics_model: AerodynamicsModel,
    config: Config,
    cluster: Cluster,
    iteration: Iteration,
    material: Material,
    anchor_rbs: Vec<AnchorRB>,
    vertex_pins: Vec<VertexPin>,
}

impl SoftBody {
    pub fn has_flag(&self, flag: SoftBodyFlag) -> bool {
        self.flag().get_state(flag as u8)
    }

    pub fn name(&self) -> &PmxTextGroup {
        &self.name
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    pub fn material_index(&self) -> &Index {
        &self.material_index
    }

    pub fn group_id(&self) -> i8 {
        self.group_id
    }

    pub fn non_collision_mask(&self) -> i16 {
        self.non_collision_mask
    }

    pub fn flag(&self) -> Flag {
        self.flag
    }

    pub fn b_link_distance(&self) -> i32 {
        self.b_link_distance
    }

    pub fn num_clusters(&self) -> i32 {
        self.num_clusters
    }

    pub fn total_mass(&self) -> f32 {
        self.total_mass
    }

    pub fn collision_margin(&self) -> f32 {
        self.collision_margin
    }

    pub fn aerodynamics_model(&self) -> &AerodynamicsModel {
        &self.aerodynamics_model
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn cluster(&self) -> &Cluster {
        &self.cluster
    }

    pub fn iteration(&self) -> &Iteration {
        &self.iteration
    }

    pub fn material(&self) -> &Material {
        &self.material
    }

    pub fn anchor_rbs(&self) -> &[AnchorRB] {
        &self.anchor_rbs
    }

    pub fn vertex_pins(&self) -> &[VertexPin] {
        &self.vertex_pins
    }
}

impl PmxParseable for SoftBody {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, globals: &Globals) -> Result<Self> {
        let name = parser.parse()?;
        let shape = parser.reader.read_byte()?.try_into()?;
        let material_index = Index::parse_material(&mut parser.reader, globals.material_idx_size)?;
        let group_id = parser.reader.read_i8()?;
        let non_collision_mask = parser.reader.read_i16_le()?;
        let flag = parser.parse()?;
        let b_link_distance = parser.reader.read_i32_le()?;
        let num_clusters = parser.reader.read_i32_le()?;
        let total_mass = parser.reader.read_f32_le()?;
        let collision_margin = parser.reader.read_f32_le()?;
        let aerodynamics_model = parser.reader.read_i32_le()?.try_into()?;
        let config = parser.parse()?;
        let cluster = parser.parse()?;
        let iteration = parser.parse()?;
        let material = parser.parse()?;

        let anchor_rb_count = parser.reader.read_i32_le()?;
        if anchor_rb_count.is_negative() {
            Err(Error::NegativeSize)?
        }
        let mut anchor_rbs = Vec::with_capacity(anchor_rb_count as _);
        for _ in 0..anchor_rb_count {
            let anchor_rb = parser.parse()?;
            anchor_rbs.push(anchor_rb);
        }

        let vertex_pin_count = parser.reader.read_i32_le()?;
        if vertex_pin_count.is_negative() {
            Err(Error::NegativeSize)?
        }
        let mut vertex_pins = Vec::with_capacity(vertex_pin_count as _);
        for _ in 0..vertex_pin_count {
            let vertex_pin = VertexPin {
                index: Index::parse_vertex(&mut parser.reader, globals.vert_idx_size)?,
            };
            vertex_pins.push(vertex_pin);
        }

        Ok(Self {
            name,
            shape,
            material_index,
            group_id,
            non_collision_mask,
            flag,
            b_link_distance,
            num_clusters,
            total_mass,
            collision_margin,
            aerodynamics_model,
            config,
            cluster,
            iteration,
            material,
            anchor_rbs,
            vertex_pins,
        })
    }
}

// Config VCF	float	Velocities correction factor (Baumgarte)
// Config DP	float	Damping coefficient
// Config DG	float	Drag coefficient
// Config LF	float	Lift coefficient
// Config PR	float	Pressure coefficient
// Config VC	float	Volume conversation coefficient
// Config DF	float	Dynamic friction coefficient
// Config MT	float	Pose matching coefficient
// Config CHR	float	Rigid contacts hardness
// Config KHR	float	Kinetic contacts hardness
// Config SHR	float	Soft contacts hardness
// Config AHR	float	Anchors hardness
#[derive(Debug)]
pub struct Config {
    pub vel_correction_factor: f32,
    pub damping_coefficient: f32,
    pub drag_coefficient: f32,
    pub lift_coefficient: f32,
    pub pressure_coefficient: f32,
    pub volume_conversation_coefficient: f32,
    pub dynamic_friction_coefficient: f32,
    pub pose_matching_coefficient: f32,
    pub rigid_contacts_hardness: f32,
    pub kinetic_contacts_hardness: f32,
    pub soft_contacts_hardness: f32,
    pub anchors_hardness: f32,
}

impl PmxParseable for Config {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, _globals: &Globals) -> Result<Self> {
        Ok(Self {
            vel_correction_factor: parser.reader.read_f32_le()?,
            damping_coefficient: parser.reader.read_f32_le()?,
            drag_coefficient: parser.reader.read_f32_le()?,
            lift_coefficient: parser.reader.read_f32_le()?,
            pressure_coefficient: parser.reader.read_f32_le()?,
            volume_conversation_coefficient: parser.reader.read_f32_le()?,
            dynamic_friction_coefficient: parser.reader.read_f32_le()?,
            pose_matching_coefficient: parser.reader.read_f32_le()?,
            rigid_contacts_hardness: parser.reader.read_f32_le()?,
            kinetic_contacts_hardness: parser.reader.read_f32_le()?,
            soft_contacts_hardness: parser.reader.read_f32_le()?,
            anchors_hardness: parser.reader.read_f32_le()?,
        })
    }
}

// Cluster SRHR_CL	float	Soft vs rigid hardness
// Cluster SKHR_CL	float	Soft vs kinetic hardness
// Cluster SSHR_CL	float	Soft vs soft hardness
// Cluster SR_SPLT_CL	float	Soft vs rigid impulse split
// Cluster SK_SPLT_CL	float	Soft vs kinetic impulse split
// Cluster SS_SPLT_CL	float	Soft vs soft impulse split
#[derive(Debug)]
pub struct Cluster {
    pub soft_vs_rigid_hardness: f32,
    pub soft_vs_kinetic_hardness: f32,
    pub soft_vs_soft_hardness: f32,
    pub soft_vs_rigid_impulse_split: f32,
    pub soft_vs_kinetic_impulse_split: f32,
    pub soft_vs_soft_impulse_split: f32,
}

impl PmxParseable for Cluster {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, _globals: &Globals) -> Result<Self> {
        Ok(Self {
            soft_vs_rigid_hardness: parser.reader.read_f32_le()?,
            soft_vs_kinetic_hardness: parser.reader.read_f32_le()?,
            soft_vs_soft_hardness: parser.reader.read_f32_le()?,
            soft_vs_rigid_impulse_split: parser.reader.read_f32_le()?,
            soft_vs_kinetic_impulse_split: parser.reader.read_f32_le()?,
            soft_vs_soft_impulse_split: parser.reader.read_f32_le()?,
        })
    }
}

// Iteration V_IT	int	Velocities solver iterations
// Iteration P_IT	int	Positions solver iterations
// Iteration D_IT	int	Drift solver iterations
// Iteration C_IT	int	Cluster solver iterations
#[derive(Debug)]
pub struct Iteration {
    pub velocities_solver_iterations: i32,
    pub positions_solver_iterations: i32,
    pub drift_solver_iterations: i32,
    pub cluster_solver_iterations: i32,
}

impl PmxParseable for Iteration {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, _globals: &Globals) -> Result<Self> {
        Ok(Self {
            velocities_solver_iterations: parser.reader.read_i32_le()?,
            positions_solver_iterations: parser.reader.read_i32_le()?,
            drift_solver_iterations: parser.reader.read_i32_le()?,
            cluster_solver_iterations: parser.reader.read_i32_le()?,
        })
    }
}

// Material LST	int	Linear stiffness coefficient
// Material AST	int	Area / Angular stiffness coefficient
// Material VST	int	Volume stiffness coefficient
#[derive(Debug)]
pub struct Material {
    pub linear_stiffness_coefficient: i32,
    pub area_angular_stiffness_coefficient: i32,
    pub volume_stiffness_coefficient: i32,
}

impl PmxParseable for Material {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, _globals: &Globals) -> Result<Self> {
        Ok(Self {
            linear_stiffness_coefficient: parser.reader.read_i32_le()?,
            area_angular_stiffness_coefficient: parser.reader.read_i32_le()?,
            volume_stiffness_coefficient: parser.reader.read_i32_le()?,
        })
    }
}

#[derive(Debug)]
pub enum Shape {
    TriMesh,
    Rope,
}

impl TryFrom<u8> for Shape {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        let ok = match value {
            0 => Self::TriMesh,
            1 => Self::Rope,
            _ => Err(Error::InvalidAerodynamicsModelType)?,
        };

        Ok(ok)
    }
}

#[derive(Debug)]
pub enum AerodynamicsModel {
    VPoint,
    VTwoSided,
    VOneSided,
    FTwoSided,
    FOneSided,
}

impl TryFrom<i32> for AerodynamicsModel {
    type Error = Error;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        let ok = match value {
            0 => Self::VPoint,
            1 => Self::VTwoSided,
            2 => Self::VOneSided,
            3 => Self::FTwoSided,
            4 => Self::FOneSided,
            _ => Err(Error::InvalidAerodynamicsModelType)?,
        };

        Ok(ok)
    }
}

#[derive(Debug)]
pub struct AnchorRB {
    pub rb_index: Index,
    pub vertex_index: Index,
    pub near_mode: i8,
}

impl PmxParseable for AnchorRB {
    type Error = Error;

    fn parse<R: Read>(parser: &mut Parser<R, Pmx>, globals: &Globals) -> Result<Self> {
        Ok(Self {
            rb_index: Index::parse_rigidbody(&mut parser.reader, globals.rb_idx_size)?,
            vertex_index: Index::parse_vertex(&mut parser.reader, globals.vert_idx_size)?,
            near_mode: parser.reader.read_i8()?,
        })
    }
}

#[derive(Debug)]
pub struct VertexPin {
    pub index: Index,
}
