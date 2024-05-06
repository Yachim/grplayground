#import bevy_sprite::mesh2d_vertex_output::VertexOutput

const PI = 3.141592653589793238462643383279;

@group(2) @binding(0) var up_texture: texture_2d<f32>;
@group(2) @binding(1) var up_sampler: sampler;

@group(2) @binding(2) var down_texture: texture_2d<f32>;
@group(2) @binding(3) var down_sampler: sampler;

@group(2) @binding(4) var left_texture: texture_2d<f32>;
@group(2) @binding(5) var left_sampler: sampler;

@group(2) @binding(6) var right_texture: texture_2d<f32>;
@group(2) @binding(7) var right_sampler: sampler;

@group(2) @binding(8) var forward_texture: texture_2d<f32>;
@group(2) @binding(9) var forward_sampler: sampler;

@group(2) @binding(10) var backward_texture: texture_2d<f32>;
@group(2) @binding(11) var backward_sampler: sampler;

@group(2) @binding(12) var<uniform> skybox_intensity: f32;

@group(2) @binding(13) var<uniform> fov: f32;

@group(2) @binding(14) var<uniform> cam_pos: vec3<f32>;
@group(2) @binding(15) var<uniform> cam_x: vec3<f32>; // cam right
@group(2) @binding(16) var<uniform> cam_y: vec3<f32>; // camup
@group(2) @binding(17) var<uniform> cam_z: vec3<f32>; // the way the camera is facing

@group(2) @binding(18) var accretion_disc_texture: texture_2d<f32>;
@group(2) @binding(19) var accretion_disc_sampler: sampler;
@group(2) @binding(20) var<uniform> accretion_disc_r: f32;
@group(2) @binding(21) var<uniform> accretion_disc_width: f32;
@group(2) @binding(22) var<uniform> accretion_disc_intensity: f32;

@group(2) @binding(23) var<uniform> time: f32;

const STEP_CNT = 50;
const MAX_ORBITS = 2;
const STEP_SIZE = f32(MAX_ORBITS) * 2. * PI / f32(STEP_CNT);

const UP = vec3(0., 1., 0.);
const DOWN = vec3(0., -1., 0.);
const LEFT = vec3(-1., 0., 0.);
const RIGHT = vec3(1., 0., 0.);
const FORWARD = vec3(0., 0., -1.);
const BACKWARD = vec3(0., 0., 1.);

fn second_derivative(u: f32) -> f32 {
    return u * (3. * u - 1.);
}

struct EulerOut {
    u: f32,
    v: f32,
}

fn euler(u: f32, v: f32, delta: f32) -> EulerOut {
    let a = second_derivative(u);
    let new_v = v + a * delta;
    let new_u = u + v * delta;

    return EulerOut(new_u, new_v);
}

struct CubemapOut {
    coords: vec2<f32>,
    direction: i32
}

fn ray_plane_intersect(ray_direction: vec3<f32>, ray_origin: vec3<f32>, normal: vec3<f32>, plane_center: vec3<f32>) -> vec3<f32> {
    // source: https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-plane-and-ray-disk-intersection.html
    let t = dot(plane_center - ray_origin, normal) / dot(ray_direction, normal);
    return ray_origin + t * ray_direction;
}

fn to_cubemap(v_: vec3<f32>) -> CubemapOut {
    var coords = vec2(0., 0.);
    var direction: i32;
    var v = normalize(v_);

    let up = dot(UP, v);
    let down = dot(DOWN, v);
    let left = dot(LEFT, v);
    let right = dot(RIGHT, v);
    let forward = dot(FORWARD, v);
    let backward = dot(BACKWARD, v);

    // FIXME: the directions are reversed
    if (
        up > down &&
        up > left &&
        up > right &&
        up > forward &&
        up > backward
    ) {
        direction = 1; // down
        v /= abs(v.y);
        coords = v.xz;
        coords *= -1.;
    }
    else if (
        down > up &&
        down > left &&
        down > right &&
        down > forward &&
        down > backward
    ) {
        direction = 0; // up
        v /= abs(v.y);
        coords = v.xz;
        coords.x *= -1.;
    }
    else if (
        left > up &&
        left > down &&
        left > right &&
        left > forward &&
        left > backward
    ) {
        direction = 3; // right
        v /= abs(v.x);
        coords = v.zy;
        coords.x *= -1.;
    }
    else if (
        right > up &&
        right > down &&
        right > left &&
        right > forward &&
        right > backward
    ) {
        direction = 2; // left
        v /= abs(v.x);
        coords = v.zy;
    }
    else if (
        forward > up &&
        forward > down &&
        forward > left &&
        forward > right &&
        forward > backward
    ) {
        direction = 5;
        v /= abs(v.z);
        coords = v.xy;
    }
    else if ( // backward
        backward > up &&
        backward > down &&
        backward > left &&
        backward > right &&
        backward > forward
    ) {
        direction = 4;
        v /= abs(v.z);
        coords = v.xy;
        coords.x *= -1.;
    }

    coords /= 2.;
    coords += 0.5;
    return CubemapOut(coords, direction);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let fov_mult = 1. / tan(fov / 2.);

    var uv = mesh.uv;
    uv.y = 1. - uv.y;
    uv *= 2.;
    uv -= 1.;

    var ray = normalize((uv.x * cam_x) + (uv.y * cam_y) + (fov_mult * cam_z));

    let r_c = length(cam_pos);

    let n = normalize(cam_pos);
    let t = normalize(cross(cross(n, ray), n));

    let u0 = 1. / r_c;
    let v0 = -u0 * (dot(ray, n) / dot(ray, t));

    var u = u0;
    var v = v0;

    var pos = cam_pos;
    var phi: f32 = 0.;
    var out_color = vec4(0., 0., 0., 1.);

    let accretion_disc_max_r = accretion_disc_r + accretion_disc_width;

    let accretion_disc_rotated: f32 = time / (accretion_disc_r * sqrt(accretion_disc_r));

    for (var i = 0; i < STEP_CNT; i++) {
        if u >= 0.5 {
            out_color += vec4(0., 0., 0., 1.);
            return out_color;
        }

        if u < 0. {
            break;
        }

        let euler_out = euler(u, v, STEP_SIZE);
        u = euler_out.u;
        v = euler_out.v;
        phi += STEP_SIZE;
        let new_pos = (cos(phi) * n + sin(phi) * t) / u;
        ray = normalize(new_pos - pos);

        // accretion disc
        if (
            (cam_pos.y > 0. && pos.y > 0. && new_pos.y < 0.) ||
            (cam_pos.y < 0. && pos.y < 0. && new_pos.y > 0.) // if I don't do this, it glitches out, there is another disc when the camera turns around.
        ) {
            var plane_normal = vec3(0., 1., 0.);

            let point = ray_plane_intersect(ray, pos, plane_normal, vec3(0., 0., 0.));
            let point_r = length(point);

            if (point_r > accretion_disc_r && point_r < accretion_disc_max_r) {
                var accretion_disc_phi = atan2(point.z, point.x);
                accretion_disc_phi += accretion_disc_rotated;
                accretion_disc_phi += PI;
                accretion_disc_phi %= 2 * PI;

                let coords = vec2(accretion_disc_phi / (2 * PI), (accretion_disc_max_r - point_r) / accretion_disc_width);

                out_color += textureSample(accretion_disc_texture, accretion_disc_sampler, coords) * accretion_disc_intensity;
            }
        }

        pos = new_pos;
    }

    let cubemap = to_cubemap(ray);
    let tex_coords = cubemap.coords;
    let direction = cubemap.direction;

    switch (direction) {
        case 0: {
            // up
            out_color += textureSample(up_texture, up_sampler, tex_coords) * skybox_intensity;
        }
        case 1: {
            // down
            out_color += textureSample(down_texture, down_sampler, tex_coords) * skybox_intensity;
        }
        case 2: {
            // left
            out_color += textureSample(left_texture, left_sampler, tex_coords) * skybox_intensity;
        }
        case 3: {
            // right
            out_color += textureSample(right_texture, right_sampler, tex_coords) * skybox_intensity;
        }
        case 4: {
            // forward
            out_color += textureSample(forward_texture, forward_sampler, tex_coords) * skybox_intensity;
        }
        case 5: {
            // backward
            out_color += textureSample(backward_texture, backward_sampler, tex_coords) * skybox_intensity;
        }
        default: {
            out_color = vec4(1., 0., 1., 1.);
        }
    }

    return out_color;
}