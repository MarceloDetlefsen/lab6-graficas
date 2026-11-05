use raylib::prelude::Vector3;

use crate::fragment::Fragment;
use crate::Uniforms;

pub fn shade(_fragment: &Fragment, _uniforms: &Uniforms) -> Vector3 {
    Vector3::new(1.0, 0.55, 0.08)
}
