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
                        
                        if let Some(root_id) = cx.ir.root {
                            
                            // Simple recursive traverser (using stack to avoid closure borrow issues if any, though recursion is fine here)
                            // We need pre-order traversal for painter's algorithm (draw parent, then children on top? 
                            // Actually usually background first. Parent background, then children. Yes.)
                            
                            fn generate_display_list(
                                node_id: fission_ir::NodeId,
                                ir: &fission_ir::CoreIR,
                                snapshot: &fission_layout::LayoutSnapshot,
                                list: &mut DisplayList
                            ) {
                                if let Some(geom) = snapshot.nodes.get(&node_id) {
                                    if let Some(node) = ir.nodes.get(&node_id) {
                                        // Debug styling based on Op type
                                        match &node.op {
                                            fission_ir::Op::Layout(fission_ir::LayoutOp::Flex { .. }) => {
                                                // Container: Grey border, no fill (transparent)
                                                list.push(fission_render::DisplayOp::DrawRect { 
                                                    rect: geom.rect,
                                                    fill: None,
                                                    stroke: Some(fission_render::Stroke { 
                                                        color: fission_render::Color { r: 100, g: 100, b: 100, a: 255 }, 
                                                        width: 1.0 
                                                    }),
                                                    bounds: geom.rect,
                                                    node_id: Some(node_id)
                                                });
                                            },
                                            fission_ir::Op::Layout(fission_ir::LayoutOp::Box { .. }) => {
                                                // Leaf/Box: Blue fill, Black border
                                                list.push(fission_render::DisplayOp::DrawRect { 
                                                    rect: geom.rect,
                                                    fill: Some(fission_render::Fill { 
                                                        color: fission_render::Color { r: 100, g: 149, b: 237, a: 255 } // Cornflower Blue
                                                    }),
                                                    stroke: Some(fission_render::Stroke { 
                                                        color: fission_render::Color::BLACK, 
                                                        width: 1.0 
                                                    }),
                                                    bounds: geom.rect,
                                                    node_id: Some(node_id)
                                                });
                                            },
                                            _ => {} // Semantics nodes pass through
                                        }

                                        for child in &node.children {
                                            generate_display_list(*child, ir, snapshot, list);
                                        }
                                    }
                                }
                            }

                            generate_display_list(root_id, &cx.ir, &snapshot, &mut display_list);
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
