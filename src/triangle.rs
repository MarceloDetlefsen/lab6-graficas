use raylib::prelude::Vector3;

use crate::fragment::Fragment;
use crate::rasterizer::{VIEWPORT_HEIGHT, VIEWPORT_WIDTH};
use crate::vertex::Vertex;

pub const DEGENERATE_EPS: f32 = 1e-6;

fn edge(p: (f32, f32), q: (f32, f32), r: (f32, f32)) -> f32 {
    (r.0 - p.0) * (q.1 - p.1) - (r.1 - p.1) * (q.0 - p.0)
}

pub fn is_degenerate(a: &Vertex, b: &Vertex, c: &Vertex) -> bool {
    let pa = (a.transformed_position.x, a.transformed_position.y);
    let pb = (b.transformed_position.x, b.transformed_position.y);
    let pc = (c.transformed_position.x, c.transformed_position.y);
    edge(pa, pb, pc).abs() <= DEGENERATE_EPS
}

pub fn triangle_wireframe(a: &Vertex, b: &Vertex, c: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    let edges = [
        (a, b, Vector3::new(1.0, 0.0, 0.0)),
        (b, c, Vector3::new(0.0, 1.0, 0.0)),
        (c, a, Vector3::new(0.0, 0.0, 1.0)),
    ];
    for (start, end, edge_color) in edges {
        let dx = end.transformed_position.x - start.transformed_position.x;
        let dy = end.transformed_position.y - start.transformed_position.y;
        let steps = dx.abs().max(dy.abs()).ceil() as usize;
        for step in 0..=steps {
            let t = if steps == 0 { 0.0 } else { step as f32 / steps as f32 };
            let p = start.transformed_position + (end.transformed_position - start.transformed_position) * t;
            fragments.push(Fragment::new(p.x, p.y, edge_color, p.z));
        }
    }
    fragments
}

/// Rellena usando centros de píxel y bordes inclusivos en y, x y aristas
/// horizontales. Incluir ambos extremos horizontales evita huecos; conservar
/// únicamente el intervalo mínimo-máximo evita duplicar píxeles cuando un
/// vértice pertenece a dos aristas.
pub fn triangle_scanline(a: &Vertex, b: &Vertex, c: &Vertex) -> Vec<Fragment> {
    // La interpolación de x puede redondear distinto a la función de arista
    // baricéntrica cuando el centro cae casi exactamente sobre un borde.
    if is_degenerate(a, b, c) {
        return Vec::new();
    }

    let points = [
        (a.transformed_position.x, a.transformed_position.y),
        (b.transformed_position.x, b.transformed_position.y),
        (c.transformed_position.x, c.transformed_position.y),
    ];
    let area = edge(points[0], points[1], points[2]);
    let inverse_area = 1.0 / area;
    let y_min = points[0].1.min(points[1].1).min(points[2].1);
    let y_max = points[0].1.max(points[1].1).max(points[2].1);
    if y_max < 0.5 || y_min > VIEWPORT_HEIGHT as f32 - 0.5 {
        return Vec::new();
    }
    let start_y = ((y_min - 0.5).ceil() as i32).clamp(0, VIEWPORT_HEIGHT - 1);
    let end_y = ((y_max - 0.5).floor() as i32).clamp(0, VIEWPORT_HEIGHT - 1);
    let depth = (a.transformed_position.z + b.transformed_position.z + c.transformed_position.z) / 3.0;
    let mut fragments = Vec::new();

    for y in start_y..=end_y {
        let yc = y as f32 + 0.5;
        let mut crossings = Vec::with_capacity(6);

        for (p0, p1) in [(points[0], points[1]), (points[1], points[2]), (points[2], points[0])] {
            let (x0, y0) = p0;
            let (x1, y1) = p1;
            if y0 == y1 {
                if yc == y0 {
                    crossings.push(x0);
                    crossings.push(x1);
                }
            } else if yc >= y0.min(y1) && yc <= y0.max(y1) {
                crossings.push(x0 + (yc - y0) * (x1 - x0) / (y1 - y0));
            }
        }

        if crossings.is_empty() {
            continue;
        }
        let x_left = crossings.iter().copied().fold(f32::INFINITY, f32::min);
        let x_right = crossings.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        if x_right < 0.5 || x_left > VIEWPORT_WIDTH as f32 - 0.5 {
            continue;
        }
        let x_start = ((x_left - 0.5).ceil() as i32).clamp(0, VIEWPORT_WIDTH - 1);
        let x_end = ((x_right - 0.5).floor() as i32).clamp(0, VIEWPORT_WIDTH - 1);

        if x_start <= x_end {
            for x in x_start..=x_end {
                let p = (x as f32 + 0.5, yc);
                let w_a = edge(points[1], points[2], p) * inverse_area;
                let w_b = edge(points[2], points[0], p) * inverse_area;
                let w_c = edge(points[0], points[1], p) * inverse_area;
                let color = a.color * w_a + b.color * w_b + c.color * w_c;
                fragments.push(Fragment::new(p.0, p.1, color, depth));
            }
        }
    }

    fragments
}

pub fn triangle_barycentric(a: &Vertex, b: &Vertex, c: &Vertex) -> Vec<Fragment> {
    // La función de arista puede redondear distinto a la interpolación de x
    // de scanline cuando el centro cae casi exactamente sobre un borde.
    if is_degenerate(a, b, c) {
        return Vec::new();
    }

    let pa = (a.transformed_position.x, a.transformed_position.y);
    let pb = (b.transformed_position.x, b.transformed_position.y);
    let pc = (c.transformed_position.x, c.transformed_position.y);
    let area = edge(pa, pb, pc);
    let inverse_area = 1.0 / area;

    let min_x = pa.0.min(pb.0).min(pc.0).floor() as i32;
    let max_x = pa.0.max(pb.0).max(pc.0).ceil() as i32;
    let min_y = pa.1.min(pb.1).min(pc.1).floor() as i32;
    let max_y = pa.1.max(pb.1).max(pc.1).ceil() as i32;

    let start_x = min_x.clamp(0, VIEWPORT_WIDTH - 1);
    let end_x = max_x.clamp(0, VIEWPORT_WIDTH - 1);
    let start_y = min_y.clamp(0, VIEWPORT_HEIGHT - 1);
    let end_y = max_y.clamp(0, VIEWPORT_HEIGHT - 1);

    let estimated_capacity = ((area.abs() * 0.5) as usize)
        .min((VIEWPORT_WIDTH * VIEWPORT_HEIGHT) as usize);
    let mut fragments = Vec::with_capacity(estimated_capacity);
    let depth = (a.transformed_position.z + b.transformed_position.z + c.transformed_position.z) / 3.0;

    for y in start_y..=end_y {
        for x in start_x..=end_x {
            // Se evalúa el centro del píxel y los bordes son inclusivos.
            let p = (x as f32 + 0.5, y as f32 + 0.5);
            let w_a = edge(pb, pc, p) * inverse_area;
            let w_b = edge(pc, pa, p) * inverse_area;
            let w_c = edge(pa, pb, p) * inverse_area;

            if w_a >= 0.0 && w_b >= 0.0 && w_c >= 0.0 {
                // El color se interpola entre los tres vértices; la profundidad
                // conserva el promedio hasta implementar interpolación de depth.
                let color = a.color * w_a + b.color * w_b + c.color * w_c;
                fragments.push(Fragment::new(x as f32 + 0.5, y as f32 + 0.5, color, depth));
            }
        }
    }

    fragments
}
