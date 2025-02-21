struct VertexInput {
    @location(0) position: vec2f,
    @builtin(vertex_index) index: u32,
};

struct VertexOutput {
   @builtin(position) pos: vec4<f32>,
}

struct FragmentInput {
   @builtin(position) pos: vec4<f32>,
}

@vertex
fn vs_main(vertex_in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4<f32>(vertex_in.position, 0.0, 1.0);
    return out;
}

struct CameraUniform {
    view_proj: mat4x4f,
};
struct ScreenUniform {
    width: f32,
    height: f32,
}
@group(0) @binding(0)
var<uniform> camera_candle: CameraUniform;
@group(0) @binding(1)
var<uniform> camera_volume: CameraUniform;
@group(0) @binding(2)
var<uniform> screen_info: ScreenUniform;

@vertex
fn vs_main_candle(vertex_in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.pos = camera_candle.view_proj * vec4<f32>(vertex_in.position, 0.0, 1.0);
    return out;
}

@vertex
fn vs_main_volume(vertex_in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.pos = camera_volume.view_proj * vec4<f32>(vertex_in.position, 0.0, 1.0);
    return out;
}

@vertex
fn vs_main_buy_cover(vertex_in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.pos = camera_candle.view_proj * vec4<f32>(vertex_in.position, 0.0, 1.0);
    if(vertex_in.index % 3 == 1){
        out.pos.x -= 10.0/screen_info.width;
        out.pos.y -= 20.0/screen_info.height;
    }
    else if(vertex_in.index % 3 == 2){
        out.pos.x += 10.0/screen_info.width;
        out.pos.y -= 20.0/screen_info.height;
    }
    return out;
}

@vertex
fn vs_main_sell_short(vertex_in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.pos = camera_candle.view_proj * vec4<f32>(vertex_in.position, 0.0, 1.0);
    if(vertex_in.index % 3 == 1){
        out.pos.x += 10.0/screen_info.width;
        out.pos.y += 20.0/screen_info.height;
    }
    else if(vertex_in.index % 3 == 2){
        out.pos.x -= 10.0/screen_info.width;
        out.pos.y += 20.0/screen_info.height;
    }
    return out;
}

@fragment
fn fs_main_gray1(fragment_in: FragmentInput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.3, 0.3, 0.3, 1.0);
}

@fragment
fn fs_main_gray2(fragment_in: FragmentInput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.8, 0.8, 0.8, 1.0);
}

@fragment
fn fs_main_label_bg(fragment_in: FragmentInput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 0.759, 1.0);
}

@fragment
fn fs_main_up(fragment_in: FragmentInput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.068, 0.068, 1.0);
}

@fragment
fn fs_main_down(fragment_in: FragmentInput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 1.0, 1.0);
}

@fragment
fn fs_main_stay(fragment_in: FragmentInput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

@fragment
fn fs_main_profit(fragment_in: FragmentInput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}

@fragment
fn fs_main_loss(fragment_in: FragmentInput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}

@fragment
fn fs_main_buy_sell(fragment_in: FragmentInput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 0.0, 1.0);
}

@fragment
fn fs_main_short_cover(fragment_in: FragmentInput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
}
