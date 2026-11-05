mod framebuffer;
mod fragment;
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
use shaders::{fragment_shader, vertex_shader};

pub struct Uniforms {
    pub model_matrix: Matrix,
    pub time: f32,
    pub light_dir: Vector3,
    pub camera_pos: Vector3,
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[vertex::Vertex],
    mode: TriangleMode,
) {
    let transformed_vertices: Vec<vertex::Vertex> = vertex_array
        .iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    let triangles = primitive_assembly(&transformed_vertices);
    let fragments = rasterize(&triangles, mode);

    for fragment in fragments {
        let fragment_color = fragment_shader(&fragment);
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

    let mut framebuffer = Framebuffer::new(
        VIEWPORT_WIDTH as u32,
        VIEWPORT_HEIGHT as u32,
        Color::new(15, 18, 28, 255),
    );
    let (mut window, thread) = raylib::init()
        .size(VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
        .title("Lab 6 - Esfera procedural")
        .build();
    window.set_target_fps(60);

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

    let mut mode = TriangleMode::FilledBarycentric;
    let mut yaw = 0.0_f32;
    let mut pitch = 0.0_f32;
    let mut zoom = 1.0_f32;
    let mut time = 0.0_f32;

    while !window.window_should_close() {
        time += window.get_frame_time();
        if window.is_key_pressed(KeyboardKey::KEY_W) {
            mode = TriangleMode::Wireframe;
        }
        if window.is_key_pressed(KeyboardKey::KEY_B) {
            mode = TriangleMode::FilledBarycentric;
        }
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
        let uniforms = Uniforms {
            model_matrix,
            time,
            light_dir: Vector3::new(-1.0, -1.0, -1.0).normalize(),
            camera_pos: Vector3::zero(),
        };

        framebuffer.clear();
        render(&mut framebuffer, &uniforms, &vertex_array, mode);
        framebuffer.draw_in_window(&mut window, &thread);
    }
}
