#![allow(dead_code)]

use raylib::prelude::Vector3;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Obj {
    pub vertices: Vec<Vector3>,
    pub vertex_normals: Vec<Vector3>,
    pub source_normals: Vec<Vector3>,
    pub indices: Vec<usize>,
}

impl Obj {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            vertex_normals: Vec::new(),
            source_normals: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn load(path: &str) -> io::Result<Self> {
        load_obj(path)
    }

    /// Calcula la caja envolvente (bounding box) del modelo devolviendo (min, max).
    pub fn bounds(&self) -> (Vector3, Vector3) {
        if self.vertices.is_empty() {
            return (Vector3::zero(), Vector3::zero());
        }

        let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);

        for v in &self.vertices {
            min.x = min.x.min(v.x);
            min.y = min.y.min(v.y);
            min.z = min.z.min(v.z);

            max.x = max.x.max(v.x);
            max.y = max.y.max(v.y);
            max.z = max.z.max(v.z);
        }

        (min, max)
    }
}

/// Carga un archivo OBJ reconociendo vértices, normales fuente y caras.
pub fn load_obj(path: &str) -> io::Result<Obj> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut vertices = Vec::new();
    let mut source_normals = Vec::new();
    let mut indices = Vec::new();
    let mut faces = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        // Ignorar líneas vacías o comentarios
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let prefix = match parts.next() {
            Some(p) => p,
            None => continue,
        };

        match prefix {
            "v" => {
                if let (Some(x_str), Some(y_str), Some(z_str)) =
                    (parts.next(), parts.next(), parts.next())
                {
                    if let (Ok(x), Ok(y), Ok(z)) = (
                        x_str.parse::<f32>(),
                        y_str.parse::<f32>(),
                        z_str.parse::<f32>(),
                    ) {
                        vertices.push(Vector3::new(x, y, z));
                    }
                }
            }
            "vn" => {
                if let (Some(x_str), Some(y_str), Some(z_str)) =
                    (parts.next(), parts.next(), parts.next())
                {
                    if let (Ok(x), Ok(y), Ok(z)) = (
                        x_str.parse::<f32>(),
                        y_str.parse::<f32>(),
                        z_str.parse::<f32>(),
                    ) {
                        source_normals.push(Vector3::new(x, y, z));
                    }
                }
            }
            "f" => {
                let mut face = Vec::new();
                for part in parts {
                    let vertex_idx_str = part.split('/').next().unwrap_or("");
                    if let Ok(idx_1based) = vertex_idx_str.parse::<usize>() {
                        if idx_1based > 0 {
                            let index = idx_1based - 1;
                            indices.push(index);
                            face.push(index);
                        }
                    }
                }
                if face.len() >= 3 {
                    faces.push(face);
                }
            }
            _ => {
                // Las coordenadas de textura y metadatos no son necesarios aquí.
            }
        }
    }

    let centroid = vertices
        .iter()
        .copied()
        .fold(Vector3::zero(), |sum, position| sum + position)
        * (1.0 / vertices.len().max(1) as f32);
    let centered_at_origin = centroid.length() <= 1e-5;
    let vertex_normals = if centered_at_origin {
        vertices
            .iter()
            .map(|position| position.normalize())
            .collect()
    } else {
        let mut accumulated = vec![Vector3::zero(); vertices.len()];
        for face in faces {
            let [a, b, c] = [face[0], face[1], face[2]];
            if a >= vertices.len() || b >= vertices.len() || c >= vertices.len() {
                continue;
            }
            let face_normal = (vertices[b] - vertices[a]).cross(vertices[c] - vertices[a]);
            for &index in &face {
                if index < accumulated.len() {
                    accumulated[index] += face_normal;
                }
            }
        }
        accumulated
            .into_iter()
            .map(|normal| normal.normalize())
            .collect()
    };

    Ok(Obj {
        vertices,
        vertex_normals,
        source_normals,
        indices,
    })
}

#[cfg(test)]
mod tests {
    use super::load_obj;

    #[test]
    fn sphere_counts_and_smooth_normals_are_valid() {
        let obj = load_obj("assets/sphere.obj").expect("sphere OBJ should load");

        assert_eq!(obj.vertices.len(), 98);
        assert_eq!(obj.source_normals.len(), 192);
        assert_eq!(obj.indices.len(), 192 * 3);
        assert_eq!(obj.vertex_normals.len(), obj.vertices.len());
        assert!(obj
            .vertex_normals
            .iter()
            .all(|normal| (normal.length() - 1.0).abs() <= 1e-4));
    }
}
