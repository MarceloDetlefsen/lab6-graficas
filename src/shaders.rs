use raylib::prelude::*;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::noise;
use crate::vertex::Vertex;

mod gas_giant;
mod rocky;
mod star;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Shader {
    Star,
    Rocky,
    GasGiant,
}

fn multiply_matrix_vector4(matrix: &Matrix, vector: &Vector4) -> Vector4 {
    Vector4::new(
        matrix.m0 * vector.x + matrix.m4 * vector.y + matrix.m8 * vector.z + matrix.m12 * vector.w,
        matrix.m1 * vector.x + matrix.m5 * vector.y + matrix.m9 * vector.z + matrix.m13 * vector.w,
        matrix.m2 * vector.x + matrix.m6 * vector.y + matrix.m10 * vector.z + matrix.m14 * vector.w,
        matrix.m3 * vector.x + matrix.m7 * vector.y + matrix.m11 * vector.z + matrix.m15 * vector.w,
    )
}

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position = Vector4::new(vertex.position.x, vertex.position.y, vertex.position.z, 1.0);
    let transformed = multiply_matrix_vector4(&uniforms.model_matrix, &position);

    let mut result = vertex.clone();
    result.world_position = Vector3::new(
        transformed.x - uniforms.model_matrix.m12,
        transformed.y - uniforms.model_matrix.m13,
        transformed.z - uniforms.model_matrix.m14,
    );
    result.transformed_position = Vector3::new(transformed.x, transformed.y, transformed.z);
    result.transformed_normal = Vector3::new(
        uniforms.model_matrix.m0 * vertex.normal.x
            + uniforms.model_matrix.m4 * vertex.normal.y
            + uniforms.model_matrix.m8 * vertex.normal.z,
        uniforms.model_matrix.m1 * vertex.normal.x
            + uniforms.model_matrix.m5 * vertex.normal.y
            + uniforms.model_matrix.m9 * vertex.normal.z,
        uniforms.model_matrix.m2 * vertex.normal.x
            + uniforms.model_matrix.m6 * vertex.normal.y
            + uniforms.model_matrix.m10 * vertex.normal.z,
    )
    .normalize();
    result
}

/// Vector desde la superficie hacia la cámara, en espacio mundo.
pub fn surface_view(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    (uniforms.camera_pos - fragment.world_pos).normalize()
}

/// Coseno de vista radial. El valor absoluto compensa que el z-buffer usa
/// menor z como más cercano mientras la cámara ortográfica está en +z; la
/// cara visible tiene entonces la normal opuesta al vector hacia la cámara.
pub fn surface_mu(fragment: &Fragment, uniforms: &Uniforms) -> f32 {
    fragment
        .normal
        .normalize()
        .dot(surface_view(fragment, uniforms))
        .abs()
        .clamp(0.0, 1.0)
}

/// Difusa con light_dir apuntando desde la superficie hacia la fuente.
pub fn diffuse_factor(normal: Vector3, uniforms: &Uniforms) -> f32 {
    normal.normalize().dot(uniforms.light_dir).max(0.0)
}

pub fn fragment_shader(fragment: &Fragment, shader: Shader, uniforms: &Uniforms) -> Vector3 {
    if uniforms.debug_noise {
        let sample_position = fragment.object_pos.normalize() * uniforms.noise_frequency;
        let value = noise::fbm(sample_position, 5, 2.0, 0.5);
        return Vector3::new(value, value, value);
    }

    match shader {
        Shader::Star => star::shade(fragment, uniforms),
        Shader::Rocky => rocky::shade(fragment, uniforms),
        Shader::GasGiant => gas_giant::shade(fragment, uniforms),
    }
}

pub fn shader_name(shader: Shader) -> &'static str {
    match shader {
        Shader::Star => "Shader: Star",
        Shader::Rocky => "Shader: Rocky",
        Shader::GasGiant => "Shader: Gas Giant",
    }
}
