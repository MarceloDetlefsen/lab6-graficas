#![allow(dead_code)]

use raylib::prelude::Vector3;
use std::sync::Once;

use crate::fragment::Fragment;
use crate::noise;
use crate::Uniforms;

// Frecuencia del seno latitudinal; valores altos producen más bandas.
pub const BAND_FREQ: f32 = 9.0;
// Frecuencia del primer warp, responsable de curvas amplias.
pub const WARP1_FREQ: f32 = 1.35;
// Amplitud del primer warp.
pub const WARP1_AMP: f32 = 0.34;
// Frecuencia del segundo warp, responsable del marmoleado líquido fino.
pub const WARP2_FREQ: f32 = 3.8;
// Amplitud menor de la segunda pasada de warp.
pub const WARP2_AMP: f32 = 0.13;
// Percentil bajo del patrón usado para expandir el contraste a [0,1].
pub const PATTERN_LOW: f32 = 0.18;
// Percentil alto del patrón usado para expandir el contraste a [0,1].
pub const PATTERN_HIGH: f32 = 0.82;

static PRINTED_STATS: Once = Once::new();

fn domain_warp(object_pos: Vector3) -> Vector3 {
    let p = object_pos.normalize();
    let warp1 = noise::fbm(p * WARP1_FREQ, 3, 2.0, 0.5) * 2.0 - 1.0;
    let first = p + Vector3::new(warp1, warp1 * 0.72, -warp1 * 0.58) * WARP1_AMP;
    let warp2 = noise::fbm(first * WARP2_FREQ, 3, 2.0, 0.5) * 2.0 - 1.0;
    first + Vector3::new(-warp2 * 0.62, warp2, warp2 * 0.48) * WARP2_AMP
}

fn raw_pattern(object_pos: Vector3) -> f32 {
    let warped = domain_warp(object_pos);
    let latitude = warped.y.clamp(-1.0, 1.0);
    let bands = (latitude * BAND_FREQ).sin() * 0.5 + 0.5;
    let fluid_noise = noise::fbm(warped * 2.2, 3, 2.0, 0.5);
    bands * 0.68 + fluid_noise * 0.32
}

pub fn band_pattern(object_pos: Vector3) -> f32 {
    noise::remap(raw_pattern(object_pos), PATTERN_LOW, PATTERN_HIGH, 0.0, 1.0)
        .clamp(0.0, 1.0)
}

fn percentile(sorted: &[f32], fraction: f32) -> f32 {
    sorted[(fraction * (sorted.len() - 1) as f32) as usize]
}

fn print_statistics_once() {
    PRINTED_STATS.call_once(|| {
        let sample_count = 4096;
        let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
        let mut values = Vec::with_capacity(sample_count);
        for i in 0..sample_count {
            let y = 1.0 - (i as f32 + 0.5) * 2.0 / sample_count as f32;
            let radius = (1.0 - y * y).sqrt();
            let angle = golden_angle * i as f32;
            let point = Vector3::new(radius * angle.cos(), y, radius * angle.sin());
            values.push(raw_pattern(point));
        }
        values.sort_by(f32::total_cmp);
        println!(
            "Gas pattern: min={:.3}, p05={:.3}, p50={:.3}, p95={:.3}, max={:.3}",
            values[0],
            percentile(&values, 0.05),
            percentile(&values, 0.50),
            percentile(&values, 0.95),
            values[values.len() - 1],
        );
    });
}

pub fn shade(fragment: &Fragment, _uniforms: &Uniforms) -> Vector3 {
    print_statistics_once();
    let value = band_pattern(fragment.object_pos);
    Vector3::new(value, value, value)
}
