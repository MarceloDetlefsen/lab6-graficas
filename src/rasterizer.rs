use crate::fragment::Fragment;
use crate::triangle::{triangle_barycentric, triangle_scanline, triangle_wireframe};
use crate::vertex::Vertex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriangleMode { Wireframe, FilledBarycentric, FilledScanline }

pub const VIEWPORT_WIDTH: i32 = 800;
pub const VIEWPORT_HEIGHT: i32 = 600;

pub fn rasterize(triangles: &[[Vertex; 3]], mode: TriangleMode) -> Vec<Fragment> {
    let mut fragments = Vec::new();

    for tri in triangles {
        let triangle_fragments = match mode {
            TriangleMode::Wireframe => triangle_wireframe(&tri[0], &tri[1], &tri[2]),
            TriangleMode::FilledBarycentric => triangle_barycentric(&tri[0], &tri[1], &tri[2]),
            TriangleMode::FilledScanline => triangle_scanline(&tri[0], &tri[1], &tri[2]),
        };
        fragments.extend(triangle_fragments);
    }

    fragments
}
