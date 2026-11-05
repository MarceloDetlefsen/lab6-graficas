#![allow(dead_code)]

use raylib::prelude::Vector3;

use crate::fragment::Fragment;
use crate::noise;
use crate::Uniforms;

// Nivel de depuración: 0=altura, 1=bioma verde, 2=grietas.
pub const DEBUG_VIEW: i32 = 0;
// Estiramiento anisotrópico de las vetas en el eje Y.
pub const VEIN_STRETCH: f32 = 0.42;
// Amplitud del desplazamiento de domain warping.
pub const WARP_AMP: f32 = 0.30;
// Frecuencia del ruido que deforma el dominio.
pub const WARP_FREQ: f32 = 1.25;
// Frecuencia de la máscara de vegetación de baja frecuencia.
pub const BIOME_FREQ: f32 = 1.8;
// Umbral alto para limitar Brinstar a una fracción de la superficie.
pub const BIOME_THRESHOLD: f32 = 0.62;
// Suavidad del umbral de la máscara de vegetación.
pub const BIOME_SOFTNESS: f32 = 0.12;
// Umbral de las vetas más altas que se mostrarán como grietas.
pub const CRACK_THRESHOLD: f32 = 0.72;
// Suavidad del umbral de grietas.
pub const CRACK_SOFTNESS: f32 = 0.08;
// Octavas máximas del campo de altura y de las máscaras.
pub const TERRAIN_OCTAVES: usize = 4;

pub fn height_field(object_pos: Vector3) -> f32 {
    let p = object_pos.normalize();
    let warp = noise::fbm(p * WARP_FREQ, 3, 2.0, 0.5) * 2.0 - 1.0;
    let warped = Vector3::new(
        p.x + warp * WARP_AMP,
        p.y * VEIN_STRETCH + warp * WARP_AMP * 0.5,
        p.z + warp * WARP_AMP,
    );
    noise::ridged(warped, TERRAIN_OCTAVES, 2.0, 0.5)
}

pub fn biome_mask(object_pos: Vector3) -> f32 {
    let value = noise::fbm(object_pos.normalize() * BIOME_FREQ, 3, 2.0, 0.5);
    noise::smoothstep(BIOME_THRESHOLD, BIOME_THRESHOLD + BIOME_SOFTNESS, value)
}

pub fn crack_mask(height: f32) -> f32 {
    noise::smoothstep(CRACK_THRESHOLD, CRACK_THRESHOLD + CRACK_SOFTNESS, height)
}

pub fn shade(fragment: &Fragment, _uniforms: &Uniforms) -> Vector3 {
    let height = height_field(fragment.object_pos);
    let value = match DEBUG_VIEW {
        1 => biome_mask(fragment.object_pos),
        2 => crack_mask(height),
        _ => height,
    };
    Vector3::new(value, value, value)
}
