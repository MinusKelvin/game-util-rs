pub extern crate arrayvec;
pub extern crate euclid;
pub extern crate futures;
pub extern crate rusttype;
pub extern crate winit;

mod gameloop;
pub mod glutil;
pub mod sound;
pub mod sprite;
pub mod text;
pub mod tilemap;

#[cfg_attr(target_arch = "wasm32", path = "web/mod.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path = "desktop/mod.rs")]
mod backend;

pub use backend::util::*;
pub use gameloop::*;

pub mod prelude {
    pub use arrayvec::{ArrayString, ArrayVec};
    pub use euclid::{point2, point3, rect, size2, size3, vec2, vec3};
    pub use glow::HasContext;
    pub use serde::{Deserialize, Serialize};

    pub use crate::glutil;
    pub use crate::glutil::Gl;
    pub use glow;

    pub type Vec2<T, U = euclid::UnknownUnit> = euclid::Vector2D<T, U>;
    pub type Vec3<T, U = euclid::UnknownUnit> = euclid::Vector3D<T, U>;
    pub type Point2<T, U = euclid::UnknownUnit> = euclid::Point2D<T, U>;
    pub type Point3<T, U = euclid::UnknownUnit> = euclid::Point3D<T, U>;
    pub type Size2<T, U = euclid::UnknownUnit> = euclid::Size2D<T, U>;
    pub type Size3<T, U = euclid::UnknownUnit> = euclid::Size3D<T, U>;
    pub type Rect<T, U = euclid::UnknownUnit> = euclid::Rect<T, U>;
    pub type Transform2D<T, Src = euclid::UnknownUnit, Dst = euclid::UnknownUnit> =
        euclid::Transform2D<T, Src, Dst>;
    pub type Transform3D<T, Src = euclid::UnknownUnit, Dst = euclid::UnknownUnit> =
        euclid::Transform3D<T, Src, Dst>;
}
