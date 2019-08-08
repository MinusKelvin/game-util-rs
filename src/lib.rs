pub extern crate arrayvec;
pub extern crate glutin;
pub extern crate gl;
pub extern crate lyon;
pub extern crate euclid;
pub extern crate rusttype;
pub extern crate image;

mod gameloop;
mod glutil;
mod text;
mod window;

pub use gameloop::*;
pub use glutil::*;
pub use text::*;
pub use window::*;

pub mod prelude {
    pub use euclid::{ vec2, vec3, point2, point3 };
    pub use serde::{ Serialize, Deserialize };
    pub use arrayvec::{ ArrayVec, ArrayString };

    pub use gl;
    pub use glutin;

    pub type Vec2<T, U=euclid::UnknownUnit> = euclid::Vector2D<T, U>;
    pub type Vec3<T, U=euclid::UnknownUnit> = euclid::Vector3D<T, U>;
    pub type Point2<T, U=euclid::UnknownUnit> = euclid::Point2D<T, U>;
    pub type Point3<T, U=euclid::UnknownUnit> = euclid::Point3D<T, U>;
}