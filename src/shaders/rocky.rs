#![allow(dead_code)]

use raylib::prelude::Vector3;
use std::sync::Once;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::noise;
use crate::shaders::{diffuse_factor, surface_view};

// Percentil inferior usado para remapear el campo y evitar una masa gris plana.
pub const H_LOW: f32 = 0.507;
// Percentil superior usado para remapear el campo y reservar contraste para vetas.
pub const H_HIGH: f32 = 0.826;
// Factor anisotrópico: valores mayores alargan las vetas sobre el eje Y.
pub const VEIN_STRETCH: f32 = 2.8;
// Amplitud del domain warp; valores altos curvan pero también pueden disolver vetas.
pub const WARP_AMP: f32 = 0.14;
// Frecuencia del ruido que deforma el dominio.
pub const WARP_FREQ: f32 = 1.25;
// Frecuencia de la máscara de vegetación de baja frecuencia.
pub const BIOME_FREQ: f32 = 1.8;
// Umbral alto que limita la vegetación a regiones aisladas.
pub const BIOME_THRESHOLD: f32 = 0.52;
// Suavidad del umbral de vegetación.
pub const BIOME_SOFTNESS: f32 = 0.06;
// Umbral de las crestas finas que se muestran como grietas.
pub const CRACK_THRESHOLD: f32 = 0.84;
// Suavidad del umbral de grietas.
pub const CRACK_SOFTNESS: f32 = 0.05;
// Octavas máximas para mantener usable el renderer en CPU.
pub const TERRAIN_OCTAVES: usize = 4;
// Agudeza de cada cresta ridged; valores mayores producen vetas más finas.
pub const RIDGE_SHARPNESS: f32 = 2.5;
// Amplitud de la capa de detalle de alta frecuencia.
pub const RIDGE_DETAIL_AMP: f32 = 0.18;
// Nivel de altura por debajo del cual aparecen los mares ácidos.
pub const SEA_LEVEL: f32 = 0.15;
// Color de las cuencas más bajas.
pub const COLOR_SEA: [f32; 3] = [0.10, 0.75, 0.55];
// Color de la roca ámbar en alturas bajas.
pub const COLOR_AMBER: [f32; 3] = [0.45, 0.22, 0.05];
// Color de la roca naranja en alturas medias-altas.
pub const COLOR_BURNT_ORANGE: [f32; 3] = [0.85, 0.40, 0.08];
// Color de las crestas altas y oscuras.
pub const COLOR_BLUE_VIOLET: [f32; 3] = [0.18, 0.12, 0.32];
// Color base del bioma Brinstar.
pub const COLOR_TOXIC_GREEN: [f32; 3] = [0.35, 0.85, 0.20];
// Toque rosado mezclado dentro del bioma por ruido fino.
pub const COLOR_PINK: [f32; 3] = [0.90, 0.35, 0.60];
// Color de las grietas, todavía sin emisión propia en esta fase.
pub const COLOR_CRACK: [f32; 3] = [1.00, 0.55, 0.10];
// Iluminación mínima para conservar detalle en el lado oscuro.
pub const AMBIENT: f32 = 0.15;
// Separación de las muestras de altura usadas para estimar el gradiente.
pub const BUMP_DELTA: f32 = 0.002;
// Fuerza del relieve sobre la normal; valores altos hacen más marcado el modelado.
pub const BUMP_STRENGTH: f32 = 1.8;
// Exponente del reflejo concentrado de los mares ácidos.
pub const SEA_SPEC_POWER: f32 = 48.0;
// Intensidad máxima del reflejo especular del mar.
pub const SEA_SPEC_STRENGTH: f32 = 0.8;
// Emisión máxima de las grietas en la cara nocturna.
pub const LAVA_GLOW: f32 = 0.75;

static PRINTED_STATS: Once = Once::new();

fn raw_height_field(object_pos: Vector3) -> f32 {
    let p = object_pos.normalize();
    let warp = noise::fbm(p * WARP_FREQ, 3, 2.0, 0.5) * 2.0 - 1.0;
    let warped = Vector3::new(
        p.x + warp * WARP_AMP,
        p.y / VEIN_STRETCH + warp * WARP_AMP * 0.5,
        p.z + warp * WARP_AMP,
    );
    let broad_ridges = noise::ridged(warped, TERRAIN_OCTAVES, 2.0, 0.5).powf(RIDGE_SHARPNESS);
    let fine_ridges = noise::ridged(warped * 3.2, 3, 2.0, 0.5).powf(RIDGE_SHARPNESS);
    (broad_ridges * (1.0 - RIDGE_DETAIL_AMP) + fine_ridges * RIDGE_DETAIL_AMP).clamp(0.0, 1.0)
}

fn fine_ridge_field(object_pos: Vector3) -> f32 {
    let p = object_pos.normalize();
    let warp = noise::fbm(p * WARP_FREQ, 3, 2.0, 0.5) * 2.0 - 1.0;
    let warped = Vector3::new(
        p.x + warp * WARP_AMP,
        p.y / VEIN_STRETCH + warp * WARP_AMP * 0.5,
        p.z + warp * WARP_AMP,
    );
    noise::ridged(warped * 3.2, 3, 2.0, 0.5).powf(RIDGE_SHARPNESS)
}

pub fn height_field(object_pos: Vector3) -> f32 {
    noise::remap(raw_height_field(object_pos), H_LOW, H_HIGH, 0.0, 1.0).clamp(0.0, 1.0)
}

pub fn biome_mask(object_pos: Vector3) -> f32 {
    let value = noise::fbm(object_pos.normalize() * BIOME_FREQ, 3, 2.0, 0.5);
    noise::smoothstep(BIOME_THRESHOLD, BIOME_THRESHOLD + BIOME_SOFTNESS, value)
}

pub fn crack_mask(object_pos: Vector3) -> f32 {
    let fine_ridge = fine_ridge_field(object_pos);
    noise::smoothstep(
        CRACK_THRESHOLD,
        CRACK_THRESHOLD + CRACK_SOFTNESS,
        fine_ridge,
    )
}

fn percentile(sorted: &[f32], fraction: f32) -> f32 {
    sorted[(fraction * (sorted.len() - 1) as f32) as usize]
}

fn print_statistics_once() {
    PRINTED_STATS.call_once(|| {
        let sample_count = 4096;
        let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
        let mut raw_values = Vec::with_capacity(sample_count);
        let mut biome_values = Vec::with_capacity(sample_count);
        let mut fine_ridges = Vec::with_capacity(sample_count);
        let mut height_values = Vec::with_capacity(sample_count);

        for i in 0..sample_count {
            let y = 1.0 - (i as f32 + 0.5) * 2.0 / sample_count as f32;
            let radius = (1.0 - y * y).sqrt();
            let angle = golden_angle * i as f32;
            let point = Vector3::new(radius * angle.cos(), y, radius * angle.sin());
            let raw = raw_height_field(point);
            raw_values.push(raw);
            height_values.push(height_field(point));
            biome_values.push(biome_mask(point));
            fine_ridges.push(fine_ridge_field(point));
        }

        raw_values.sort_by(f32::total_cmp);
        println!(
            "Rocky h raw: min={:.3}, p05={:.3}, p25={:.3}, p50={:.3}, p75={:.3}, p95={:.3}, max={:.3}",
            raw_values[0],
            percentile(&raw_values, 0.05),
            percentile(&raw_values, 0.25),
            percentile(&raw_values, 0.50),
            percentile(&raw_values, 0.75),
            percentile(&raw_values, 0.95),
            raw_values[raw_values.len() - 1],
        );
        biome_values.sort_by(f32::total_cmp);
        fine_ridges.sort_by(f32::total_cmp);
        height_values.sort_by(f32::total_cmp);
        println!(
            "Rocky bioma mask: p50={:.3}, p75={:.3}, p95={:.3}, max={:.3}",
            percentile(&biome_values, 0.50),
            percentile(&biome_values, 0.75),
            percentile(&biome_values, 0.95),
            biome_values[biome_values.len() - 1],
        );
        let biome_coverage = biome_values.iter().filter(|&&value| value > 0.5).count();
        let crack_coverage = fine_ridges
            .iter()
            .filter(|&&value| noise::smoothstep(CRACK_THRESHOLD, CRACK_THRESHOLD + CRACK_SOFTNESS, value) > 0.5)
            .count();
        let sea_coverage = height_values.iter().filter(|&&value| value < SEA_LEVEL).count();
        let low_height_coverage = height_values.iter().filter(|&&value| value < 0.2).count();
        println!(
            "Rocky cobertura: mar={:.1}%, bioma={:.1}%, grietas={:.1}%, h<0.2={:.1}%",
            sea_coverage as f32 * 100.0 / sample_count as f32,
            biome_coverage as f32 * 100.0 / sample_count as f32,
            crack_coverage as f32 * 100.0 / sample_count as f32,
            low_height_coverage as f32 * 100.0 / sample_count as f32,
        );
    });
}

fn color(values: [f32; 3]) -> Vector3 {
    Vector3::new(values[0], values[1], values[2])
}

fn height_color(height: f32) -> Vector3 {
    if height < SEA_LEVEL {
        color(COLOR_SEA)
    } else if height < 0.45 {
        noise::mix_color(
            color(COLOR_AMBER),
            color(COLOR_BURNT_ORANGE),
            noise::smoothstep(SEA_LEVEL, 0.45, height),
        )
    } else {
        noise::mix_color(
            color(COLOR_BURNT_ORANGE),
            color(COLOR_BLUE_VIOLET),
            noise::smoothstep(0.45, 1.0, height),
        )
    }
}

fn surface_color(object_pos: Vector3, height: f32) -> Vector3 {
    let base = height_color(height);
    let biome = biome_mask(object_pos);
    let pink_noise = noise::fbm(object_pos.normalize() * 8.0, 3, 2.0, 0.5);
    let pink_amount = noise::smoothstep(0.55, 0.72, pink_noise);
    let biome_color = noise::mix_color(color(COLOR_TOXIC_GREEN), color(COLOR_PINK), pink_amount);
    let with_biome = noise::mix_color(base, biome_color, biome);
    noise::mix_color(with_biome, color(COLOR_CRACK), crack_mask(object_pos))
}

fn model_normal(model: &raylib::prelude::Matrix, normal: Vector3) -> Vector3 {
    Vector3::new(
        model.m0 * normal.x + model.m4 * normal.y + model.m8 * normal.z,
        model.m1 * normal.x + model.m5 * normal.y + model.m9 * normal.z,
        model.m2 * normal.x + model.m6 * normal.y + model.m10 * normal.z,
    )
    .normalize()
}

fn bumped_normal(fragment: &Fragment, uniforms: &Uniforms, height: f32) -> Vector3 {
    let p = fragment.object_pos.normalize();
    let dx = Vector3::new(BUMP_DELTA, 0.0, 0.0);
    let dy = Vector3::new(0.0, BUMP_DELTA, 0.0);
    let dz = Vector3::new(0.0, 0.0, BUMP_DELTA);
    let gradient = Vector3::new(
        (height_field(p + dx) - height) / BUMP_DELTA,
        (height_field(p + dy) - height) / BUMP_DELTA,
        (height_field(p + dz) - height) / BUMP_DELTA,
    );
    // La altura aumenta en la dirección del gradiente; la normal de una superficie
    // elevada se inclina contra él. Este signo mantiene las crestas claras hacia la luz.
    let object_normal = (p - gradient * BUMP_STRENGTH).normalize();
    model_normal(&uniforms.model_matrix, object_normal)
}

fn lit_surface_color(
    fragment: &Fragment,
    uniforms: &Uniforms,
    height: f32,
    base: Vector3,
) -> Vector3 {
    let normal = if height < SEA_LEVEL {
        fragment.normal.normalize()
    } else {
        bumped_normal(fragment, uniforms, height)
    };
    let diffuse = diffuse_factor(normal, uniforms);
    let mut result = base * (AMBIENT + (1.0 - AMBIENT) * diffuse);

    let sea_edge = noise::smoothstep(SEA_LEVEL, SEA_LEVEL + 0.12, height);
    let view = surface_view(fragment, uniforms);
    let half_vector = (uniforms.light_dir + view).normalize();
    let specular = normal
        .dot(half_vector)
        .max(0.0)
        .powf(SEA_SPEC_POWER)
        * SEA_SPEC_STRENGTH
        * (1.0 - sea_edge);
    result += Vector3::new(1.0, 1.0, 1.0) * specular;

    let crack = crack_mask(fragment.object_pos);
    let n_dot_l = diffuse_factor(fragment.normal, uniforms);
    result += color(COLOR_CRACK) * (crack * LAVA_GLOW * (1.0 - n_dot_l));
    result
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    print_statistics_once();
    let height = height_field(fragment.object_pos);
    match uniforms.rocky_debug_view {
        1 => {
            let value = biome_mask(fragment.object_pos);
            Vector3::new(value, value, value)
        }
        2 => {
            let value = crack_mask(fragment.object_pos);
            Vector3::new(value, value, value)
        }
        3 => {
            let base = surface_color(fragment.object_pos, height);
            lit_surface_color(fragment, uniforms, height, base)
        }
        _ => Vector3::new(height, height, height),
    }
}
