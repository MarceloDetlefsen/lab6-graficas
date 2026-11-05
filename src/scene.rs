use raylib::prelude::{Matrix, Vector3};

use crate::obj::Obj;
use crate::primitive::primitive_assembly;
use crate::rasterizer::{VIEWPORT_HEIGHT, VIEWPORT_WIDTH};
use crate::shaders::vertex_shader;
use crate::vertex::Vertex;
use crate::Uniforms;

pub fn build_model_matrix(obj: &Obj, scale: f32, yaw: f32, pitch: f32, zoom: f32) -> Matrix {
    let (min, max) = obj.bounds();
    let model_center = Vector3::new(
        (min.x + max.x) * 0.5,
        (min.y + max.y) * 0.5,
        (min.z + max.z) * 0.5,
    );
    let linear = Matrix::rotate_y(yaw)
        * Matrix::rotate_x(pitch)
        * Matrix::scale(scale * zoom, -scale * zoom, scale * zoom);
    let transformed_center = Vector3::new(
        linear.m0 * model_center.x + linear.m4 * model_center.y + linear.m8 * model_center.z,
        linear.m1 * model_center.x + linear.m5 * model_center.y + linear.m9 * model_center.z,
        linear.m2 * model_center.x + linear.m6 * model_center.y + linear.m10 * model_center.z,
    );
    let mut model_matrix = linear;
    model_matrix.m12 = VIEWPORT_WIDTH as f32 / 2.0 - transformed_center.x;
    model_matrix.m13 = VIEWPORT_HEIGHT as f32 / 2.0 - transformed_center.y;
    model_matrix.m14 = -transformed_center.z;
    model_matrix
}

pub fn build_triangles(obj: &Obj, uniforms: &Uniforms) -> Vec<[Vertex; 3]> {
    let vertices: Vec<Vertex> = obj.indices.iter().enumerate().map(|(i, &index)| {
        let color = match i % 3 {
            0 => Vector3::new(1.0, 0.0, 0.0),
            1 => Vector3::new(0.0, 1.0, 0.0),
            _ => Vector3::new(0.0, 0.0, 1.0),
        };
        Vertex::new_with_color(obj.vertices[index], color)
    }).map(|vertex| vertex_shader(&vertex, uniforms)).collect();
    primitive_assembly(&vertices)
}
