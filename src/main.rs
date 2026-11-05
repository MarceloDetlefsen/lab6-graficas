mod framebuffer;
mod fragment;
mod noise;
mod obj;
mod primitive;
mod rasterizer;
mod scene;
mod shaders;
mod triangle;
mod vertex;

use framebuffer::Framebuffer;
use obj::load_obj;
use primitive::primitive_assembly;
use rasterizer::{rasterize, TriangleMode, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};
use raylib::prelude::*;
use scene::build_model_matrix;
use shaders::{fragment_shader, shader_name, vertex_shader, Shader};
use std::fs;
use std::path::PathBuf;

pub struct Uniforms {
    pub model_matrix: Matrix,
    pub time: f32,
    pub light_dir: Vector3,
    pub camera_pos: Vector3,
    pub debug_noise: bool,
    pub noise_frequency: f32,
    pub rocky_debug_view: i32,
}

struct CaptureOptions {
    shader: Shader,
    view: i32,
    light_dir: Vector3,
    time: f32,
    output: PathBuf,
}

fn parse_vector3(value: &str, argument: &str) -> Vector3 {
    let components: Vec<f32> = value
        .split(',')
        .map(|component| {
            component
                .parse::<f32>()
                .unwrap_or_else(|_| panic!("Valor inválido para {argument}: {value}"))
        })
        .collect();
    if components.len() != 3 {
        panic!("{argument} debe tener formato x,y,z: {value}");
    }
    Vector3::new(components[0], components[1], components[2]).normalize()
}

fn parse_shader(value: &str) -> Shader {
    match value {
        "1" => Shader::Star,
        "2" => Shader::Rocky,
        "3" => Shader::GasGiant,
        _ => panic!("Shader inválido: {value}; usá 1, 2 o 3"),
    }
}

fn parse_view(value: &str) -> i32 {
    match value {
        "height" | "altura" => 0,
        "biome" | "bioma" => 1,
        "cracks" | "grietas" => 2,
        "color" => 3,
        _ => panic!("Vista inválida: {value}; usá height, biome, cracks o color"),
    }
}

fn parse_capture_options() -> Option<CaptureOptions> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.is_empty() {
        return None;
    }

    let mut shader = None;
    let mut view = None;
    let mut light_dir = None;
    let mut time = None;
    let mut output = None;
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        let value = arguments.get(index + 1).unwrap_or_else(|| {
            panic!("Falta valor para {argument}; argumentos esperados: --shader, --view, --light, --time, --out")
        });
        match argument.as_str() {
            "--shader" => shader = Some(parse_shader(value)),
            "--view" => view = Some(parse_view(value)),
            "--light" => light_dir = Some(parse_vector3(value, "--light")),
            "--time" => {
                time = Some(
                    value
                        .parse::<f32>()
                        .unwrap_or_else(|_| panic!("Tiempo inválido: {value}")),
                )
            }
            "--out" => output = Some(PathBuf::from(value)),
            _ => panic!("Argumento desconocido: {argument}"),
        }
        index += 2;
    }

    Some(CaptureOptions {
        shader: shader.expect("Falta --shader"),
        view: view.expect("Falta --view"),
        light_dir: light_dir.expect("Falta --light"),
        time: time.expect("Falta --time"),
        output: output.expect("Falta --out"),
    })
}

fn next_capture_path() -> PathBuf {
    let captures_dir = PathBuf::from("captures");
    fs::create_dir_all(&captures_dir).expect("No se pudo crear el directorio captures");

    let mut index = 1;
    loop {
        let path = captures_dir.join(format!("shot_{index:03}.png"));
        if !path.exists() {
            return path;
        }
        index += 1;
    }
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[vertex::Vertex],
    mode: TriangleMode,
    shader: Shader,
) {
    let transformed_vertices: Vec<vertex::Vertex> = vertex_array
        .iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    let triangles = primitive_assembly(&transformed_vertices);
    let fragments = rasterize(&triangles, mode);

    for fragment in fragments {
        let fragment_color = fragment_shader(&fragment, shader, uniforms);
        let color = Color::new(
            (fragment_color.x.clamp(0.0, 1.0) * 255.0) as u8,
            (fragment_color.y.clamp(0.0, 1.0) * 255.0) as u8,
            (fragment_color.z.clamp(0.0, 1.0) * 255.0) as u8,
            255,
        );
        framebuffer.set_pixel_depth(
            fragment.position.x as i32,
            fragment.position.y as i32,
            fragment.depth,
            color,
        );
    }
}

fn main() {
    let obj_path = "assets/sphere.obj";
    let obj = load_obj(obj_path).expect("No se pudo cargar assets/sphere.obj");
    println!("Modelo cargado: '{obj_path}'");
    println!("  Vértices:   {}", obj.vertices.len());
    println!("  Normales fuente (vn): {}", obj.source_normals.len());
    println!("  Triángulos: {}", obj.indices.len() / 3);
    let normals_are_unit = obj
        .vertex_normals
        .iter()
        .all(|normal| (normal.length() - 1.0).abs() <= 1e-4);
    println!("  Normales suaves unitarias (~1): {normals_are_unit}");

    let vertex_array: Vec<vertex::Vertex> = obj
        .indices
        .iter()
        .map(|&index| {
            vertex::Vertex::new(
                obj.vertices[index],
                obj.vertex_normals[index],
                Vector3::new(0.25, 0.55, 0.95),
            )
        })
        .collect();

    let (min, max) = obj.bounds();
    let model_width = max.x - min.x;
    let model_height = max.y - min.y;
    let scale = (VIEWPORT_WIDTH as f32 * 0.8 / model_width)
        .min(VIEWPORT_HEIGHT as f32 * 0.8 / model_height);

    let mut framebuffer = Framebuffer::new(
        VIEWPORT_WIDTH as u32,
        VIEWPORT_HEIGHT as u32,
        Color::new(15, 18, 28, 255),
    );

    if let Some(options) = parse_capture_options() {
        let model_matrix = build_model_matrix(&obj, scale, 0.0, 0.0, 1.0);
        let uniforms = Uniforms {
            model_matrix,
            time: options.time,
            light_dir: options.light_dir,
            camera_pos: Vector3::new(0.0, 0.0, scale * 2.0),
            debug_noise: false,
            noise_frequency: 4.0,
            rocky_debug_view: options.view,
        };
        framebuffer.clear();
        render(
            &mut framebuffer,
            &uniforms,
            &vertex_array,
            TriangleMode::FilledBarycentric,
            options.shader,
        );
        if let Some(parent) = options.output.parent() {
            fs::create_dir_all(parent).expect("No se pudo crear el directorio de captura");
        }
        framebuffer.render_to_file(
            options
                .output
                .to_str()
                .expect("Ruta de captura inválida"),
        );
        println!("Captura automática guardada en {}", options.output.display());
        return;
    }

    let (mut window, thread) = raylib::init()
        .size(VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
        .title("Lab 6 - Esfera procedural")
        .build();
    window.set_target_fps(60);

    let mut mode = TriangleMode::FilledBarycentric;
    let mut yaw = 0.0_f32;
    let mut pitch = 0.0_f32;
    let mut zoom = 1.0_f32;
    let mut time = 0.0_f32;
    let mut shader = Shader::Rocky;
    let mut debug_noise = false;
    let mut noise_frequency = 4.0_f32;
    let mut rocky_debug_view = 3_i32;
    let mut light_angle = 0.0_f32;

    while !window.window_should_close() {
        time += window.get_frame_time();
        if window.is_key_pressed(KeyboardKey::KEY_W) {
            mode = TriangleMode::Wireframe;
        }
        if window.is_key_pressed(KeyboardKey::KEY_B) {
            mode = TriangleMode::FilledBarycentric;
        }
        if window.is_key_pressed(KeyboardKey::KEY_ONE) {
            shader = Shader::Star;
        }
        if window.is_key_pressed(KeyboardKey::KEY_TWO) {
            shader = Shader::Rocky;
        }
        if window.is_key_pressed(KeyboardKey::KEY_THREE) {
            shader = Shader::GasGiant;
        }
        if window.is_key_pressed(KeyboardKey::KEY_D) {
            debug_noise = !debug_noise;
        }
        if window.is_key_pressed(KeyboardKey::KEY_V) {
            rocky_debug_view = (rocky_debug_view + 1) % 4;
        }
        if window.is_key_down(KeyboardKey::KEY_J) {
            light_angle -= 0.03;
        }
        if window.is_key_down(KeyboardKey::KEY_L) {
            light_angle += 0.03;
        }
        if window.is_key_pressed(KeyboardKey::KEY_LEFT_BRACKET) {
            noise_frequency = (noise_frequency - 0.25).max(0.25);
        }
        if window.is_key_pressed(KeyboardKey::KEY_RIGHT_BRACKET) {
            noise_frequency = (noise_frequency + 0.25).min(20.0);
        }
        let capture_requested = window.is_key_pressed(KeyboardKey::KEY_P);
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            yaw -= 0.03;
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            yaw += 0.03;
        }
        if window.is_key_down(KeyboardKey::KEY_UP) {
            pitch -= 0.03;
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            pitch += 0.03;
        }
        zoom = (zoom + window.get_mouse_wheel_move() * 0.1).clamp(0.2, 4.0);

        let model_matrix = build_model_matrix(&obj, scale, yaw, pitch, zoom);
        // En la convención del renderer, -z es el frente visible de la esfera.
        let base_light = Vector3::new(-0.65, -0.80, -1.0).normalize();
        let light_dir = Vector3::new(
            base_light.x * light_angle.cos() - base_light.z * light_angle.sin(),
            base_light.y,
            base_light.x * light_angle.sin() + base_light.z * light_angle.cos(),
        )
        .normalize();
        let uniforms = Uniforms {
            model_matrix,
            time,
            light_dir,
            camera_pos: Vector3::new(0.0, 0.0, scale * zoom * 2.0),
            debug_noise,
            noise_frequency,
            rocky_debug_view,
        };

        framebuffer.clear();
        render(&mut framebuffer, &uniforms, &vertex_array, mode, shader);
        if capture_requested {
            let capture_path = next_capture_path();
            framebuffer.render_to_file(capture_path.to_str().expect("Ruta de captura inválida"));
            println!("Captura guardada en {}", capture_path.display());
        }
        let debug_label = if debug_noise { " | DEBUG FBM" } else { "" };
        let label = format!(
            "{}{} | Rocky: {} | Luz: ({:.2},{:.2},{:.2}) | Noise freq: {:.2}",
            shader_name(shader),
            debug_label,
            ["altura", "bioma", "grietas", "color"][rocky_debug_view as usize],
            light_dir.x,
            light_dir.y,
            light_dir.z,
            noise_frequency,
        );
        framebuffer.draw_in_window(&mut window, &thread, &label);
    }
}
