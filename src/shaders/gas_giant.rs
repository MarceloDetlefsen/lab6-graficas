#![allow(dead_code)]

use raylib::prelude::Vector3;
use std::sync::Once;

use crate::fragment::Fragment;
use crate::noise;
use crate::shaders::{diffuse_factor, surface_mu};
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
// Centro normalizado del vórtice, ubicado hacia el hemisferio frontal (-z).
pub const VORTEX_CENTER: [f32; 3] = [0.20, 0.12, -0.97];
// Radio angular aproximado del gran vórtice naranja.
pub const VORTEX_RADIUS: f32 = 0.45;
// Giro de las coordenadas dentro del vórtice; valores altos enrollan más las bandas.
pub const VORTEX_STRENGTH: f32 = 4.5;
// Frecuencia del ruido que rompe el borde circular del vórtice.
pub const VORTEX_EDGE_FREQ: f32 = 4.5;
// Verde muy pálido de las zonas de patrón más claras.
pub const COLOR_GREEN_PALE: [f32; 3] = [0.75, 0.95, 0.55];
// Verde brillante dominante.
pub const COLOR_GREEN_BRIGHT: [f32; 3] = [0.35, 0.85, 0.15];
// Verde oliva de las bandas intermedias.
pub const COLOR_GREEN_OLIVE: [f32; 3] = [0.25, 0.40, 0.10];
// Verde profundo de las sombras.
pub const COLOR_GREEN_DARK: [f32; 3] = [0.05, 0.15, 0.05];
// Naranja rojizo del vórtice.
pub const COLOR_VORTEX: [f32; 3] = [0.95, 0.40, 0.10];
// Sombra rojiza interna que da volumen y lectura líquida al vórtice.
pub const COLOR_VORTEX_DARK: [f32; 3] = [0.48, 0.10, 0.035];
// Frecuencia del marmoleado interno del vórtice.
pub const VORTEX_DETAIL_FREQ: f32 = 7.0;
// Ambiente mínimo para conservar detalle en el lado oscuro.
pub const AMBIENT: f32 = 0.12;
// Intensidad del tinte verde brillante en el borde atmosférico.
pub const RIM_INTENSITY: f32 = 0.28;
// Velocidad temporal del primer warp, lenta para mover masas atmosféricas amplias.
pub const WARP1_SPEED: f32 = 0.035;
// Velocidad temporal del segundo warp, más rápida para mezclar el marmoleado.
pub const WARP2_SPEED: f32 = 0.085;
// Velocidad de rotación propia del vórtice.
pub const VORTEX_SPIN: f32 = 0.060;

static PRINTED_STATS: Once = Once::new();

fn domain_warp(object_pos: Vector3, time: f32) -> Vector3 {
    let p = vortex_domain(object_pos.normalize(), time);
    let warp1 = noise::fbm4(p * WARP1_FREQ, time * WARP1_SPEED, 3, 2.0, 0.5) * 2.0 - 1.0;
    let first = p + Vector3::new(warp1, warp1 * 0.72, -warp1 * 0.58) * WARP1_AMP;
    let warp2 = noise::fbm4(first * WARP2_FREQ, time * WARP2_SPEED, 3, 2.0, 0.5) * 2.0 - 1.0;
    first + Vector3::new(-warp2 * 0.62, warp2, warp2 * 0.48) * WARP2_AMP
}

fn vortex_domain(p: Vector3, time: f32) -> Vector3 {
    let center = Vector3::new(
        VORTEX_CENTER[0],
        VORTEX_CENTER[1],
        VORTEX_CENTER[2],
    )
    .normalize();
    let offset = p - center;
    let distance = offset.length();
    let falloff = (1.0 - distance / VORTEX_RADIUS).clamp(0.0, 1.0);
    if falloff <= 0.0 {
        return p;
    }

    let angle = VORTEX_STRENGTH * falloff * falloff + time * VORTEX_SPIN * falloff;
    let (sin_angle, cos_angle) = angle.sin_cos();
    let cross = Vector3::new(
        center.y * offset.z - center.z * offset.y,
        center.z * offset.x - center.x * offset.z,
        center.x * offset.y - center.y * offset.x,
    );
    offset * cos_angle + cross * sin_angle + center * center.dot(offset) * (1.0 - cos_angle)
}

fn raw_pattern(object_pos: Vector3, time: f32) -> f32 {
    let warped = domain_warp(object_pos, time);
    let latitude = warped.y.clamp(-1.0, 1.0);
    let bands = (latitude * BAND_FREQ).sin() * 0.5 + 0.5;
    let fluid_noise = noise::fbm(warped * 2.2, 3, 2.0, 0.5);
    bands * 0.68 + fluid_noise * 0.32
}

pub fn band_pattern_at(object_pos: Vector3, time: f32) -> f32 {
    noise::remap(raw_pattern(object_pos, time), PATTERN_LOW, PATTERN_HIGH, 0.0, 1.0)
        .clamp(0.0, 1.0)
}

pub fn band_pattern(object_pos: Vector3) -> f32 {
    band_pattern_at(object_pos, 0.0)
}

fn color(values: [f32; 3]) -> Vector3 {
    Vector3::new(values[0], values[1], values[2])
}

fn palette_color(pattern: f32) -> Vector3 {
    let value = pattern.clamp(0.0, 1.0);
    if value < 0.33 {
        noise::mix_color(
            color(COLOR_GREEN_DARK),
            color(COLOR_GREEN_OLIVE),
            noise::smoothstep(0.0, 0.33, value),
        )
    } else if value < 0.68 {
        noise::mix_color(
            color(COLOR_GREEN_OLIVE),
            color(COLOR_GREEN_BRIGHT),
            noise::smoothstep(0.33, 0.68, value),
        )
    } else {
        noise::mix_color(
            color(COLOR_GREEN_BRIGHT),
            color(COLOR_GREEN_PALE),
            noise::smoothstep(0.68, 1.0, value),
        )
    }
}

fn vortex_mask(object_pos: Vector3) -> f32 {
    let p = object_pos.normalize();
    let center = Vector3::new(
        VORTEX_CENTER[0],
        VORTEX_CENTER[1],
        VORTEX_CENTER[2],
    )
    .normalize();
    let distance = (p - center).length();
    let radial = 1.0 - noise::smoothstep(VORTEX_RADIUS * 0.68, VORTEX_RADIUS, distance);
    let edge_noise = noise::fbm(p * VORTEX_EDGE_FREQ, 3, 2.0, 0.5);
    let irregular_edge = noise::smoothstep(
        0.24,
        0.72,
        radial + (edge_noise - 0.5) * 0.42,
    );
    irregular_edge * radial
}

fn surface_color(object_pos: Vector3, pattern: f32, time: f32) -> Vector3 {
    let base = palette_color(pattern);
    let vortex = vortex_mask(object_pos);
    let detail = noise::fbm4(
        domain_warp(object_pos, time) * VORTEX_DETAIL_FREQ,
        time * WARP2_SPEED,
        3,
        2.0,
        0.5,
    );
    let vortex_color = noise::mix_color(
        color(COLOR_VORTEX_DARK),
        color(COLOR_VORTEX),
        noise::smoothstep(0.30, 0.70, detail),
    );
    noise::mix_color(base, vortex_color, vortex)
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
            values.push(raw_pattern(point, 0.0));
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

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    print_statistics_once();
    let pattern = band_pattern_at(fragment.object_pos, uniforms.time);
    if uniforms.rocky_debug_view == 0 {
        return Vector3::new(pattern, pattern, pattern);
    }

    let mut result = surface_color(fragment.object_pos, pattern, uniforms.time);
    let diffuse = diffuse_factor(fragment.normal, uniforms);
    let mu = surface_mu(fragment, uniforms);
    let limb = 0.22 + 0.78 * mu.powf(0.72);
    result *= AMBIENT + (1.0 - AMBIENT) * diffuse;
    result *= limb;

    let rim = (1.0 - mu).powf(3.0) * RIM_INTENSITY;
    result += Vector3::new(0.18, 0.90, 0.35) * rim;
    result
}
