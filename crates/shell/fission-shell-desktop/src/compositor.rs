use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use fission_layout::LayoutPoint;
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
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    content_key: u64,
    base: Option<CachedBaseTexture>,
}

struct CachedBaseTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    scene_cache_key: Option<u64>,
}

struct DrawBatch {
    _uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    clip: (u32, u32, u32, u32),
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
        root_transform: Option<[f32; 16]>,
        plans: &[CompositorTexturePlan],
        target_view: &wgpu::TextureView,
    ) -> Result<()> {
        let mut live_keys = HashSet::new();
        collect_plan_keys(plans, &mut live_keys);
        self.prune(&live_keys);

        let origin = LayoutPoint::ZERO;
        for plan in plans {
            let _ = self.render_plan_texture(
                device,
                queue,
                vello_renderer,
                retained_scene_cache,
                Arc::clone(&measurer),
                scale_factor,
                plan,
            )?;
        }

        let draw_batches = self.build_draw_batches(
            device,
            scale_factor,
            viewport_width,
            viewport_height,
            origin,
            root_transform,
            1.0,
            None,
            plans,
        );
        self.draw_batches_to_view(
            device,
            queue,
            target_view,
            draw_batches,
            wgpu::LoadOp::Clear(wgpu::Color {
                r: 30.0 / 255.0,
                g: 30.0 / 255.0,
                b: 30.0 / 255.0,
                a: 1.0,
            }),
        );
        Ok(())
    }

    fn render_plan_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vello_renderer: &mut VelloSceneRenderer,
        retained_scene_cache: &mut RetainedSceneCache,
        measurer: Arc<VelloTextMeasurer>,
        scale_factor: f64,
        plan: &CompositorTexturePlan,
    ) -> Result<bool> {
        let mut child_changed = false;
        for child in &plan.children {
            child_changed |= self.render_plan_texture(
                device,
                queue,
                vello_renderer,
                retained_scene_cache,
                Arc::clone(&measurer),
                scale_factor,
                child,
            )?;
        }

        let is_virtual_wrapper = plan.scene.is_none();
        if is_virtual_wrapper {
            return Ok(child_changed || plan.composite_dynamic);
        }

        let width = logical_to_physical(plan.bounds.size.width, scale_factor);
        let height = logical_to_physical(plan.bounds.size.height, scale_factor);
        let mut created = false;
        let (previous_content_key, should_use_base_cache) = {
            let cached = self.textures.entry(plan.key).or_insert_with(|| {
                created = true;
                create_cached_texture(device, width, height)
            });
            if cached.width != width || cached.height != height {
                *cached = create_cached_texture(device, width, height);
                created = true;
            }
            (
                cached.content_key,
                !plan.local_dynamic && !plan.children.is_empty() && plan.scene_cache_key.is_some(),
            )
        };

        let needs_recompose = created
            || previous_content_key != plan.content_key
            || plan.local_dynamic
            || child_changed;
        if needs_recompose {
            if should_use_base_cache {
                {
                    let cached = self
                        .textures
                        .get_mut(&plan.key)
                        .expect("missing cached compositor layer");
                    ensure_base_texture(
                        cached,
                        device,
                        queue,
                        vello_renderer,
                        retained_scene_cache,
                        Arc::clone(&measurer),
                        scale_factor,
                        plan.scene.as_ref().expect("static scene missing for base cache"),
                        plan.scene_cache_key,
                        width,
                        height,
                    )?;
                    if let Some(base) = &cached.base {
                        copy_texture_to_texture(
                            device,
                            queue,
                            &base.texture,
                            &cached.texture,
                            width,
                            height,
                        );
                    }
                }
            } else if let Some(scene) = plan.scene.as_ref() {
                let target_view = &self
                    .textures
                    .get(&plan.key)
                    .expect("missing cached compositor layer")
                    .view;
                render_plan_scene(
                    device,
                    queue,
                    vello_renderer,
                    retained_scene_cache,
                    Arc::clone(&measurer),
                    scale_factor,
                    scene,
                    plan.scene_cache_key.filter(|_| !plan.local_dynamic),
                    target_view,
                    width,
                    height,
                )?;
            } else {
                let target_view = &self
                    .textures
                    .get(&plan.key)
                    .expect("missing cached compositor layer")
                    .view;
                clear_target_view(device, queue, target_view, wgpu::Color::TRANSPARENT);
            }

            if !plan.children.is_empty() {
                let target_view = &self
                    .textures
                    .get(&plan.key)
                    .expect("missing cached compositor layer")
                    .view;
                let draw_batches = self.build_draw_batches(
                    device,
                    scale_factor,
                    width,
                    height,
                    plan.bounds.origin,
                    None,
                    1.0,
                    None,
                    &plan.children,
                );
                self.draw_batches_to_view(
                    device,
                    queue,
                    target_view,
                    draw_batches,
                    wgpu::LoadOp::Load,
                );
            }
            if let Some(cached) = self.textures.get_mut(&plan.key) {
                cached.content_key = plan.content_key;
            }
        }
        Ok(needs_recompose || plan.composite_dynamic)
    }

    fn build_draw_batches(
        &self,
        device: &wgpu::Device,
        scale_factor: f64,
        viewport_width: u32,
        viewport_height: u32,
        target_origin: LayoutPoint,
        inherited_transform: Option<[f32; 16]>,
        inherited_opacity: f32,
        inherited_scissor: Option<(u32, u32, u32, u32)>,
        plans: &[CompositorTexturePlan],
    ) -> Vec<DrawBatch> {
        let mut batches = Vec::with_capacity(plans.len());
        for plan in plans {
            let localized = localize_world_transform(plan.transform, target_origin);
            let combined = combine_transforms(inherited_transform, localized);
            let clip = intersect_scissor(
                inherited_scissor,
                plan.clip.as_ref(),
                combined,
                target_origin,
                scale_factor,
                viewport_width,
                viewport_height,
            );
            let opacity = inherited_opacity * plan.opacity;

            if plan.scene.is_none() {
                batches.extend(self.build_draw_batches(
                    device,
                    scale_factor,
                    viewport_width,
                    viewport_height,
                    target_origin,
                    combined,
                    opacity,
                    Some(clip),
                    &plan.children,
                ));
                continue;
            }

            let Some(cached) = self.textures.get(&plan.key) else {
                continue;
            };

            let rect = [
                ((plan.bounds.origin.x - target_origin.x) as f64 * scale_factor) as f32,
                ((plan.bounds.origin.y - target_origin.y) as f64 * scale_factor) as f32,
                cached.width as f32,
                cached.height as f32,
            ];
            let uniform = LayerUniform {
                rect,
                clip: [clip.0 as f32, clip.1 as f32, clip.2 as f32, clip.3 as f32],
                viewport_and_opacity: [
                    viewport_width as f32,
                    viewport_height as f32,
                    opacity,
                    0.0,
                ],
                transform: matrix_to_rows(scale_transform(combined, scale_factor)),
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
            batches.push(DrawBatch {
                _uniform_buffer: uniform_buffer,
                bind_group,
                clip,
            });
        }
        batches
    }

    fn draw_batches_to_view(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_view: &wgpu::TextureView,
        draw_batches: Vec<DrawBatch>,
        load: wgpu::LoadOp<wgpu::Color>,
    ) {
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
                        load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            for batch in &draw_batches {
                pass.set_scissor_rect(
                    batch.clip.0,
                    batch.clip.1,
                    batch.clip.2.max(1),
                    batch.clip.3.max(1),
                );
                pass.set_bind_group(0, &batch.bind_group, &[]);
                pass.draw(0..4, 0..1);
            }
        }
        queue.submit(Some(encoder.finish()));
    }
}

fn collect_plan_keys(plans: &[CompositorTexturePlan], out: &mut HashSet<u64>) {
    for plan in plans {
        if plan.scene.is_some() {
            out.insert(plan.key);
        }
        collect_plan_keys(&plan.children, out);
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
    let mut matrix = transform.unwrap_or(identity_matrix());
    matrix[12] *= scale_factor as f32;
    matrix[13] *= scale_factor as f32;
    matrix
}

fn localize_world_transform(
    transform: Option<[f32; 16]>,
    target_origin: LayoutPoint,
) -> Option<[f32; 16]> {
    let transform = transform?;
    let to_local = translation_matrix(-target_origin.x, -target_origin.y);
    let from_local = translation_matrix(target_origin.x, target_origin.y);
    Some(multiply_matrix(
        to_local,
        multiply_matrix(transform, from_local),
    ))
}

fn combine_transforms(inherited: Option<[f32; 16]>, local: Option<[f32; 16]>) -> Option<[f32; 16]> {
    match (inherited, local) {
        (Some(lhs), Some(rhs)) => Some(multiply_matrix(lhs, rhs)),
        (Some(lhs), None) => Some(lhs),
        (None, Some(rhs)) => Some(rhs),
        (None, None) => None,
    }
}

fn intersect_scissor(
    inherited_scissor: Option<(u32, u32, u32, u32)>,
    clip: Option<&LayerClip>,
    transform: Option<[f32; 16]>,
    target_origin: LayoutPoint,
    scale_factor: f64,
    viewport_width: u32,
    viewport_height: u32,
) -> (u32, u32, u32, u32) {
    let base = clip
        .map(|_| {
            clip_rect_to_physical(
                clip,
                transform,
                target_origin,
                scale_factor,
                viewport_width,
                viewport_height,
            )
        })
        .unwrap_or((0, 0, viewport_width.max(1), viewport_height.max(1)));
    match inherited_scissor {
        Some(parent) => intersect_scissor_rects(parent, base),
        None => base,
    }
}

fn clip_rect_to_physical(
    clip: Option<&LayerClip>,
    transform: Option<[f32; 16]>,
    target_origin: LayoutPoint,
    scale_factor: f64,
    viewport_width: u32,
    viewport_height: u32,
) -> (u32, u32, u32, u32) {
    let Some(rect) = clip_rect(clip) else {
        return (0, 0, viewport_width.max(1), viewport_height.max(1));
    };

    let mut points = [
        LayoutPoint::new(
            rect.origin.x - target_origin.x,
            rect.origin.y - target_origin.y,
        ),
        LayoutPoint::new(
            rect.origin.x + rect.size.width - target_origin.x,
            rect.origin.y - target_origin.y,
        ),
        LayoutPoint::new(
            rect.origin.x - target_origin.x,
            rect.origin.y + rect.size.height - target_origin.y,
        ),
        LayoutPoint::new(
            rect.origin.x + rect.size.width - target_origin.x,
            rect.origin.y + rect.size.height - target_origin.y,
        ),
    ];
    if let Some(matrix) = transform {
        for point in &mut points {
            *point = transform_point(matrix, *point);
        }
    }

    let min_x = points
        .iter()
        .map(|p| p.x)
        .fold(f32::INFINITY, f32::min)
        .max(0.0);
    let min_y = points
        .iter()
        .map(|p| p.y)
        .fold(f32::INFINITY, f32::min)
        .max(0.0);
    let max_x = points
        .iter()
        .map(|p| p.x)
        .fold(f32::NEG_INFINITY, f32::max)
        .min(viewport_width as f32 / scale_factor as f32);
    let max_y = points
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max)
        .min(viewport_height as f32 / scale_factor as f32);

    let x = (min_x as f64 * scale_factor).round().max(0.0) as u32;
    let y = (min_y as f64 * scale_factor).round().max(0.0) as u32;
    let max_x = (max_x as f64 * scale_factor)
        .round()
        .clamp(0.0, viewport_width as f64) as u32;
    let max_y = (max_y as f64 * scale_factor)
        .round()
        .clamp(0.0, viewport_height as f64) as u32;
    (
        x.min(viewport_width),
        y.min(viewport_height),
        max_x.saturating_sub(x).max(1),
        max_y.saturating_sub(y).max(1),
    )
}

fn intersect_scissor_rects(
    lhs: (u32, u32, u32, u32),
    rhs: (u32, u32, u32, u32),
) -> (u32, u32, u32, u32) {
    let left = lhs.0.max(rhs.0);
    let top = lhs.1.max(rhs.1);
    let right = lhs
        .0
        .saturating_add(lhs.2)
        .min(rhs.0.saturating_add(rhs.2));
    let bottom = lhs
        .1
        .saturating_add(lhs.3)
        .min(rhs.1.saturating_add(rhs.3));
    (
        left,
        top,
        right.saturating_sub(left).max(1),
        bottom.saturating_sub(top).max(1),
    )
}

fn clip_rect(clip: Option<&LayerClip>) -> Option<fission_layout::LayoutRect> {
    match clip {
        Some(LayerClip::Rect(rect)) => Some(*rect),
        Some(LayerClip::RoundedRect { rect, .. }) => Some(*rect),
        None => None,
    }
}

fn transform_point(matrix: [f32; 16], point: LayoutPoint) -> LayoutPoint {
    LayoutPoint::new(
        matrix[0] * point.x + matrix[4] * point.y + matrix[12],
        matrix[1] * point.x + matrix[5] * point.y + matrix[13],
    )
}

fn identity_matrix() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

fn translation_matrix(tx: f32, ty: f32) -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, tx, ty, 0.0, 1.0,
    ]
}

fn multiply_matrix(a: [f32; 16], b: [f32; 16]) -> [f32; 16] {
    let mut out = [0.0; 16];
    for row in 0..4 {
        for col in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[row * 4 + k] * b[k * 4 + col];
            }
            out[row * 4 + col] = sum;
        }
    }
    out
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
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    CachedLayerTexture {
        texture,
        view,
        width,
        height,
        content_key: 0,
        base: None,
    }
}

fn create_base_texture(device: &wgpu::Device, width: u32, height: u32) -> CachedBaseTexture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fission-compositor base texture"),
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
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    CachedBaseTexture {
        texture,
        view,
        width,
        height,
        scene_cache_key: None,
    }
}

fn ensure_base_texture(
    cached: &mut CachedLayerTexture,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    vello_renderer: &mut VelloSceneRenderer,
    retained_scene_cache: &mut RetainedSceneCache,
    measurer: Arc<VelloTextMeasurer>,
    scale_factor: f64,
    scene: &fission_render::RenderScene,
    scene_cache_key: Option<u64>,
    width: u32,
    height: u32,
) -> Result<()> {
    let needs_render = {
        let base = cached
            .base
            .get_or_insert_with(|| create_base_texture(device, width, height));
        if base.width != width || base.height != height {
            *base = create_base_texture(device, width, height);
        }
        base.scene_cache_key != scene_cache_key
    };
    if needs_render {
        let base = cached
            .base
            .as_mut()
            .expect("missing base texture after creation");
        render_plan_scene(
            device,
            queue,
            vello_renderer,
            retained_scene_cache,
            measurer,
            scale_factor,
            scene,
            scene_cache_key,
            &base.view,
            width,
            height,
        )?;
        base.scene_cache_key = scene_cache_key;
    }
    Ok(())
}

fn copy_texture_to_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    src: &wgpu::Texture,
    dst: &wgpu::Texture,
    width: u32,
    height: u32,
) {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("fission-compositor copy encoder"),
    });
    encoder.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo {
            texture: src,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyTextureInfo {
            texture: dst,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));
}

fn clear_target_view(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target_view: &wgpu::TextureView,
    color: wgpu::Color,
) {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("fission-compositor clear encoder"),
    });
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fission-compositor clear pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }
    queue.submit(Some(encoder.finish()));
}

fn render_plan_scene(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    vello_renderer: &mut VelloSceneRenderer,
    retained_scene_cache: &mut RetainedSceneCache,
    measurer: Arc<VelloTextMeasurer>,
    scale_factor: f64,
    scene: &fission_render::RenderScene,
    scene_cache_key: Option<u64>,
    target_view: &wgpu::TextureView,
    width: u32,
    height: u32,
) -> Result<()> {
    let params = RenderParams {
        base_color: vello::peniko::Color::from_rgba8(0, 0, 0, 0),
        width,
        height,
        antialiasing_method: vello::AaConfig::Area,
    };

    if let Some(cache_key) = scene_cache_key {
        let cached_scene = retained_scene_cache.get_or_insert_with(cache_key, |scene_cache| {
            let mut encoded = Scene::new();
            let mut renderer = VelloRenderer::new(
                &mut encoded,
                Arc::clone(&measurer),
                scene_cache,
                scale_factor,
            );
            renderer.render_scene(scene)?;
            Ok(encoded)
        })?;
        vello_renderer.render_to_texture(device, queue, cached_scene, target_view, &params)?;
        return Ok(());
    }

    let mut encoded = Scene::new();
    let mut renderer =
        VelloRenderer::new(&mut encoded, measurer, retained_scene_cache, scale_factor);
    renderer.render_scene(scene)?;
    vello_renderer.render_to_texture(device, queue, &encoded, target_view, &params)?;
    Ok(())
}
