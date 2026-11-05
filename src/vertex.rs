use raylib::prelude::Vector3;

#[derive(Clone)]
pub struct Vertex {
    pub position: Vector3,
    pub color: Vector3,
    pub transformed_position: Vector3,
}

impl Vertex {
    pub fn new_with_color(position: Vector3, color: Vector3) -> Self {
        Self { position, color, transformed_position: position }
    }
}
