#![allow(dead_code)]

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use raylib::prelude::Vector3;

#[derive(Debug, Clone)]
pub struct Obj {
    pub vertices: Vec<Vector3>,
    pub indices: Vec<usize>,
}

impl Obj {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
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

/// Carga un archivo OBJ reconociendo vértices ('v') y caras ('f').
pub fn load_obj(path: &str) -> io::Result<Obj> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

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
                // Paso 3: Cargar vértices (x, y, z)
                if let (Some(x_str), Some(y_str), Some(z_str)) = (parts.next(), parts.next(), parts.next()) {
                    if let (Ok(x), Ok(y), Ok(z)) = (
                        x_str.parse::<f32>(),
                        y_str.parse::<f32>(),
                        z_str.parse::<f32>(),
                    ) {
                        vertices.push(Vector3::new(x, y, z));
                    }
                }
            }
            "f" => {
                // Paso 4: Cargar caras e índices
                // Soporta formatos: "f 1 2 3", "f 1/2/3 4/5/6 7/8/9", "f 1//3 ...", etc.
                for part in parts {
                    // Tomar únicamente el primer valor antes del primer '/'
                    let vertex_idx_str = part.split('/').next().unwrap_or("");
                    if let Ok(idx_1based) = vertex_idx_str.parse::<usize>() {
                        // El formato OBJ comienza en 1; Rust Vec en 0
                        if idx_1based > 0 {
                            indices.push(idx_1based - 1);
                        }
                    }
                }
            }
            _ => {
                // Ignorar normales (vn), texturas (vt), materiales, objetos, etc.
            }
        }
    }

    Ok(Obj { vertices, indices })
}
