#[cfg(not(feature = "math_glam"))]
pub type Vec2 = [f32; 2];
#[cfg(not(feature = "math_glam"))]
pub type Vec3 = [f32; 3];
#[cfg(not(feature = "math_glam"))]
pub type Vec4 = [f32; 4];

#[cfg(feature = "math_glam")]
pub use glam::{Vec2, Vec3, Vec4};

mod bone;
mod material;
mod morph;
mod parser;
pub mod pmx;
mod surface;
mod texture;
mod types;
mod util;
mod vertex;
