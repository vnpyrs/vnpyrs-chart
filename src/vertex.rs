use std::sync::Mutex;

use glam::{Mat4, Vec3, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 2],
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

pub fn create_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x2,
        }],
    }
}

pub static VP_MATRIX: Mutex<Mat4> = Mutex::new(Mat4::IDENTITY);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj_candle(
        &mut self,
        left_ix: i64,
        right_ix: i64,
        min_price: f64,
        max_price: f64,
    ) {
        let view = glam::Mat4::look_at_rh(
            Vec3 {
                x: (left_ix + right_ix) as f32 / 2.0,
                y: (min_price + max_price) as f32 / 2.0,
                z: 0.0,
            },
            Vec3 {
                x: (left_ix + right_ix) as f32 / 2.0,
                y: (min_price + max_price) as f32 / 2.0,
                z: -1.0,
            },
            Vec3::Y,
        );
        let proj = glam::Mat4::orthographic_rh(
            -((right_ix - left_ix) as f32 / 2.0 + 0.5),
            (right_ix - left_ix) as f32 / 2.0 + 0.5,
            -((max_price - min_price) as f32 / 2.0),
            (max_price - min_price) as f32 / 2.0,
            0.0,
            1.0,
        );
        let vp = proj * view;
        self.view_proj = vp.to_cols_array_2d();
        *VP_MATRIX.lock().unwrap() = vp;
    }

    pub fn update_view_proj_volume(&mut self, left_ix: i64, right_ix: i64, max_volume: f64) {
        let view = glam::Mat4::look_at_rh(
            Vec3 {
                x: (left_ix + right_ix) as f32 / 2.0,
                y: max_volume as f32 / 2.0,
                z: 0.0,
            },
            Vec3 {
                x: (left_ix + right_ix) as f32 / 2.0,
                y: max_volume as f32 / 2.0,
                z: -1.0,
            },
            Vec3::Y,
        );
        let proj = glam::Mat4::orthographic_rh(
            -((right_ix - left_ix) as f32 / 2.0 + 0.5),
            (right_ix - left_ix) as f32 / 2.0 + 0.5,
            -(max_volume as f32 / 2.0),
            max_volume as f32 / 2.0,
            0.0,
            1.0,
        );
        self.view_proj = (proj * view).to_cols_array_2d();
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenUniform {
    pub width: f32,
    pub height: f32,
}

impl ScreenUniform {
    pub fn new() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }
}

#[derive(Default)]
pub struct RectangleFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub vertex: Option<[Vertex; 5]>,
}

impl RectangleFrame {
    pub fn make_vertex(&mut self, screen_width: u32, screen_height: u32) {
        let s_width = screen_width as f32;
        let s_height = screen_height as f32;
        let x = self.x;
        let y = s_height - self.y;
        self.vertex = Some([
            Vertex {
                position: [x / s_width * 2.0 - 1.0, y / s_height * 2.0 - 1.0],
            },
            Vertex {
                position: [
                    (x + self.width) / s_width * 2.0 - 1.0,
                    y / s_height * 2.0 - 1.0,
                ],
            },
            Vertex {
                position: [
                    (x + self.width) / s_width * 2.0 - 1.0,
                    (y - self.height) / s_height * 2.0 - 1.0,
                ],
            },
            Vertex {
                position: [
                    x / s_width * 2.0 - 1.0,
                    (y - self.height) / s_height * 2.0 - 1.0,
                ],
            },
            Vertex {
                position: [x / s_width * 2.0 - 1.0, y / s_height * 2.0 - 1.0],
            },
        ]);
    }
}

#[derive(Default)]
pub struct RectangleFilled {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub vertex: Option<[Vertex; 4]>,
}

impl RectangleFilled {
    pub fn make_vertex(&mut self, screen_width: u32, screen_height: u32) {
        let s_width = screen_width as f32;
        let s_height = screen_height as f32;
        let x = self.x;
        let y = s_height - self.y;
        self.vertex = Some([
            Vertex {
                position: [x / s_width * 2.0 - 1.0, y / s_height * 2.0 - 1.0],
            },
            Vertex {
                position: [
                    x / s_width * 2.0 - 1.0,
                    (y - self.height) / s_height * 2.0 - 1.0,
                ],
            },
            Vertex {
                position: [
                    (x + self.width) / s_width * 2.0 - 1.0,
                    y / s_height * 2.0 - 1.0,
                ],
            },
            Vertex {
                position: [
                    (x + self.width) / s_width * 2.0 - 1.0,
                    (y - self.height) / s_height * 2.0 - 1.0,
                ],
            },
        ]);
    }
}

#[derive(Default)]
pub struct Line {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub vertex: Option<[Vertex; 2]>,
}

impl Line {
    pub fn make_vertex(&mut self, screen_width: u32, screen_height: u32) {
        let s_width = screen_width as f32;
        let s_height = screen_height as f32;
        let x1 = self.x1;
        let y1 = s_height - self.y1;
        let x2 = self.x2;
        let y2 = s_height - self.y2;
        self.vertex = Some([
            Vertex {
                position: [x1 / s_width * 2.0 - 1.0, y1 / s_height * 2.0 - 1.0],
            },
            Vertex {
                position: [x2 / s_width * 2.0 - 1.0, y2 / s_height * 2.0 - 1.0],
            },
        ]);
    }
}

pub struct CandleVertex {
    pub up: Vec<Vertex>,
    pub down: Vec<Vertex>,
    pub down_hl: Vec<Vertex>,
    pub stay: Vec<Vertex>,
}

pub struct VolumeVertex {
    pub up: Vec<Vertex>,
    pub down: Vec<Vertex>,
    pub stay: Vec<Vertex>,
}

pub struct TradePairVertex {
    pub profit: Vec<Vertex>,
    pub loss: Vec<Vertex>,
    pub buy: Vec<Vertex>,
    pub sell: Vec<Vertex>,
    pub short: Vec<Vertex>,
    pub cover: Vec<Vertex>,
    pub buy_text: Vec<(Vec4, String)>,
    pub sell_text: Vec<(Vec4, String)>,
    pub short_text: Vec<(Vec4, String)>,
    pub cover_text: Vec<(Vec4, String)>,
}
