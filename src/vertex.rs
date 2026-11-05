use raylib::prelude::Vector3;

#[derive(Clone)]
pub struct Vertex {
    pub position: Vector3,
    pub normal: Vector3,
    pub color: Vector3,
    pub world_position: Vector3,
    pub transformed_position: Vector3,
    pub transformed_normal: Vector3,
}

impl Vertex {
    pub fn new(position: Vector3, normal: Vector3, color: Vector3) -> Self {
        Self {
            position,
            normal,
            color,
            world_position: position,
            transformed_position: position,
            transformed_normal: normal,
        }
    }
}
