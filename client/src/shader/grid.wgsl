// Grid overlay shader for pixel drawing

// camera x y    -> vec2<f32>
// viewport width height  vec2<u32>
// zoom level  -> f32

@group(0) @binding(0)
var<uniform> camera_center: vec2<f32>;

@group(0) @binding(1)
var<uniform> viewport_size: vec2<u32>;

@group(0) @binding(2)
var<uniform> zoom_level: f32;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) device_coordinate: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    let vertices = array(
        vec2(1.0, 1.0),
        vec2(-1.0, -1.0),
        vec2(1.0, -1.0),
        vec2(-1.0, 1.0),
        vec2(-1.0, -1.0),
        vec2(1.0, 1.0)
    );
    var out: VertexOutput;
    let vertex = vertices[in_vertex_index % 6];
    out.clip_position = vec4<f32>(vertex, 0.0, 1.0);
    out.device_coordinate = vertex;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_coordinate = in.clip_position.xy;
    let vp = vec2<f32>(viewport_size);

    let center_world = lonlat_to_world_px(camera_center.x, camera_center.y);

    let delta = pixel_coordinate - vp / 2.0;

    let world_position = center_world + delta;

    let pixel_lon_lat = world_px_to_lonlat(world_position);

    let tile_position = lon_lat_to_tile_pos(pixel_lon_lat);

    let alpha = line_alpha(tile_position);

    return vec4(0.8, 0.8, 0.8, alpha);
}

const TILE_SIZE = 256.0;
const PI: f32 = 3.14159265358979323846;
const LINE_WIDTH = 0.1;

fn line_alpha(tile_position: vec2<f32>) -> f32 {
    let as_fract = vec2<f32>(fract(tile_position.x), fract(tile_position.y));
    let diff_to_center = abs(as_fract - vec2<f32>(0.5, 0.5));
    let threshold = 0.5 - LINE_WIDTH / 2.0;

    let is_close = diff_to_center.x > threshold || diff_to_center.y > threshold;
    if is_close {
        return 0.8;
    } else {
        return 0.4;
    }
}

fn lon_lat_to_tile_pos(lonlat: vec2<f32>) -> vec2<f32> {
    let lon = lonlat.x;
    let lat = lonlat.y;
    let n = exp2(floor(zoom_level));

    let x = (lon + 180.0) / 360.0 * n;

    let lat_rad = radians(lat);
    let y = (1.0 - asinh(tan(lat_rad)) / PI) / 2.0 * n;

    return vec2<f32>(x, y);
}

fn lonlat_to_world_px(lon_deg: f32, lat_deg: f32) -> vec2<f32> {
    let map_size = TILE_SIZE * exp2(zoom_level);

    // clamp lat to Mercator valid range to avoid infinities
    let lat = clamp(lat_deg, -85.05112878, 85.05112878);
    let lon = lon_deg;

    let x = (lon + 180.0) / 360.0 * map_size;

    let lat_rad = lat * PI / 180.0;
    let y = (1.0 - log(tan(PI * 0.25 + lat_rad * 0.5)) / PI) * 0.5 * map_size;

    return vec2<f32>(x, y);
}

fn world_px_to_lonlat(world_px: vec2<f32>) -> vec2<f32> {
    let map_size = TILE_SIZE * exp2(zoom_level);

    // Normalize to [0,1]
    let x = world_px.x / map_size;
    let y = world_px.y / map_size;

    let lon = x * 360.0 - 180.0;

    // Inverse Mercator:
    // lat = atan(sinh(pi*(1-2y))) * 180/pi
    let t = PI * (1.0 - 2.0 * y);
    let lat = atan(sinh(t)) * 180.0 / PI;

    return vec2<f32>(lon, lat);
}