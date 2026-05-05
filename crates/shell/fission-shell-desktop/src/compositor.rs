use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use fission_render::LayerClip;
use fission_render::Renderer as _;
use fission_render_vello::{RetainedSceneCache, VelloRenderer, VelloTextMeasurer};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use vello::wgpu;
use vello::{RenderParams, Renderer as VelloSceneRenderer, Scene};
use wgpu::util::DeviceExt;

use crate::pipeline::CompositorTexturePlan;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LayerUniform {
    rect: [f32; 4],
    clip: [f32; 4],
    viewport_and_opacity: [f32; 4],
    transform: [[f32; 4]; 4],
}

struct CachedLayerTexture {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
}

pub struct TextureLayerCompositor {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    textures: HashMap<u64, CachedLayerTexture>,
}

impl TextureLayerCompositor {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fission-compositor shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("compositor.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fission-compositor bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fission-compositor pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fission-compositor pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fission-compositor sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            textures: HashMap::new(),
        }
    }

    pub fn prune(&mut self, live_keys: &HashSet<u64>) {
        self.textures.retain(|key, _| live_keys.contains(key));
    }

    pub fn render_layers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vello_renderer: &mut VelloSceneRenderer,
        retained_scene_cache: &mut RetainedSceneCache,
        measurer: Arc<VelloTextMeasurer>,
        scale_factor: f64,
        viewport_width: u32,
        viewport_height: u32,
        plans: &[CompositorTexturePlan],
        target_view: &wgpu::TextureView,
    ) -> Result<()> {
        let live_keys = plans.iter().map(|plan| plan.key).collect::<HashSet<_>>();
        self.prune(&live_keys);

        let mut draw_batches = Vec::with_capacity(plans.len());
        for plan in plans {
            let width = logical_to_physical(plan.bounds.size.width, scale_factor);
            let height = logical_to_physical(plan.bounds.size.height, scale_factor);
            let mut created = false;
            let cached = self.textures.entry(plan.key).or_insert_with(|| {
                created = true;
                create_cached_texture(device, width, height)
            });
            if cached.width != width || cached.height != height {
                *cached = create_cached_texture(device, width, height);
                created = true;
            }

            if plan.dynamic || created {
                render_plan_scene(
                    device,
                    queue,
                    vello_renderer,
                    retained_scene_cache,
                    Arc::clone(&measurer),
                    scale_factor,
                    plan,
                    &cached.view,
                    width,
                    height,
                )?;
            }

            let clip = clip_rect_to_physical(
                plan.clip.as_ref(),
                scale_factor,
                viewport_width,
                viewport_height,
            );
            let uniform = LayerUniform {
                rect: [
                    (plan.bounds.origin.x as f64 * scale_factor) as f32,
                    (plan.bounds.origin.y as f64 * scale_factor) as f32,
                    width as f32,
                    height as f32,
                ],
                clip: [clip.0 as f32, clip.1 as f32, clip.2 as f32, clip.3 as f32],
                viewport_and_opacity: [
                    viewport_width as f32,
                    viewport_height as f32,
                    plan.opacity,
                    0.0,
                ],
                transform: matrix_to_rows(scale_transform(plan.transform, scale_factor)),
            };
            let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("fission-compositor uniform"),
                contents: bytemuck::bytes_of(&uniform),
                usage: wgpu::BufferUsages::UNIFORM,
            });
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fission-compositor bind group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&cached.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                ],
            });
            draw_batches.push((uniform_buffer, bind_group, clip));
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fission-compositor encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fission-compositor pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 30.0 / 255.0,
                            g: 30.0 / 255.0,
                            b: 30.0 / 255.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            for (_, bind_group, clip) in &draw_batches {
                pass.set_scissor_rect(clip.0, clip.1, clip.2.max(1), clip.3.max(1));
                pass.set_bind_group(0, bind_group, &[]);
                pass.draw(0..4, 0..1);
            }
        }
        queue.submit(Some(encoder.finish()));
        Ok(())
    }
}

fn logical_to_physical(value: f32, scale_factor: f64) -> u32 {
    ((value as f64 * scale_factor).round() as u32).max(1)
}

fn matrix_to_rows(matrix: [f32; 16]) -> [[f32; 4]; 4] {
    [
        [matrix[0], matrix[1], matrix[2], matrix[3]],
        [matrix[4], matrix[5], matrix[6], matrix[7]],
        [matrix[8], matrix[9], matrix[10], matrix[11]],
        [matrix[12], matrix[13], matrix[14], matrix[15]],
    ]
}

fn scale_transform(transform: Option<[f32; 16]>, scale_factor: f64) -> [f32; 16] {
    let mut matrix = transform.unwrap_or([
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]);
    matrix[12] *= scale_factor as f32;
    matrix[13] *= scale_factor as f32;
    matrix
}

fn clip_rect_to_physical(
    clip: Option<&LayerClip>,
    scale_factor: f64,
    viewport_width: u32,
    viewport_height: u32,
) -> (u32, u32, u32, u32) {
    match clip {
        Some(LayerClip::Rect(rect)) => {
            let x = (rect.origin.x as f64 * scale_factor).round().max(0.0) as u32;
            let y = (rect.origin.y as f64 * scale_factor).round().max(0.0) as u32;
            let max_x = ((rect.origin.x + rect.size.width) as f64 * scale_factor)
                .round()
                .clamp(0.0, viewport_width as f64) as u32;
            let max_y = ((rect.origin.y + rect.size.height) as f64 * scale_factor)
                .round()
                .clamp(0.0, viewport_height as f64) as u32;
            let width = max_x.saturating_sub(x).max(1);
            let height = max_y.saturating_sub(y).max(1);
            (x.min(viewport_width), y.min(viewport_height), width, height)
        }
        Some(LayerClip::RoundedRect { rect, .. }) => clip_rect_to_physical(
            Some(&LayerClip::Rect(*rect)),
            scale_factor,
            viewport_width,
            viewport_height,
        ),
        None => (0, 0, viewport_width.max(1), viewport_height.max(1)),
    }
}

fn create_cached_texture(device: &wgpu::Device, width: u32, height: u32) -> CachedLayerTexture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fission-compositor layer texture"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    CachedLayerTexture {
        _texture: texture,
        view,
        width,
        height,
    }
}

fn render_plan_scene(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    vello_renderer: &mut VelloSceneRenderer,
    retained_scene_cache: &mut RetainedSceneCache,
    measurer: Arc<VelloTextMeasurer>,
    scale_factor: f64,
    plan: &CompositorTexturePlan,
    target_view: &wgpu::TextureView,
    width: u32,
    height: u32,
) -> Result<()> {
    let mut scene = Scene::new();
    let mut renderer = VelloRenderer::new(&mut scene, measurer, retained_scene_cache, scale_factor);
    renderer.render_scene(&plan.scene)?;

    let params = RenderParams {
        base_color: vello::peniko::Color::from_rgba8(0, 0, 0, 0),
        width,
        height,
        antialiasing_method: vello::AaConfig::Area,
    };

    vello_renderer.render_to_texture(device, queue, &scene, target_view, &params)?;
    Ok(())
}
