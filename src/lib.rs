pub extern crate arrayvec;
pub extern crate euclid;
pub extern crate rusttype;
pub extern crate image;
pub extern crate winit;

mod gameloop;
pub mod glutil;
mod text;
mod tilemap;
mod sprite;

#[cfg_attr(target_arch = "wasm32", path = "web.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path = "desktop.rs")]
pub mod platform;

pub use gameloop::*;
pub use text::*;
pub use tilemap::*;
pub use sprite::*;

pub mod prelude {
    pub use euclid::{ vec2, vec3, point2, point3, rect, size2, size3 };
    pub use serde::{ Serialize, Deserialize };
    pub use arrayvec::{ ArrayVec, ArrayString };
    pub use glow::HasContext;

    pub use glow;
    pub use crate::glutil;
    pub use crate::glutil::Gl;

    pub type Vec2<T, U=euclid::UnknownUnit> = euclid::Vector2D<T, U>;
    pub type Vec3<T, U=euclid::UnknownUnit> = euclid::Vector3D<T, U>;
    pub type Point2<T, U=euclid::UnknownUnit> = euclid::Point2D<T, U>;
    pub type Point3<T, U=euclid::UnknownUnit> = euclid::Point3D<T, U>;
    pub type Size2<T, U=euclid::UnknownUnit> = euclid::Size2D<T, U>;
    pub type Size3<T, U=euclid::UnknownUnit> = euclid::Size3D<T, U>;
    pub type Rect<T, U=euclid::UnknownUnit> = euclid::Rect<T, U>;
    pub type Transform2D<T, Src=euclid::UnknownUnit, Dst=euclid::UnknownUnit> =
        euclid::Transform2D<T, Src, Dst>;
    pub type Transform3D<T, Src=euclid::UnknownUnit, Dst=euclid::UnknownUnit> =
        euclid::Transform3D<T, Src, Dst>;
}