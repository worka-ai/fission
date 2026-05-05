struct LayerUniform {
  rect: vec4<f32>,
  clip: vec4<f32>,
  viewport_and_opacity: vec4<f32>,
  transform: mat4x4<f32>,
};

@group(0) @binding(0) var layer_tex: texture_2d<f32>;
@group(0) @binding(1) var layer_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: LayerUniform;

struct VsOut {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
  var local = array<vec2<f32>, 4>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 1.0),
  );

  let uv = local[vertex_index];
  let px = uniforms.rect.xy + uv * uniforms.rect.zw;
  let transformed = uniforms.transform * vec4<f32>(px, 0.0, 1.0);
  let ndc = vec2<f32>(
    (transformed.x / uniforms.viewport_and_opacity.x) * 2.0 - 1.0,
    1.0 - (transformed.y / uniforms.viewport_and_opacity.y) * 2.0,
  );

  var out: VsOut;
  out.position = vec4<f32>(ndc, 0.0, 1.0);
  out.uv = uv;
  return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  let color = textureSample(layer_tex, layer_sampler, in.uv);
  return vec4<f32>(color.rgb, color.a * uniforms.viewport_and_opacity.z);
}
