#![allow(dead_code)]

use noise::{Fbm, MultiFractal, NoiseFn, OpenSimplex};
use raylib::prelude::Vector3;
use std::sync::OnceLock;

struct NoiseTools {
    source: Fbm<OpenSimplex>,
}

static NOISE: OnceLock<NoiseTools> = OnceLock::new();

fn noise_tools() -> &'static NoiseTools {
    NOISE.get_or_init(|| NoiseTools {
        source: Fbm::<OpenSimplex>::new(0).set_octaves(1),
    })
}

fn normalize_noise(value: f64) -> f32 {
    ((value * 0.5 + 0.5) as f32).clamp(0.0, 1.0)
}

pub fn noise3(p: Vector3, freq: f32) -> f32 {
    let source = &noise_tools().source;
    normalize_noise(source.get([
        p.x as f64 * freq as f64,
        p.y as f64 * freq as f64,
        p.z as f64 * freq as f64,
    ]))
}

pub fn noise4(p: Vector3, t: f32, freq: f32) -> f32 {
    let source = &noise_tools().source;
    normalize_noise(source.get([
        p.x as f64 * freq as f64,
        p.y as f64 * freq as f64,
        p.z as f64 * freq as f64,
        t as f64,
    ]))
}

fn fractal<F>(octaves: usize, lacunarity: f32, gain: f32, mut sample: F) -> f32
where
    F: FnMut(f32) -> f32,
{
    let octaves = octaves.max(1);
    let lacunarity = lacunarity.max(0.0);
    let gain = gain.max(0.0);
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut total = 0.0;
    let mut weight = 0.0;

    for _ in 0..octaves {
        total += sample(frequency) * amplitude;
        weight += amplitude;
        frequency *= lacunarity;
        amplitude *= gain;
    }

    if weight > 0.0 {
        (total / weight).clamp(0.0, 1.0)
    } else {
        0.5
    }
}

pub fn fbm(p: Vector3, octaves: usize, lacunarity: f32, gain: f32) -> f32 {
    fractal(octaves, lacunarity, gain, |frequency| noise3(p, frequency))
}

pub fn fbm4(p: Vector3, t: f32, octaves: usize, lacunarity: f32, gain: f32) -> f32 {
    fractal(octaves, lacunarity, gain, |frequency| noise4(p, t, frequency))
}

pub fn ridged(p: Vector3, octaves: usize, lacunarity: f32, gain: f32) -> f32 {
    fractal(octaves, lacunarity, gain, |frequency| {
        1.0 - (noise3(p, frequency) * 2.0 - 1.0).abs()
    })
}

pub fn turbulence(p: Vector3, octaves: usize, lacunarity: f32, gain: f32) -> f32 {
    fractal(octaves, lacunarity, gain, |frequency| {
        (noise3(p, frequency) * 2.0 - 1.0).abs()
    })
}

pub fn clamp01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

pub fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = clamp01((x - a) / (b - a));
    t * t * (3.0 - 2.0 * t)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn mix_color(a: Vector3, b: Vector3, t: f32) -> Vector3 {
    a + (b - a) * t
}

pub fn remap(x: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    if (b - a).abs() <= f32::EPSILON {
        c
    } else {
        lerp(c, d, (x - a) / (b - a))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_helpers_stay_in_normalized_range() {
        let p = Vector3::new(0.37, -1.2, 2.4);
        for value in [
            noise3(p, 2.0),
            noise4(p, 1.5, 2.0),
            fbm(p, 5, 2.0, 0.5),
            fbm4(p, 1.5, 5, 2.0, 0.5),
            ridged(p, 5, 2.0, 0.5),
            turbulence(p, 5, 2.0, 0.5),
        ] {
            assert!((0.0..=1.0).contains(&value));
        }
    }
}
