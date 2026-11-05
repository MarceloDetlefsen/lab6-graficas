#![allow(dead_code)]

use raylib::prelude::Vector3;

use crate::fragment::Fragment;
use crate::noise;
use crate::Uniforms;

// Frecuencia espacial de la granulación: valores altos producen grano más fino.
pub const GRANULATION_FREQ: f32 = 18.0;
// Reservada para la animación de la Fase 2: velocidad temporal del hervor.
pub const GRANULATION_SPEED: f32 = 1.0;
// Amplitud de la granulación alrededor de la temperatura base.
pub const GRANULATION_AMP: f32 = 0.12;
// Frecuencia espacial de las manchas: valores bajos producen manchas grandes.
pub const SPOT_FREQ: f32 = 2.5;
// Reservada para la animación de la Fase 2: velocidad temporal de las manchas.
pub const SPOT_SPEED: f32 = 0.15;
// Umbral de ruido a partir del cual comienzan las manchas oscuras.
pub const SPOT_THRESHOLD: f32 = 0.62;
// Suavidad del borde de las manchas.
pub const SPOT_SOFTNESS: f32 = 0.12;
// Exponente del oscurecimiento hacia el borde del disco.
pub const LIMB_EXPONENT: f32 = 0.55;
// Exponente del anillo de corona de la Fase 2.
pub const CORONA_EXPONENT: f32 = 5.0;
// Intensidad del anillo de corona de la Fase 2.
pub const CORONA_INTENSITY: f32 = 1.0;
// Paradas de color reservadas para la paleta de la Fase 2.
pub const PALETTE_WHITE_PINK: [f32; 3] = [1.0, 0.86, 0.96];
pub const PALETTE_MAGENTA: [f32; 3] = [0.90, 0.05, 0.55];
pub const PALETTE_VIOLET: [f32; 3] = [0.32, 0.02, 0.55];
pub const PALETTE_INDIGO: [f32; 3] = [0.015, 0.005, 0.08];

pub fn temperature(fragment: &Fragment, uniforms: &Uniforms) -> f32 {
    let normal = fragment.normal.normalize();
    let view = (uniforms.camera_pos - fragment.world_pos).normalize();
    let mu = normal.dot(view).clamp(0.0, 1.0);
    let limb = mu.powf(LIMB_EXPONENT);

    let surface_position = fragment.object_pos.normalize();
    let granulation = noise::fbm(
        surface_position * GRANULATION_FREQ,
        4,
        2.0,
        0.5,
    ) - 0.5;
    let mut value = limb + granulation * GRANULATION_AMP;

    let spots = noise::fbm(surface_position * SPOT_FREQ, 3, 2.0, 0.5);
    let spot_mask = noise::smoothstep(
        SPOT_THRESHOLD,
        SPOT_THRESHOLD + SPOT_SOFTNESS,
        spots,
    );
    value -= spot_mask * 0.45;

    value.clamp(0.0, 1.0)
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let value = temperature(fragment, uniforms);
    Vector3::new(value, value, value)
}
