use crate::manager::{Manager, CANDLE_VERTEX, HISTORY, TRADE_PAIRS_VERTEX, VOLUME_VERTEX};
use crate::vertex::{
    create_vertex_buffer_layout, CameraUniform, Line, RectangleFilled, RectangleFrame,
    ScreenUniform, VP_MATRIX,
};
use std::borrow::Cow;
use std::sync::Arc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::MemoryHints::Performance;
use wgpu::{BindGroup, BindGroupLayout, Buffer, ShaderSource};
use wgpu_text::glyph_brush::{ab_glyph::FontRef, Section, Text};
use wgpu_text::{BrushBuilder, TextBrush};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

const MARGIN: f32 = 5.0;
const AXIS_X_HEIGHT: f32 = 32.0;
const AXIS_X_LABEL_BIAS: f32 = 30.0;
const AXIS_Y_WIDTH: f32 = 80.0;
const AXIS_Y_LABEL_BIAS: f32 = 8.0;
const HINT_HEIGHT: f32 = 80.0;
const INFO_SIZE: (f32, f32) = (80.0, 320.0);

pub struct RectangleFramePack {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    shape: RectangleFrame,
}

impl RectangleFramePack {
    pub fn new(
        device: &wgpu::Device,
        swap_chain_format: wgpu::TextureFormat,
        fs_main: &str,
        buffer_size: usize,
    ) -> Self {
        let render_pipeline = create_pipeline(
            device,
            swap_chain_format,
            fs_main,
            wgpu::PrimitiveTopology::LineStrip,
        );
        let bytes: &[u8] = &vec![0; buffer_size];
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytes,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let shape = RectangleFrame::default();

        RectangleFramePack {
            render_pipeline,
            vertex_buffer,
            shape,
        }
    }
}

pub struct RectangleFilledPack {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    shape: RectangleFilled,
}

impl RectangleFilledPack {
    pub fn new(
        device: &wgpu::Device,
        swap_chain_format: wgpu::TextureFormat,
        fs_main: &str,
        buffer_size: usize,
    ) -> Self {
        let render_pipeline = create_pipeline(
            device,
            swap_chain_format,
            fs_main,
            wgpu::PrimitiveTopology::TriangleStrip,
        );
        let bytes: &[u8] = &vec![0; buffer_size];
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytes,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let shape = RectangleFilled::default();

        RectangleFilledPack {
            render_pipeline,
            vertex_buffer,
            shape,
        }
    }
}

pub struct LinePack {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    shape: Line,
}

impl LinePack {
    pub fn new(
        device: &wgpu::Device,
        swap_chain_format: wgpu::TextureFormat,
        fs_main: &str,
        buffer_size: usize,
    ) -> Self {
        let render_pipeline = create_pipeline(
            device,
            swap_chain_format,
            fs_main,
            wgpu::PrimitiveTopology::LineList,
        );
        let bytes: &[u8] = &vec![0; buffer_size];
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytes,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let shape = Line::default();

        LinePack {
            render_pipeline,
            vertex_buffer,
            shape,
        }
    }
}

pub struct CandlePack {
    up_render_pipeline: wgpu::RenderPipeline,
    up_vertex_buffer: wgpu::Buffer,
    down_render_pipeline: wgpu::RenderPipeline,
    down_vertex_buffer: wgpu::Buffer,
    down_hl_render_pipeline: wgpu::RenderPipeline,
    down_hl_vertex_buffer: wgpu::Buffer,
    stay_render_pipeline: wgpu::RenderPipeline,
    stay_vertex_buffer: wgpu::Buffer,
}

impl CandlePack {
    pub fn new(
        device: &wgpu::Device,
        swap_chain_format: wgpu::TextureFormat,
        camera_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let up_render_pipeline = create_candle_pipeline(
            device,
            swap_chain_format,
            "fs_main_up",
            wgpu::PrimitiveTopology::LineList,
            camera_bind_group_layout,
        );
        let up_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&CANDLE_VERTEX.up),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let down_render_pipeline = create_candle_pipeline(
            device,
            swap_chain_format,
            "fs_main_down",
            wgpu::PrimitiveTopology::TriangleList,
            camera_bind_group_layout,
        );
        let down_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&CANDLE_VERTEX.down),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let down_hl_render_pipeline = create_candle_pipeline(
            device,
            swap_chain_format,
            "fs_main_down",
            wgpu::PrimitiveTopology::LineList,
            camera_bind_group_layout,
        );
        let down_hl_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&CANDLE_VERTEX.down_hl),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let stay_render_pipeline = create_candle_pipeline(
            device,
            swap_chain_format,
            "fs_main_stay",
            wgpu::PrimitiveTopology::LineList,
            camera_bind_group_layout,
        );
        let stay_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&CANDLE_VERTEX.stay),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        CandlePack {
            up_render_pipeline,
            up_vertex_buffer,
            down_render_pipeline,
            down_vertex_buffer,
            down_hl_render_pipeline,
            down_hl_vertex_buffer,
            stay_render_pipeline,
            stay_vertex_buffer,
        }
    }
}

pub struct VolumePack {
    up_render_pipeline: wgpu::RenderPipeline,
    up_vertex_buffer: wgpu::Buffer,
    down_render_pipeline: wgpu::RenderPipeline,
    down_vertex_buffer: wgpu::Buffer,
    stay_render_pipeline: wgpu::RenderPipeline,
    stay_vertex_buffer: wgpu::Buffer,
}

impl VolumePack {
    pub fn new(
        device: &wgpu::Device,
        swap_chain_format: wgpu::TextureFormat,
        camera_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let up_render_pipeline = create_volume_pipeline(
            device,
            swap_chain_format,
            "fs_main_up",
            wgpu::PrimitiveTopology::LineList,
            camera_bind_group_layout,
        );
        let up_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&VOLUME_VERTEX.up),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let down_render_pipeline = create_volume_pipeline(
            device,
            swap_chain_format,
            "fs_main_down",
            wgpu::PrimitiveTopology::TriangleList,
            camera_bind_group_layout,
        );
        let down_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&VOLUME_VERTEX.down),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let stay_render_pipeline = create_volume_pipeline(
            device,
            swap_chain_format,
            "fs_main_stay",
            wgpu::PrimitiveTopology::LineList,
            camera_bind_group_layout,
        );
        let stay_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&VOLUME_VERTEX.stay),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        VolumePack {
            up_render_pipeline,
            up_vertex_buffer,
            down_render_pipeline,
            down_vertex_buffer,
            stay_render_pipeline,
            stay_vertex_buffer,
        }
    }
}

pub struct TradePack {
    profit_render_pipeline: wgpu::RenderPipeline,
    profit_vertex_buffer: wgpu::Buffer,
    loss_render_pipeline: wgpu::RenderPipeline,
    loss_vertex_buffer: wgpu::Buffer,
    buy_render_pipeline: wgpu::RenderPipeline,
    buy_vertex_buffer: wgpu::Buffer,
    sell_render_pipeline: wgpu::RenderPipeline,
    sell_vertex_buffer: wgpu::Buffer,
    short_render_pipeline: wgpu::RenderPipeline,
    short_vertex_buffer: wgpu::Buffer,
    cover_render_pipeline: wgpu::RenderPipeline,
    cover_vertex_buffer: wgpu::Buffer,
}
impl TradePack {
    pub fn new(
        device: &wgpu::Device,
        swap_chain_format: wgpu::TextureFormat,
        camera_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let profit_render_pipeline = create_candle_pipeline(
            device,
            swap_chain_format,
            "fs_main_profit",
            wgpu::PrimitiveTopology::LineList,
            camera_bind_group_layout,
        );
        let profit_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&TRADE_PAIRS_VERTEX.profit),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let loss_render_pipeline = create_candle_pipeline(
            device,
            swap_chain_format,
            "fs_main_loss",
            wgpu::PrimitiveTopology::LineList,
            camera_bind_group_layout,
        );
        let loss_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&TRADE_PAIRS_VERTEX.loss),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let buy_render_pipeline = create_triangle_pipeline(
            device,
            swap_chain_format,
            "vs_main_buy_cover",
            "fs_main_buy_sell",
            camera_bind_group_layout,
        );
        let buy_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&TRADE_PAIRS_VERTEX.buy),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let sell_render_pipeline = create_triangle_pipeline(
            device,
            swap_chain_format,
            "vs_main_sell_short",
            "fs_main_buy_sell",
            camera_bind_group_layout,
        );
        let sell_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&TRADE_PAIRS_VERTEX.sell),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let short_render_pipeline = create_triangle_pipeline(
            device,
            swap_chain_format,
            "vs_main_sell_short",
            "fs_main_short_cover",
            camera_bind_group_layout,
        );
        let short_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&TRADE_PAIRS_VERTEX.short),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let cover_render_pipeline = create_triangle_pipeline(
            device,
            swap_chain_format,
            "vs_main_buy_cover",
            "fs_main_short_cover",
            camera_bind_group_layout,
        );
        let cover_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&TRADE_PAIRS_VERTEX.cover),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        TradePack {
            profit_render_pipeline,
            profit_vertex_buffer,
            loss_render_pipeline,
            loss_vertex_buffer,
            buy_render_pipeline,
            buy_vertex_buffer,
            sell_render_pipeline,
            sell_vertex_buffer,
            short_render_pipeline,
            short_vertex_buffer,
            cover_render_pipeline,
            cover_vertex_buffer,
        }
    }
}

pub struct WgpuCtx<'window, 'font> {
    surface: wgpu::Surface<'window>,
    surface_config: wgpu::SurfaceConfiguration,
    // adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    brush: Option<TextBrush<FontRef<'font>>>,
    brush_top: Option<TextBrush<FontRef<'font>>>,
    brush_candle: Option<TextBrush<FontRef<'font>>>, //蜡烛图上的文字画板

    camera_uniform: CameraUniform,
    camera_buffer_candle: Buffer,
    camera_buffer_volume: Buffer,
    screen_uniform: ScreenUniform,
    screen_buffer: Buffer,
    camera_bind_group: BindGroup,

    chart_frame: RectangleFramePack,
    chart_k: RectangleFramePack,
    chart_volume: RectangleFramePack,
    candle_bar: CandlePack,
    trade: TradePack,
    volume_bar: VolumePack,
    cursor_horizontal: LinePack,
    cursor_horizontal_label: RectangleFilledPack,
    cursor_vertical: LinePack,
    cursor_vertical_label: RectangleFilledPack,
    info_box: RectangleFilledPack,

    cursor_show: bool,
    cursor_dock_left: bool,
    manager: Manager,
}

impl<'window, 'font> WgpuCtx<'window, 'font> {
    pub async fn new_async(window: Arc<Window>) -> WgpuCtx<'window, 'font> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");
        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: Performance,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        // 获取窗口内部物理像素尺寸（没有标题栏）
        let size = window.inner_size();
        // 至少（w = 1, h = 1），否则Wgpu会panic
        let width = size.width.max(1);
        let height = size.height.max(1);
        // 获取一个默认配置
        let surface_config = surface.get_default_config(&adapter, width, height).unwrap();
        // 完成首次配置
        surface.configure(&device, &surface_config);

        let font = include_bytes!("msyhbd.ttf");
        let brush = Some(BrushBuilder::using_font_bytes(font).unwrap().build(
            &device,
            width,
            height,
            surface_config.format,
        ));
        let brush_top = Some(BrushBuilder::using_font_bytes(font).unwrap().build(
            &device,
            width,
            height,
            surface_config.format,
        ));
        let brush_candle = Some(BrushBuilder::using_font_bytes(font).unwrap().build(
            &device,
            width,
            height,
            surface_config.format,
        ));

        let camera_uniform = CameraUniform::new();
        let camera_buffer_candle = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_buffer_volume = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let screen_uniform = ScreenUniform::new();
        let screen_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen Buffer"),
            contents: bytemuck::cast_slice(&[screen_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer_candle.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: camera_buffer_volume.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: screen_buffer.as_entire_binding(),
                },
            ],
            label: Some("camera_bind_group"),
        });

        let chart_frame =
            RectangleFramePack::new(&device, surface_config.format, "fs_main_gray1", 40);

        let chart_k = RectangleFramePack::new(&device, surface_config.format, "fs_main_gray2", 40);

        let chart_volume =
            RectangleFramePack::new(&device, surface_config.format, "fs_main_gray2", 40);

        let candle_bar = CandlePack::new(&device, surface_config.format, &camera_bind_group_layout);
        let trade = TradePack::new(&device, surface_config.format, &camera_bind_group_layout);
        let volume_bar = VolumePack::new(&device, surface_config.format, &camera_bind_group_layout);

        let cursor_horizontal = LinePack::new(&device, surface_config.format, "fs_main_gray2", 16);
        let cursor_horizontal_label =
            RectangleFilledPack::new(&device, surface_config.format, "fs_main_label_bg", 32);
        let cursor_vertical = LinePack::new(&device, surface_config.format, "fs_main_gray2", 16);
        let cursor_vertical_label =
            RectangleFilledPack::new(&device, surface_config.format, "fs_main_label_bg", 32);

        let info_box =
            RectangleFilledPack::new(&device, surface_config.format, "fs_main_label_bg", 32);

        WgpuCtx {
            surface,
            surface_config,
            // adapter,
            device,
            queue,
            brush,
            brush_top,
            brush_candle,
            camera_uniform,
            camera_buffer_candle,
            camera_buffer_volume,
            screen_uniform,
            screen_buffer,
            camera_bind_group,
            chart_frame,
            chart_k,
            chart_volume,
            candle_bar,
            trade,
            volume_bar,
            cursor_horizontal,
            cursor_horizontal_label,
            cursor_vertical,
            cursor_vertical_label,
            info_box,
            cursor_show: false,
            cursor_dock_left: true,
            manager: Manager::new(),
        }
    }

    pub fn new(window: Arc<Window>) -> WgpuCtx<'window, 'font> {
        pollster::block_on(WgpuCtx::new_async(window))
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        let (s_width, s_height) = new_size;
        let s_width = s_width.max(1);
        let s_height = s_height.max(1);
        self.surface_config.width = s_width;
        self.surface_config.height = s_height;
        self.surface.configure(&self.device, &self.surface_config);

        //更新文字画刷
        self.brush
            .as_mut()
            .unwrap()
            .resize_view(s_width as f32, s_height as f32, &self.queue);
        self.brush_top
            .as_mut()
            .unwrap()
            .resize_view(s_width as f32, s_height as f32, &self.queue);

        //更新边框
        self.chart_frame.shape = RectangleFrame {
            x: MARGIN,
            y: MARGIN,
            width: s_width as f32 - MARGIN * 2.0,
            height: s_height as f32 - MARGIN * 2.0 - HINT_HEIGHT,
            vertex: None,
        };
        self.chart_frame.shape.make_vertex(s_width, s_height);
        self.queue.write_buffer(
            &self.chart_frame.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.chart_frame.shape.vertex.unwrap()),
        );

        self.chart_k.shape = RectangleFrame {
            x: MARGIN,
            y: MARGIN,
            width: s_width as f32 - MARGIN * 2.0 - AXIS_Y_WIDTH,
            height: (self.chart_frame.shape.height - AXIS_X_HEIGHT) * 0.7,
            vertex: None,
        };
        self.chart_k.shape.make_vertex(s_width, s_height);
        self.queue.write_buffer(
            &self.chart_k.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.chart_k.shape.vertex.unwrap()),
        );
        self.brush_candle.as_mut().unwrap().resize_view(
            self.chart_k.shape.width,
            self.chart_k.shape.height,
            &self.queue,
        );

        self.chart_volume.shape = RectangleFrame {
            x: MARGIN,
            y: MARGIN + self.chart_k.shape.height,
            width: s_width as f32 - MARGIN * 2.0 - AXIS_Y_WIDTH,
            height: (self.chart_frame.shape.height - AXIS_X_HEIGHT) * 0.3,
            vertex: None,
        };
        self.chart_volume.shape.make_vertex(s_width, s_height);
        self.queue.write_buffer(
            &self.chart_volume.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.chart_volume.shape.vertex.unwrap()),
        );
        //更新屏幕uniform
        self.screen_uniform = ScreenUniform {
            width: s_width as f32,
            height: s_height as f32,
        };
        self.queue.write_buffer(
            &self.screen_buffer,
            0,
            bytemuck::cast_slice(&[self.screen_uniform]),
        );
    }

    pub fn cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.manager.current_cursor_position = (position.x, position.y);

        //平移
        if self.manager.pressed_position.is_some() {
            let delta = position.x - self.manager.pressed_position.unwrap().0;
            let delta_bar_count = (delta / self.chart_k.shape.width as f64
                * (self.manager.right_ix - self.manager.left_ix + 1) as f64)
                as i64;
            self.manager.left_ix = self.manager.pressed_left_right_ix.unwrap().0 - delta_bar_count;
            self.manager.right_ix = self.manager.pressed_left_right_ix.unwrap().1 - delta_bar_count;
            //防止移过界
            let diff = self.manager.right_ix - self.manager.left_ix;
            if self.manager.left_ix < 0 {
                self.manager.left_ix = 0;
                self.manager.right_ix = self.manager.left_ix + diff;
            }
            if self.manager.right_ix > HISTORY.datetime.len() as i64 - 1 {
                self.manager.right_ix = HISTORY.datetime.len() as i64 - 1;
                self.manager.left_ix = self.manager.right_ix - diff;
            }
        }

        //光标与信息栏
        if position.x as f32 >= self.chart_k.shape.x
            && position.x as f32 <= self.chart_k.shape.x + self.chart_k.shape.width
            && position.y as f32 >= self.chart_k.shape.y
            && position.y as f32 <= self.chart_volume.shape.y + self.chart_volume.shape.height
        {
            self.cursor_show = true;

            //垂直光标
            let bar_width = self.chart_k.shape.width
                / (self.manager.right_ix - self.manager.left_ix + 1) as f32;
            self.manager.cursor_ix = ((position.x as f32 - self.chart_k.shape.x) / bar_width)
                as i64
                + self.manager.left_ix;
            self.manager.cursor_ix = self
                .manager
                .cursor_ix
                .min(HISTORY.datetime.len() as i64 - 1);
            self.cursor_vertical.shape = Line {
                x1: position.x as f32,
                y1: self.chart_k.shape.y,
                x2: position.x as f32,
                y2: self.chart_volume.shape.y + self.chart_volume.shape.height,
                vertex: None,
            };
            self.cursor_vertical
                .shape
                .make_vertex(self.surface_config.width, self.surface_config.height);
            self.queue.write_buffer(
                &self.cursor_vertical.vertex_buffer,
                0,
                bytemuck::cast_slice(&self.cursor_vertical.shape.vertex.unwrap()),
            );

            self.cursor_vertical_label.shape = RectangleFilled {
                x: position.x as f32,
                y: self.chart_volume.shape.y + self.chart_volume.shape.height,
                width: 80.0,
                height: 32.0,
                vertex: None,
            };
            self.cursor_vertical_label
                .shape
                .make_vertex(self.surface_config.width, self.surface_config.height);
            self.queue.write_buffer(
                &self.cursor_vertical_label.vertex_buffer,
                0,
                bytemuck::cast_slice(&self.cursor_vertical_label.shape.vertex.unwrap()),
            );

            //水平光标
            //价格框内
            if position.y as f32 >= self.chart_k.shape.y
                && (position.y as f32) < self.chart_k.shape.y + self.chart_k.shape.height
            {
                self.manager.cursor_price = Some(
                    (self.chart_k.shape.y as f64 + self.chart_k.shape.height as f64 - position.y)
                        / self.chart_k.shape.height as f64
                        * (self.manager.max_price_view - self.manager.min_price_view)
                        + self.manager.min_price_view,
                );
                self.cursor_horizontal.shape = Line {
                    x1: self.chart_k.shape.x,
                    y1: position.y as f32,
                    x2: self.chart_k.shape.x + self.chart_k.shape.width,
                    y2: position.y as f32,
                    vertex: None,
                };
                self.cursor_horizontal
                    .shape
                    .make_vertex(self.surface_config.width, self.surface_config.height);
                self.queue.write_buffer(
                    &self.cursor_horizontal.vertex_buffer,
                    0,
                    bytemuck::cast_slice(&self.cursor_horizontal.shape.vertex.unwrap()),
                );

                self.cursor_horizontal_label.shape = RectangleFilled {
                    x: self.chart_k.shape.x + self.chart_k.shape.width,
                    y: position.y as f32 - AXIS_Y_LABEL_BIAS,
                    width: AXIS_Y_WIDTH,
                    height: 16.0,
                    vertex: None,
                };
                self.cursor_horizontal_label
                    .shape
                    .make_vertex(self.surface_config.width, self.surface_config.height);
                self.queue.write_buffer(
                    &self.cursor_horizontal_label.vertex_buffer,
                    0,
                    bytemuck::cast_slice(&self.cursor_horizontal_label.shape.vertex.unwrap()),
                );
            } else {
                self.manager.cursor_price = None;
            }
            //成交量框内
            if position.y as f32 > self.chart_volume.shape.y
                && (position.y as f32) <= self.chart_volume.shape.y + self.chart_volume.shape.height
            {
                self.manager.cursor_volume = Some(
                    (self.chart_volume.shape.y as f64 + self.chart_volume.shape.height as f64
                        - position.y)
                        / self.chart_volume.shape.height as f64
                        * self.manager.max_volume_view,
                );
                self.cursor_horizontal.shape = Line {
                    x1: self.chart_volume.shape.x,
                    y1: position.y as f32,
                    x2: self.chart_volume.shape.x + self.chart_volume.shape.width,
                    y2: position.y as f32,
                    vertex: None,
                };
                self.cursor_horizontal
                    .shape
                    .make_vertex(self.surface_config.width, self.surface_config.height);
                self.queue.write_buffer(
                    &self.cursor_horizontal.vertex_buffer,
                    0,
                    bytemuck::cast_slice(&self.cursor_horizontal.shape.vertex.unwrap()),
                );

                self.cursor_horizontal_label.shape = RectangleFilled {
                    x: self.chart_volume.shape.x + self.chart_volume.shape.width,
                    y: position.y as f32 - AXIS_Y_LABEL_BIAS,
                    width: AXIS_Y_WIDTH,
                    height: 16.0,
                    vertex: None,
                };
                self.cursor_horizontal_label
                    .shape
                    .make_vertex(self.surface_config.width, self.surface_config.height);
                self.queue.write_buffer(
                    &self.cursor_horizontal_label.vertex_buffer,
                    0,
                    bytemuck::cast_slice(&self.cursor_horizontal_label.shape.vertex.unwrap()),
                );
            } else {
                self.manager.cursor_volume = None;
            }

            //信息栏停靠在左边还是右边
            if (position.x as f32) < (self.chart_k.shape.x + INFO_SIZE.0 * 2.0) {
                self.cursor_dock_left = false;
            } else {
                self.cursor_dock_left = true;
            }
            //信息栏
            self.info_box.shape = RectangleFilled {
                x: if self.cursor_dock_left {
                    self.chart_k.shape.x
                } else {
                    self.chart_k.shape.x + self.chart_k.shape.width - INFO_SIZE.0
                },
                y: self.chart_k.shape.y,
                width: INFO_SIZE.0,
                height: INFO_SIZE.1,
                vertex: None,
            };
            self.info_box
                .shape
                .make_vertex(self.surface_config.width, self.surface_config.height);
            self.queue.write_buffer(
                &self.info_box.vertex_buffer,
                0,
                bytemuck::cast_slice(&self.info_box.shape.vertex.unwrap()),
            );
        } else {
            self.cursor_show = false;
        }
    }

    pub fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        if button == MouseButton::Left {
            match state {
                ElementState::Pressed => {
                    self.manager.pressed_position = Some(self.manager.current_cursor_position);
                    self.manager.pressed_left_right_ix =
                        Some((self.manager.left_ix, self.manager.right_ix));
                }
                ElementState::Released => {
                    self.manager.pressed_position = None;
                    self.manager.pressed_left_right_ix = None;
                }
            }
        }
    }

    pub fn mouse_wheel(&mut self, delta: MouseScrollDelta, _phase: TouchPhase) {
        match delta {
            MouseScrollDelta::LineDelta(_x, y) => {
                if y > 0.0 {
                    self.manager.zoom_in_by();
                } else if y < 0.0 {
                    self.manager.zoom_out_by();
                }
            }
            _ => {}
        }
    }

    pub fn keyboard_input(&mut self, key_event: KeyEvent) {
        match key_event.physical_key {
            PhysicalKey::Code(KeyCode::ArrowUp) => {
                self.manager.zoom_in();
            }
            PhysicalKey::Code(KeyCode::ArrowDown) => {
                self.manager.zoom_out();
            }
            _ => (),
        }
    }

    pub fn draw(&mut self) {
        self.manager.update_maxmin_by_left_right_ix();

        //画提示
        let hint1 = Section::default()
            .add_text(Text::new("红色虚线：盈利交易").with_color([1.0, 0.0, 0.0, 1.0]))
            .with_screen_position((10.0, self.surface_config.height as f32 - 70.0));
        let hint2 = Section::default()
            .add_text(Text::new("绿色虚线：亏损交易").with_color([0.0, 1.0, 0.0, 1.0]))
            .with_screen_position((220.0, self.surface_config.height as f32 - 70.0));
        let hint3 = Section::default()
            .add_text(Text::new("黄色向上箭头：买入开仓Buy").with_color([1.0, 1.0, 0.0, 1.0]))
            .with_screen_position((10.0, self.surface_config.height as f32 - 50.0));
        let hint4 = Section::default()
            .add_text(Text::new("黄色向下箭头：卖出平仓Sell").with_color([1.0, 1.0, 0.0, 1.0]))
            .with_screen_position((220.0, self.surface_config.height as f32 - 50.0));
        let hint5 = Section::default()
            .add_text(Text::new("紫色向下箭头：卖出开仓Short").with_color([1.0, 0.0, 1.0, 1.0]))
            .with_screen_position((10.0, self.surface_config.height as f32 - 30.0));
        let hint6 = Section::default()
            .add_text(Text::new("紫色向上箭头：买入平仓Cover").with_color([1.0, 0.0, 1.0, 1.0]))
            .with_screen_position((220.0, self.surface_config.height as f32 - 30.0));
        let mut text_list = vec![hint1, hint2, hint3, hint4, hint5, hint6];

        //画价格刻度值
        let num_axis_price = (self.chart_k.shape.height / 30.0) as usize + 1;
        let item_distance = self.chart_k.shape.height / num_axis_price as f32;
        let item_distance_price =
            (self.manager.max_price_view - self.manager.min_price_view) / num_axis_price as f64;
        let mut axis_price_string = Vec::new();
        for i in 0..=num_axis_price {
            let price;
            if i == num_axis_price {
                price = self.manager.max_price_view;
            } else {
                price = self.manager.min_price_view + i as f64 * item_distance_price;
            }
            axis_price_string.push(price.to_string());
        }
        for i in 0..=num_axis_price {
            let axis_price = Section::default()
                .add_text(Text::new(&axis_price_string[i]).with_color([0.8, 0.8, 0.8, 1.0]))
                .with_screen_position((
                    self.chart_k.shape.x + self.chart_k.shape.width,
                    self.chart_k.shape.y + self.chart_k.shape.height
                        - i as f32 * item_distance
                        - AXIS_Y_LABEL_BIAS,
                ));
            text_list.push(axis_price);
        }

        //画成交量刻度
        let num_axis_volume = (self.chart_volume.shape.height / 30.0) as usize + 1;
        let item_distance = self.chart_volume.shape.height / num_axis_volume as f32;
        let item_distance_volume = self.manager.max_volume_view / num_axis_volume as f64;
        let mut axis_volume_string = Vec::new();
        for i in 0..num_axis_volume {
            let volume = i as f64 * item_distance_volume;
            axis_volume_string.push(volume.to_string());
        }
        for i in 0..num_axis_volume {
            let axis_volume = Section::default()
                .add_text(Text::new(&axis_volume_string[i]).with_color([0.8, 0.8, 0.8, 1.0]))
                .with_screen_position((
                    self.chart_volume.shape.x + self.chart_volume.shape.width,
                    self.chart_volume.shape.y + self.chart_volume.shape.height
                        - i as f32 * item_distance
                        - AXIS_Y_LABEL_BIAS,
                ));
            text_list.push(axis_volume);
        }

        //画日期时间刻度值
        let num_axis_datetime = (self.chart_k.shape.width / 300.0) as usize + 1;
        let item_distance = self.chart_k.shape.width / num_axis_datetime as f32;
        let item_distance_ix =
            (self.manager.right_ix - self.manager.left_ix + 1) as usize / num_axis_datetime;
        let mut axis_datetime_string = Vec::new();
        for i in 0..=num_axis_datetime {
            let ix;
            if i == num_axis_datetime {
                ix = self.manager.right_ix as usize;
            } else {
                ix = self.manager.left_ix as usize + i * item_distance_ix;
            }
            axis_datetime_string.push(
                HISTORY.datetime[ix]
                    .format("%Y-%m-%d\n    %H:%M")
                    .to_string(),
            );
        }
        for i in 0..=num_axis_datetime {
            let axis_datetime = Section::default()
                .add_text(Text::new(&axis_datetime_string[i]).with_color([0.8, 0.8, 0.8, 1.0]))
                .with_screen_position((
                    self.chart_volume.shape.x + i as f32 * item_distance - AXIS_X_LABEL_BIAS,
                    self.chart_volume.shape.y + self.chart_volume.shape.height,
                ));
            text_list.push(axis_datetime);
        }

        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_bind_group(0, &self.camera_bind_group, &[]);
            //画边框
            rpass.set_pipeline(&self.chart_frame.render_pipeline);
            rpass.set_vertex_buffer(0, self.chart_frame.vertex_buffer.slice(..));
            rpass.draw(0..self.chart_frame.shape.vertex.unwrap().len() as u32, 0..1);

            rpass.set_pipeline(&self.chart_k.render_pipeline);
            rpass.set_vertex_buffer(0, self.chart_k.vertex_buffer.slice(..));
            rpass.draw(0..self.chart_k.shape.vertex.unwrap().len() as u32, 0..1);

            rpass.set_pipeline(&self.chart_volume.render_pipeline);
            rpass.set_vertex_buffer(0, self.chart_volume.vertex_buffer.slice(..));
            rpass.draw(
                0..self.chart_volume.shape.vertex.unwrap().len() as u32,
                0..1,
            );

            //画蜡烛图
            self.camera_uniform.update_view_proj_candle(
                self.manager.left_ix,
                self.manager.right_ix,
                self.manager.min_price_view,
                self.manager.max_price_view,
            );
            self.queue.write_buffer(
                &self.camera_buffer_candle,
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );
            rpass.set_viewport(
                self.chart_k.shape.x,
                self.chart_k.shape.y,
                self.chart_k.shape.width,
                self.chart_k.shape.height,
                0.0,
                1.0,
            );
            rpass.set_pipeline(&self.candle_bar.up_render_pipeline);
            rpass.set_vertex_buffer(0, self.candle_bar.up_vertex_buffer.slice(..));
            rpass.draw(0..CANDLE_VERTEX.up.len() as u32, 0..1);
            rpass.set_pipeline(&self.candle_bar.down_render_pipeline);
            rpass.set_vertex_buffer(0, self.candle_bar.down_vertex_buffer.slice(..));
            rpass.draw(0..CANDLE_VERTEX.down.len() as u32, 0..1);
            rpass.set_pipeline(&self.candle_bar.down_hl_render_pipeline);
            rpass.set_vertex_buffer(0, self.candle_bar.down_hl_vertex_buffer.slice(..));
            rpass.draw(0..CANDLE_VERTEX.down_hl.len() as u32, 0..1);
            rpass.set_pipeline(&self.candle_bar.stay_render_pipeline);
            rpass.set_vertex_buffer(0, self.candle_bar.stay_vertex_buffer.slice(..));
            rpass.draw(0..CANDLE_VERTEX.stay.len() as u32, 0..1);

            //画交易对连线
            rpass.set_pipeline(&self.trade.profit_render_pipeline);
            rpass.set_vertex_buffer(0, self.trade.profit_vertex_buffer.slice(..));
            rpass.draw(0..TRADE_PAIRS_VERTEX.profit.len() as u32, 0..1);
            rpass.set_pipeline(&self.trade.loss_render_pipeline);
            rpass.set_vertex_buffer(0, self.trade.loss_vertex_buffer.slice(..));
            rpass.draw(0..TRADE_PAIRS_VERTEX.loss.len() as u32, 0..1);
            //画交易对三角，平仓先画
            rpass.set_pipeline(&self.trade.sell_render_pipeline);
            rpass.set_vertex_buffer(0, self.trade.sell_vertex_buffer.slice(..));
            rpass.draw(0..TRADE_PAIRS_VERTEX.sell.len() as u32, 0..1);
            rpass.set_pipeline(&self.trade.cover_render_pipeline);
            rpass.set_vertex_buffer(0, self.trade.cover_vertex_buffer.slice(..));
            rpass.draw(0..TRADE_PAIRS_VERTEX.cover.len() as u32, 0..1);
            //开仓故意后画
            rpass.set_pipeline(&self.trade.buy_render_pipeline);
            rpass.set_vertex_buffer(0, self.trade.buy_vertex_buffer.slice(..));
            rpass.draw(0..TRADE_PAIRS_VERTEX.buy.len() as u32, 0..1);
            rpass.set_pipeline(&self.trade.short_render_pipeline);
            rpass.set_vertex_buffer(0, self.trade.short_vertex_buffer.slice(..));
            rpass.draw(0..TRADE_PAIRS_VERTEX.short.len() as u32, 0..1);

            //画成交量
            self.camera_uniform.update_view_proj_volume(
                self.manager.left_ix,
                self.manager.right_ix,
                self.manager.max_volume_view,
            );
            self.queue.write_buffer(
                &self.camera_buffer_volume,
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );
            rpass.set_viewport(
                self.chart_volume.shape.x,
                self.chart_volume.shape.y,
                self.chart_volume.shape.width,
                self.chart_volume.shape.height,
                0.0,
                1.0,
            );
            rpass.set_pipeline(&self.volume_bar.up_render_pipeline);
            rpass.set_vertex_buffer(0, self.volume_bar.up_vertex_buffer.slice(..));
            rpass.draw(0..VOLUME_VERTEX.up.len() as u32, 0..1);
            rpass.set_pipeline(&self.volume_bar.down_render_pipeline);
            rpass.set_vertex_buffer(0, self.volume_bar.down_vertex_buffer.slice(..));
            rpass.draw(0..VOLUME_VERTEX.down.len() as u32, 0..1);
            rpass.set_pipeline(&self.volume_bar.stay_render_pipeline);
            rpass.set_vertex_buffer(0, self.volume_bar.stay_vertex_buffer.slice(..));
            rpass.draw(0..VOLUME_VERTEX.stay.len() as u32, 0..1);

            if self.manager.right_ix - self.manager.left_ix + 1
                <= self.chart_k.shape.width as i64 * 5
            {
                //画每个交易成交量的文字
                rpass.set_viewport(
                    self.chart_k.shape.x,
                    self.chart_k.shape.y,
                    self.chart_k.shape.width,
                    self.chart_k.shape.height,
                    0.0,
                    1.0,
                );
                let brush_candle = self.brush_candle.as_mut().unwrap();
                let mut candle_sections = Vec::new();
                //先画平仓文字
                for pos_and_text in TRADE_PAIRS_VERTEX.sell_text.iter() {
                    let vec4 = ((*VP_MATRIX.lock().unwrap()) * pos_and_text.0).to_array();
                    if vec4[0] < -1.2 || vec4[0] > 1.2 {
                        continue;
                    }
                    let pos = (
                        (vec4[0] + 1.0) / 2.0 * self.chart_k.shape.width as f32 - 4.0,
                        (-vec4[1] + 1.0) / 2.0 * self.chart_k.shape.height as f32 - 24.0,
                    );
                    let trade_volume_text = Section::default()
                        .add_text(Text::new(&pos_and_text.1).with_color([1.0, 1.0, 0.0, 1.0]))
                        .with_screen_position(pos);
                    candle_sections.push(trade_volume_text);
                }
                for pos_and_text in TRADE_PAIRS_VERTEX.cover_text.iter() {
                    let vec4 = ((*VP_MATRIX.lock().unwrap()) * pos_and_text.0).to_array();
                    if vec4[0] < -1.2 || vec4[0] > 1.2 {
                        continue;
                    }
                    let pos = (
                        (vec4[0] + 1.0) / 2.0 * self.chart_k.shape.width as f32 - 4.0,
                        (-vec4[1] + 1.0) / 2.0 * self.chart_k.shape.height as f32 + 6.0,
                    );
                    let trade_volume_text = Section::default()
                        .add_text(Text::new(&pos_and_text.1).with_color([1.0, 0.0, 1.0, 1.0]))
                        .with_screen_position(pos);
                    candle_sections.push(trade_volume_text);
                }
                //先画开仓文字
                for pos_and_text in TRADE_PAIRS_VERTEX.buy_text.iter() {
                    let vec4 = ((*VP_MATRIX.lock().unwrap()) * pos_and_text.0).to_array();
                    if vec4[0] < -1.2 || vec4[0] > 1.2 {
                        continue;
                    }
                    let pos = (
                        (vec4[0] + 1.0) / 2.0 * self.chart_k.shape.width as f32 - 4.0,
                        (-vec4[1] + 1.0) / 2.0 * self.chart_k.shape.height as f32 + 6.0,
                    );
                    let trade_volume_text = Section::default()
                        .add_text(Text::new(&pos_and_text.1).with_color([1.0, 1.0, 0.0, 1.0]))
                        .with_screen_position(pos);
                    candle_sections.push(trade_volume_text);
                }
                for pos_and_text in TRADE_PAIRS_VERTEX.short_text.iter() {
                    let vec4 = ((*VP_MATRIX.lock().unwrap()) * pos_and_text.0).to_array();
                    if vec4[0] < -1.2 || vec4[0] > 1.2 {
                        continue;
                    }
                    let pos = (
                        (vec4[0] + 1.0) / 2.0 * self.chart_k.shape.width as f32 - 4.0,
                        (-vec4[1] + 1.0) / 2.0 * self.chart_k.shape.height as f32 - 24.0,
                    );
                    let trade_volume_text = Section::default()
                        .add_text(Text::new(&pos_and_text.1).with_color([1.0, 0.0, 1.0, 1.0]))
                        .with_screen_position(pos);
                    candle_sections.push(trade_volume_text);
                }
                match brush_candle.queue(&self.device, &self.queue, candle_sections) {
                    Ok(_) => (),
                    Err(err) => {
                        panic!("{err}");
                    }
                };
                brush_candle.draw(&mut rpass);
            }
            //画文字（第一阶段）
            let brush = self.brush.as_mut().unwrap();
            rpass.set_viewport(
                0.0,
                0.0,
                self.surface_config.width as f32,
                self.surface_config.height as f32,
                0.0,
                1.0,
            );
            match brush.queue(&self.device, &self.queue, text_list) {
                Ok(_) => (),
                Err(err) => {
                    panic!("{err}");
                }
            };
            brush.draw(&mut rpass);

            if self.cursor_show {
                let mut sections = Vec::new();
                //画光标水平线
                let price_label_string;
                if self.manager.cursor_price.is_some() {
                    rpass.set_pipeline(&self.cursor_horizontal.render_pipeline);
                    rpass.set_vertex_buffer(0, self.cursor_horizontal.vertex_buffer.slice(..));
                    rpass.draw(
                        0..self.cursor_horizontal.shape.vertex.unwrap().len() as u32,
                        0..1,
                    );
                    rpass.set_pipeline(&self.cursor_horizontal_label.render_pipeline);
                    rpass
                        .set_vertex_buffer(0, self.cursor_horizontal_label.vertex_buffer.slice(..));
                    rpass.draw(
                        0..self.cursor_horizontal_label.shape.vertex.unwrap().len() as u32,
                        0..1,
                    );
                    price_label_string = self.manager.cursor_price.unwrap().to_string();
                    let price_label = Section::default()
                        .add_text(Text::new(&price_label_string).with_color([0.0, 0.0, 0.0, 1.0]))
                        .with_screen_position((
                            self.cursor_horizontal_label.shape.x + 1.0,
                            self.cursor_horizontal_label.shape.y,
                        ));
                    sections.push(price_label);
                }
                let volume_label_string;
                if self.manager.cursor_volume.is_some() {
                    rpass.set_pipeline(&self.cursor_horizontal.render_pipeline);
                    rpass.set_vertex_buffer(0, self.cursor_horizontal.vertex_buffer.slice(..));
                    rpass.draw(
                        0..self.cursor_horizontal.shape.vertex.unwrap().len() as u32,
                        0..1,
                    );
                    rpass.set_pipeline(&self.cursor_horizontal_label.render_pipeline);
                    rpass
                        .set_vertex_buffer(0, self.cursor_horizontal_label.vertex_buffer.slice(..));
                    rpass.draw(
                        0..self.cursor_horizontal_label.shape.vertex.unwrap().len() as u32,
                        0..1,
                    );
                    volume_label_string = self.manager.cursor_volume.unwrap().to_string();
                    let volume_label = Section::default()
                        .add_text(Text::new(&volume_label_string).with_color([0.0, 0.0, 0.0, 1.0]))
                        .with_screen_position((
                            self.cursor_horizontal_label.shape.x + 1.0,
                            self.cursor_horizontal_label.shape.y,
                        ));
                    sections.push(volume_label);
                }
                //画光标垂线
                rpass.set_pipeline(&self.cursor_vertical.render_pipeline);
                rpass.set_vertex_buffer(0, self.cursor_vertical.vertex_buffer.slice(..));
                rpass.draw(
                    0..self.cursor_vertical.shape.vertex.unwrap().len() as u32,
                    0..1,
                );
                rpass.set_pipeline(&self.cursor_vertical_label.render_pipeline);
                rpass.set_vertex_buffer(0, self.cursor_vertical_label.vertex_buffer.slice(..));
                rpass.draw(
                    0..self.cursor_vertical_label.shape.vertex.unwrap().len() as u32,
                    0..1,
                );
                let datetime_label_string = HISTORY.datetime[self.manager.cursor_ix as usize]
                    .format("%Y-%m-%d\n    %H:%M")
                    .to_string();
                let datetime_label = Section::default()
                    .add_text(Text::new(&datetime_label_string).with_color([0.0, 0.0, 0.0, 1.0]))
                    .with_screen_position((
                        self.cursor_vertical_label.shape.x + 1.0,
                        self.cursor_vertical_label.shape.y,
                    ));
                //画信息栏
                rpass.set_pipeline(&self.info_box.render_pipeline);
                rpass.set_vertex_buffer(0, self.info_box.vertex_buffer.slice(..));
                rpass.draw(0..self.info_box.shape.vertex.unwrap().len() as u32, 0..1);

                let info_text_string = format!(
                    "Date\n{}\n\nTime\n{}\n\nOpen\n{}\n\nHigh\n{}\n\nLow\n{}\n\nClose\n{}\n\nVolume\n{}",
                    HISTORY.datetime[self.manager.cursor_ix as usize]
                        .format("%Y-%m-%d")
                        .to_string(),
                    HISTORY.datetime[self.manager.cursor_ix as usize].format("%H:%M").to_string(),
                    HISTORY.open_price[self.manager.cursor_ix as usize],
                    HISTORY.high_price[self.manager.cursor_ix as usize],
                    HISTORY.low_price[self.manager.cursor_ix as usize],
                    HISTORY.close_price[self.manager.cursor_ix as usize],
                    HISTORY.volume[self.manager.cursor_ix as usize],
                );
                let info_label = Section::default()
                    .add_text(Text::new(&info_text_string).with_color([0.0, 0.0, 0.0, 1.0]))
                    .with_screen_position((
                        if self.cursor_dock_left {
                            self.chart_k.shape.x
                        } else {
                            self.chart_k.shape.x + self.chart_k.shape.width - INFO_SIZE.0
                        },
                        self.chart_k.shape.y,
                    ));
                //画文字（第二阶段）
                let brush_top = self.brush_top.as_mut().unwrap();
                sections.extend([datetime_label, info_label]);
                match brush_top.queue(&self.device, &self.queue, sections) {
                    Ok(_) => (),
                    Err(err) => {
                        panic!("{err}");
                    }
                };
                brush_top.draw(&mut rpass);
            }
        }
        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}

fn create_pipeline(
    device: &wgpu::Device,
    swap_chain_format: wgpu::TextureFormat,
    fs_main: &str,
    topology: wgpu::PrimitiveTopology,
) -> wgpu::RenderPipeline {
    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[create_vertex_buffer_layout()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some(fs_main),
            compilation_options: Default::default(),
            targets: &[Some(swap_chain_format.into())],
        }),
        primitive: wgpu::PrimitiveState {
            topology,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

fn create_candle_pipeline(
    device: &wgpu::Device,
    swap_chain_format: wgpu::TextureFormat,
    fs_main: &str,
    topology: wgpu::PrimitiveTopology,
    camera_bind_group_layout: &BindGroupLayout,
) -> wgpu::RenderPipeline {
    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[camera_bind_group_layout],
        push_constant_ranges: &[],
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main_candle"),
            buffers: &[create_vertex_buffer_layout()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some(fs_main),
            compilation_options: Default::default(),
            targets: &[Some(swap_chain_format.into())],
        }),
        primitive: wgpu::PrimitiveState {
            topology,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

fn create_volume_pipeline(
    device: &wgpu::Device,
    swap_chain_format: wgpu::TextureFormat,
    fs_main: &str,
    topology: wgpu::PrimitiveTopology,
    camera_bind_group_layout: &BindGroupLayout,
) -> wgpu::RenderPipeline {
    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[camera_bind_group_layout],
        push_constant_ranges: &[],
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main_volume"),
            buffers: &[create_vertex_buffer_layout()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some(fs_main),
            compilation_options: Default::default(),
            targets: &[Some(swap_chain_format.into())],
        }),
        primitive: wgpu::PrimitiveState {
            topology,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

fn create_triangle_pipeline(
    device: &wgpu::Device,
    swap_chain_format: wgpu::TextureFormat,
    vs_main: &str,
    fs_main: &str,
    camera_bind_group_layout: &BindGroupLayout,
) -> wgpu::RenderPipeline {
    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[camera_bind_group_layout],
        push_constant_ranges: &[],
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some(vs_main),
            buffers: &[create_vertex_buffer_layout()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some(fs_main),
            compilation_options: Default::default(),
            targets: &[Some(swap_chain_format.into())],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
