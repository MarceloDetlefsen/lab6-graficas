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
// Umbral de ruido a partir del cual comienzan las manchas oscuras aisladas.
pub const SPOT_THRESHOLD: f32 = 0.59;
// Suavidad del borde de las manchas.
pub const SPOT_SOFTNESS: f32 = 0.03;
// Exponente del oscurecimiento hacia el borde del disco.
pub const LIMB_EXPONENT: f32 = 2.5;
// Temperatura mínima del disco en el limbo antes de aplicar granulación y manchas.
pub const LIMB_MIN_TEMPERATURE: f32 = 0.25;
// Temperatura máxima del centro del disco antes de aplicar granulación y manchas.
pub const LIMB_MAX_TEMPERATURE: f32 = 0.95;
// Cuánto oscurece cada mancha después del smoothstep.
pub const SPOT_DARKNESS: f32 = 0.75;
// Exponente del anillo de corona de la Fase 2.
pub const CORONA_EXPONENT: f32 = 5.0;
// Intensidad del anillo de corona de la Fase 2.
pub const CORONA_INTENSITY: f32 = 1.0;
// Paradas de color reservadas para la paleta de la Fase 2.
pub const PALETTE_WHITE_PINK: [f32; 3] = [1.0, 0.86, 0.96];
pub const PALETTE_MAGENTA: [f32; 3] = [0.90, 0.05, 0.55];
pub const PALETTE_VIOLET: [f32; 3] = [0.32, 0.02, 0.55];
pub const PALETTE_INDIGO: [f32; 3] = [0.015, 0.005, 0.08];

fn surface_mu(fragment: &Fragment, uniforms: &Uniforms) -> f32 {
    let normal = fragment.normal.normalize();
    let view = (uniforms.camera_pos - fragment.world_pos).normalize();
    // El z-buffer actual conserva menor z como más cercano, mientras la cámara
    // ortográfica está ubicada en +z. La cara visible queda con la normal
    // invertida respecto de view; el valor absoluto conserva el perfil de disco
    // (centro caliente, limbo frío) sin cambiar los espacios de los atributos.
    normal.dot(view).abs().clamp(0.0, 1.0)
}

pub fn temperature(fragment: &Fragment, uniforms: &Uniforms) -> f32 {
    let mu = surface_mu(fragment, uniforms);
    let limb = LIMB_MIN_TEMPERATURE
        + (LIMB_MAX_TEMPERATURE - LIMB_MIN_TEMPERATURE) * mu.powf(LIMB_EXPONENT);

    let surface_position = fragment.object_pos.normalize();
    let granulation_noise = noise::fbm(
        surface_position * GRANULATION_FREQ,
        4,
        2.0,
        0.5,
    );
    let granulation = granulation_noise * 2.0 - 1.0;
    let mut value = limb + granulation * GRANULATION_AMP;

    let spots = noise::fbm(surface_position * SPOT_FREQ, 3, 2.0, 0.5);
    let spot_mask = noise::smoothstep(
        SPOT_THRESHOLD,
        SPOT_THRESHOLD + SPOT_SOFTNESS,
        spots,
    );
    value -= spot_mask * SPOT_DARKNESS;

    value.clamp(0.0, 1.0)
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let value = temperature(fragment, uniforms);
    Vector3::new(value, value, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use raylib::prelude::Matrix;

    fn uniforms(camera_pos: Vector3) -> Uniforms {
        Uniforms {
            model_matrix: Matrix::identity(),
            time: 0.0,
            light_dir: Vector3::new(0.0, 0.0, -1.0),
            camera_pos,
            debug_noise: false,
            noise_frequency: 4.0,
        }
    }

    fn fragment(world_pos: Vector3, normal: Vector3) -> Fragment {
        Fragment {
            position: raylib::prelude::Vector2::zero(),
            object_pos: normal,
            world_pos,
            normal,
            color: Vector3::zero(),
            depth: 0.0,
        }
    }

    #[test]
    fn visible_disk_mu_runs_from_center_to_limb() {
        let camera = Vector3::new(0.0, 0.0, 1000.0);
        let center = surface_mu(
            &fragment(Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, 0.0, -1.0)),
            &uniforms(camera),
        );
        let limb = surface_mu(
            &fragment(Vector3::new(1.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0)),
            &uniforms(camera),
        );
        println!("star mu: center={center:.3}, limb={limb:.3}");
        assert!(center > 0.99);
        assert!(limb < 0.01);
    }

    #[test]
    fn reports_spot_noise_distribution() {
        let sample_count = 4096;
        let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
        let mut values = Vec::with_capacity(sample_count);

        for i in 0..sample_count {
            let y = 1.0 - (i as f32 + 0.5) * 2.0 / sample_count as f32;
            let radius = (1.0 - y * y).sqrt();
            let angle = golden_angle * i as f32;
            let point = Vector3::new(radius * angle.cos(), y, radius * angle.sin());
            values.push(noise::fbm(point * SPOT_FREQ, 3, 2.0, 0.5));
        }

        values.sort_by(f32::total_cmp);
        let percentile = |p: f32| values[(p * (values.len() - 1) as f32) as usize];
        println!(
            "spot noise: min={:.3}, p05={:.3}, p50={:.3}, p95={:.3}, max={:.3}",
            values[0],
            percentile(0.05),
            percentile(0.50),
            percentile(0.95),
            values[values.len() - 1],
        );
    }
}
