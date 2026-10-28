struct Uniforms { 
    mvp: mat4x4<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var smp: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) uv: vec2<f32>) ->VsOut {
    var o: VsOut;
    o.pos = u.mvp * vec4<f32>(pos, 1.0);
    o.uv = uv;
    return o;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let t = textureSample(tex, smp, in.uv);
    return vec4<f32>(t.rgb * u.color.rgb * t.a * u.color.a, t.a);
}