use raylib::prelude::Vector3;

use crate::fragment::Fragment;
use crate::shaders::diffuse_factor;
use crate::Uniforms;

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let base_color = Vector3::new(0.08, 0.30, 0.85);
    let diffuse = diffuse_factor(fragment.normal, uniforms);
    base_color * (0.1 + 0.9 * diffuse)
}
