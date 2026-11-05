#![allow(dead_code)]

use raylib::prelude::*;
use raylib::texture::Image;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    color_buffer: Image,
    background_color: Color,
    pub current_color: Color,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, background_color: Color) -> Self {
        let color_buffer = Image::gen_image_color(width as i32, height as i32, background_color);

        Self {
            width,
            height,
            color_buffer,
            background_color,
            current_color: Color::WHITE,
        }
    }

    pub fn clear(&mut self) {
        self.color_buffer = Image::gen_image_color(
            self.width as i32,
            self.height as i32,
            self.background_color,
        );
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            self.color_buffer.draw_pixel(x, y, color);
        }
    }

    pub fn pixel_color(&self, x: i32, y: i32) -> Color {
        self.color_buffer.get_color(x, y)
    }

    pub fn point(&mut self, point: Vector2) {
        let x = point.x.round() as i32;
        let y = point.y.round() as i32;
        self.set_pixel(x, y, self.current_color);
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn render_to_file(&self, filename: &str) {
        if Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("bmp"))
        {
            self.render_to_bmp_file(filename);
        } else {
            self.color_buffer.export_image(filename);
        }
    }

    pub fn draw_in_window(&self, window: &mut RaylibHandle, thread: &RaylibThread) {
        if let Ok(texture) = window.load_texture_from_image(thread, &self.color_buffer) {
            let mut drawing = window.begin_drawing(thread);
            drawing.clear_background(Color::BLACK);
            drawing.draw_texture(&texture, 0, 0, Color::WHITE);
        }
    }

    fn render_to_bmp_file(&self, filename: &str) {
        let width = self.width as usize;
        let height = self.height as usize;
        let row_stride = width * 3;
        let row_padding = (4 - (row_stride % 4)) % 4;
        let image_size = (row_stride + row_padding) * height;
        let file_size = 14 + 40 + image_size;

        let file = File::create(filename).expect("failed to create BMP file");
        let mut writer = BufWriter::new(file);

        // BMP file header.
        writer.write_all(&[0x42, 0x4D]).expect("failed to write BMP signature");
        writer
            .write_all(&(file_size as u32).to_le_bytes())
            .expect("failed to write BMP file size");
        writer
            .write_all(&[0u8; 4])
            .expect("failed to write BMP reserved fields");
        writer
            .write_all(&(54u32).to_le_bytes())
            .expect("failed to write BMP pixel offset");

        // DIB header (BITMAPINFOHEADER).
        writer
            .write_all(&(40u32).to_le_bytes())
            .expect("failed to write BMP DIB header size");
        writer
            .write_all(&(self.width as i32).to_le_bytes())
            .expect("failed to write BMP width");
        writer
            .write_all(&(self.height as i32).to_le_bytes())
            .expect("failed to write BMP height");
        writer
            .write_all(&(1u16).to_le_bytes())
            .expect("failed to write BMP planes");
        writer
            .write_all(&(24u16).to_le_bytes())
            .expect("failed to write BMP bpp");
        writer
            .write_all(&(0u32).to_le_bytes())
            .expect("failed to write BMP compression");
        writer
            .write_all(&(image_size as u32).to_le_bytes())
            .expect("failed to write BMP image size");
        writer
            .write_all(&(2835u32).to_le_bytes())
            .expect("failed to write BMP x ppm");
        writer
            .write_all(&(2835u32).to_le_bytes())
            .expect("failed to write BMP y ppm");
        writer
            .write_all(&(0u32).to_le_bytes())
            .expect("failed to write BMP palette colors");
        writer
            .write_all(&(0u32).to_le_bytes())
            .expect("failed to write BMP important colors");

        let padding = [0u8; 3];
        for y in (0..height).rev() {
            for x in 0..width {
                let color = self.color_buffer.get_color(x as i32, y as i32);
                writer
                    .write_all(&[color.b, color.g, color.r])
                    .expect("failed to write BMP pixel data");
            }
            writer
                .write_all(&padding[..row_padding])
                .expect("failed to write BMP row padding");
        }

        writer.flush().expect("failed to flush BMP file");
    }
}

pub fn point(framebuffer: &mut Framebuffer, point: Vector2) {
    framebuffer.point(point);
}
