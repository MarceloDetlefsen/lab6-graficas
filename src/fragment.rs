use raylib::math::{Vector2, Vector3};

  pub struct Fragment {
      pub position: Vector2,
      pub object_pos: Vector3,
      pub world_pos: Vector3,
      pub normal: Vector3,
      pub color: Vector3,
      pub depth: f32,
  }

  impl Fragment {
      pub fn new(x: f32, y: f32, color: Vector3, depth: f32) -> Self {
          Self {
              position: Vector2::new(x, y),
              object_pos: Vector3::zero(),
              world_pos: Vector3::zero(),
              normal: Vector3::zero(),
              color,
              depth,
          }
      }
  }
