use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use softbuffer::{Context, Surface};
use skia_safe::{ColorType, AlphaType};
use std::num::NonZeroU32;
use std::rc::Rc;
use anyhow::Result;

use fission_shell::Platform;
use fission_render::{Renderer, DisplayList, LayoutRect};
use fission_render_skia::SkiaRenderer;
use fission_core::{Runtime, Clock, Action, ActionId, AppState};
use fission_core::lowering::{Desugar, build_layout_tree, LoweringContext};
use fission_layout::{LayoutEngine, LayoutSize, LayoutInputNode};

pub struct DesktopApp<S: AppState, W: Desugar> {
    runtime: Runtime,
    layout_engine: LayoutEngine,
    root_widget: W,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: AppState + Default, W: Desugar + 'static> DesktopApp<S, W> {
    pub fn new(root_widget: W) -> Self {
        let mut runtime = Runtime::default();
        runtime.add_app_state(Box::new(S::default())).unwrap();
        
        Self {
            runtime,
            layout_engine: LayoutEngine::new(),
            root_widget,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn run(mut self) -> Result<()> {
        let event_loop = EventLoop::new().map_err(|e| anyhow::anyhow!("Event loop error: {}", e))?;
        let window = Rc::new(WindowBuilder::new().with_title("Fission App").build(&event_loop).map_err(|e| anyhow::anyhow!("Window build error: {}", e))?);
        
        // Map errors to string to avoid Send/Sync issues with softbuffer errors
        let context = Context::new(window.clone()).map_err(|e| anyhow::anyhow!("Context creation failed: {:?}", e))?;
        let mut surface = Surface::new(&context, window.clone()).map_err(|e| anyhow::anyhow!("Surface creation failed: {:?}", e))?;

        event_loop.run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent { window_id, event: WindowEvent::RedrawRequested } if window_id == window.id() => {
                    let size = window.inner_size();
                    if let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
                        surface.resize(width, height).unwrap();
                        
                        let mut buffer = surface.buffer_mut().unwrap();
                        let stride = width.get() * 4;
                        let layout_width = size.width as f32;
                        let layout_height = size.height as f32;

                        // 1. Lowering
                        let mut cx = LoweringContext::new();
                        self.root_widget.desugar(&mut cx);
                        let layout_input_nodes = build_layout_tree(&cx.ir);

                        // 2. Layout
                        let viewport = LayoutSize { width: layout_width, height: layout_height };
                        let snapshot = self.layout_engine.compute_layout(&layout_input_nodes, viewport).unwrap();

                        // 3. Render to DisplayList
                        let mut display_list = DisplayList::new(fission_render::LayoutRect::new(0.0, 0.0, layout_width, layout_height));
                        
                        for (id, geom) in &snapshot.nodes {
                            let color = fission_render::Color::BLUE; // Placeholder color
                            
                            display_list.push(fission_render::DisplayOp::DrawRect { 
                                rect: geom.rect,
                                fill: Some(fission_render::Fill { color }),
                                stroke: Some(fission_render::Stroke { color: fission_render::Color::BLACK, width: 1.0 }),
                                bounds: geom.rect,
                                node_id: Some(*id)
                            });
                        }

                        // 4. Render to Skia Surface (Softbuffer)
                        let image_info = skia_safe::ImageInfo::new(
                            (size.width as i32, size.height as i32),
                            ColorType::BGRA8888, 
                            AlphaType::Premul,
                            None,
                        );

                        // Wrap softbuffer using surfaces::wrap_pixels
                        let mut sk_surface = skia_safe::surfaces::wrap_pixels(
                            &image_info,
                            bytemuck::cast_slice_mut(&mut buffer),
                            stride as usize,
                            None
                        ).unwrap();

                        let canvas = sk_surface.canvas();
                        let mut renderer = SkiaRenderer::new(canvas);
                        renderer.render(&display_list).unwrap();
                        
                        buffer.present().unwrap();
                    }
                }
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    elwt.exit();
                }
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            }
        }).map_err(|e| anyhow::anyhow!("Event loop error: {}", e))
    }
}
