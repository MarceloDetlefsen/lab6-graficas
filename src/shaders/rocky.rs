#![allow(dead_code)]

use raylib::prelude::Vector3;
use std::sync::Once;

use crate::fragment::Fragment;
use crate::noise;
use crate::Uniforms;

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

static PRINTED_STATS: Once = Once::new();

fn raw_height_field(object_pos: Vector3) -> f32 {
    let p = object_pos.normalize();
    let warp = noise::fbm(p * WARP_FREQ, 3, 2.0, 0.5) * 2.0 - 1.0;
    let warped = Vector3::new(
        p.x + warp * WARP_AMP,
        p.y / VEIN_STRETCH + warp * WARP_AMP * 0.5,
        p.z + warp * WARP_AMP,
    );
    let broad_ridges = noise::ridged(warped, TERRAIN_OCTAVES, 2.0, 0.5)
        .powf(RIDGE_SHARPNESS);
    let fine_ridges = noise::ridged(warped * 3.2, 3, 2.0, 0.5)
        .powf(RIDGE_SHARPNESS);
    (broad_ridges * (1.0 - RIDGE_DETAIL_AMP) + fine_ridges * RIDGE_DETAIL_AMP)
        .clamp(0.0, 1.0)
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
    noise::smoothstep(CRACK_THRESHOLD, CRACK_THRESHOLD + CRACK_SOFTNESS, fine_ridge)
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

        for i in 0..sample_count {
            let y = 1.0 - (i as f32 + 0.5) * 2.0 / sample_count as f32;
            let radius = (1.0 - y * y).sqrt();
            let angle = golden_angle * i as f32;
            let point = Vector3::new(radius * angle.cos(), y, radius * angle.sin());
            let raw = raw_height_field(point);
            raw_values.push(raw);
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
        println!("Rocky cobertura: bioma={:.1}%, grietas={:.1}%", biome_coverage as f32 * 100.0 / sample_count as f32, crack_coverage as f32 * 100.0 / sample_count as f32);
    });
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    print_statistics_once();
    let height = height_field(fragment.object_pos);
    let value = match uniforms.rocky_debug_view {
        1 => biome_mask(fragment.object_pos),
        2 => crack_mask(fragment.object_pos),
        _ => height,
    };
    Vector3::new(value, value, value)
}
