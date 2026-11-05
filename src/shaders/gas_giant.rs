use raylib::prelude::Vector3;

use crate::fragment::Fragment;
use crate::Uniforms;

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let base_color = Vector3::new(0.08, 0.30, 0.85);
    let normal = fragment.normal.normalize();
    let diffuse = normal.dot(uniforms.light_dir).max(0.0);
    base_color * (0.1 + 0.9 * diffuse)
}
