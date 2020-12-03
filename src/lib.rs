mod gameloop;
pub mod glutil;
mod text;
mod tilemap;
mod sprite;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        
    } else {
        pub mod desktop;
        mod sound_rodio;

        pub use sound_rodio::*;
    }
}

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
    pub use winit;
    pub use crate::glutil;

    pub type Gl = std::rc::Rc<glow::Context>;

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