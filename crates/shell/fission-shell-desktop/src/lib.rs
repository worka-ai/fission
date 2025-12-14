use anyhow::Result;
use skia_safe::{AlphaType, ColorType};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use fission_core::lowering::{build_layout_tree, Desugar, LoweringContext};
use fission_core::{AppState, BuildCtx, InputEvent, PointerButton, PointerEvent, Runtime};
use fission_ir::CoreIR;
use fission_layout::{LayoutEngine, LayoutSize};
use fission_render::{Color as RenderColor, DisplayList, LayoutPoint, LayoutRect, Renderer};
use fission_render_skia::SkiaRenderer;

pub struct DesktopApp<S: AppState, W: Desugar> {
    runtime: Runtime,
    layout_engine: LayoutEngine,
    root_widget: W,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: AppState + Default, W: Desugar + 'static> DesktopApp<S, W> {
    pub fn build(ui_builder: impl FnOnce(&mut BuildCtx<S>) -> W) -> Self {
        let mut ctx = BuildCtx::new();
        let root_widget = ui_builder(&mut ctx);

        let mut runtime = Runtime::default();
        runtime.add_app_state(Box::new(S::default())).unwrap();
        // Absorb the registry from the build context
        runtime.absorb_registry(ctx.registry);

        Self {
            runtime,
            layout_engine: LayoutEngine::new(),
            root_widget,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn run(self) -> Result<()> {
        println!("Starting DesktopApp::run");
        let event_loop =
            EventLoop::new().map_err(|e| anyhow::anyhow!("Event loop error: {}", e))?;
        let window = Rc::new(
            WindowBuilder::new()
                .with_title("Fission App")
                .build(&event_loop)
                .map_err(|e| anyhow::anyhow!("Window build error: {}", e))?,
        );

        println!("Window created: {:?}", window.id());

        let context = Context::new(window.clone())
            .map_err(|e| anyhow::anyhow!("Context creation failed: {:?}", e))?;
        let mut surface = Surface::new(&context, window.clone())
            .map_err(|e| anyhow::anyhow!("Surface creation failed: {:?}", e))?;

        println!("Softbuffer surface created");

        window.request_redraw();

        let mut runtime = self.runtime;
        let layout_engine = self.layout_engine;
        let root_widget = self.root_widget;

        let mut last_ir: Option<CoreIR> = None;
        let mut last_snapshot: Option<fission_layout::LayoutSnapshot> = None;
        let mut last_cursor_position: Option<PhysicalPosition<f64>> = None;

        event_loop
            .run(move |event, elwt| {
                elwt.set_control_flow(ControlFlow::Wait);

                match event {
                    Event::WindowEvent { window_id, event } if window_id == window.id() => {
                        match event {
                            WindowEvent::RedrawRequested => {
                                let size = window.inner_size();
                                if let (Some(width), Some(height)) =
                                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                                {
                                    surface.resize(width, height).unwrap();

                                    let mut buffer = surface.buffer_mut().unwrap();
                                    let stride = width.get() * 4;
                                    let layout_width = size.width as f32;
                                    let layout_height = size.height as f32;

                                    let mut cx = LoweringContext::new();
                                    let root_id = root_widget.desugar(&mut cx);
                                    cx.ir.root = Some(root_id);

                                    let layout_input_nodes = build_layout_tree(&cx.ir);

                                    let viewport = LayoutSize {
                                        width: layout_width,
                                        height: layout_height,
                                    };
                                    let snapshot =
                                        layout_engine.compute_layout(&layout_input_nodes, root_id, viewport).unwrap();

                                    let mut display_list = DisplayList::new(LayoutRect::new(
                                        0.0,
                                        0.0,
                                        layout_width,
                                        layout_height,
                                    ));

                                    if let Some(root_id) = cx.ir.root {
                                        fn generate_display_list(
                                            node_id: fission_ir::NodeId,
                                            ir: &fission_ir::CoreIR,
                                            snapshot: &fission_layout::LayoutSnapshot,
                                            list: &mut DisplayList,
                                        ) {
                                            if let Some(geom) = snapshot.nodes.get(&node_id) {
                                                if let Some(node) = ir.nodes.get(&node_id) {
                                                    match &node.op {
                                                        fission_ir::Op::Layout(fission_ir::LayoutOp::Flex { .. }) => {
                                                            list.push(fission_render::DisplayOp::DrawRect {
                                                                rect: geom.rect,
                                                                fill: None,
                                                                stroke: Some(fission_render::Stroke {
                                                                    color: RenderColor { r: 255, g: 0, b: 0, a: 255 },
                                                                    width: 2.0,
                                                                }),
                                                                bounds: geom.rect,
                                                                node_id: Some(node_id),
                                                            });
                                                        }
                                                        fission_ir::Op::Layout(fission_ir::LayoutOp::Box { .. }) => {
                                                            list.push(fission_render::DisplayOp::DrawRect {
                                                                rect: geom.rect,
                                                                fill: Some(fission_render::Fill {
                                                                    color: RenderColor {
                                                                        r: 100,
                                                                        g: 149,
                                                                        b: 237,
                                                                        a: 255,
                                                                    },
                                                                }),
                                                                stroke: Some(fission_render::Stroke {
                                                                    color: RenderColor { r: 0, g: 255, b: 0, a: 255 },
                                                                    width: 2.0,
                                                                }),
                                                                bounds: geom.rect,
                                                                node_id: Some(node_id),
                                                            });
                                                        }
                                                        fission_ir::Op::Paint(fission_ir::PaintOp::DrawText { text, size, color }) => {
                                                            list.push(fission_render::DisplayOp::DrawText {
                                                                text: text.clone(),
                                                                position: geom.rect.origin,
                                                                size: *size,
                                                                color: RenderColor {
                                                                    r: color.r,
                                                                    g: color.g,
                                                                    b: color.b,
                                                                    a: color.a,
                                                                },
                                                                bounds: geom.rect,
                                                                node_id: Some(node_id),
                                                            });
                                                        }
                                                        _ => {}
                                                    }

                                                    for child in &node.children {
                                                        generate_display_list(*child, ir, snapshot, list);
                                                    }
                                                }
                                            }
                                        }
                                        generate_display_list(root_id, &cx.ir, &snapshot, &mut display_list);
                                    }

                                    let image_info = skia_safe::ImageInfo::new(
                                        (size.width as i32, size.height as i32),
                                        ColorType::BGRA8888,
                                        AlphaType::Premul,
                                        None,
                                    );

                                    let slice = bytemuck::cast_slice_mut(&mut buffer);
                                    if let Some(mut sk_surface) = skia_safe::surfaces::wrap_pixels(
                                        &image_info,
                                        slice,
                                        stride as usize,
                                        None,
                                    ) {
                                        let canvas = sk_surface.canvas();
                                        let mut renderer = SkiaRenderer::new(canvas);
                                        renderer.render(&display_list).unwrap();
                                    } else {
                                        eprintln!("Failed to wrap pixels");
                                    }

                                    last_ir = Some(cx.ir.clone());
                                    last_snapshot = Some(snapshot.clone());

                                    buffer.present().unwrap();
                                }
                            }
                            WindowEvent::CursorMoved { position, .. } => {
                                last_cursor_position = Some(position);
                            }
                            WindowEvent::MouseInput { state, button, .. } => {
                                if let Some(pointer_button) = map_mouse_button(button) {
                                    if let (Some(position), Some(ir), Some(snapshot)) = (
                                        last_cursor_position.as_ref(),
                                        last_ir.as_ref(),
                                        last_snapshot.as_ref(),
                                    ) {
                                        if let Some(input_event) = build_pointer_event(
                                            state,
                                            pointer_button,
                                            *position,
                                        ) {
                                            if let Err(err) =
                                                runtime.handle_input(input_event, ir, snapshot)
                                            {
                                                eprintln!("Failed to handle input: {err:?}");
                                            } else {
                                                window.request_redraw();
                                            }
                                        }
                                    }
                                }
                            }
                            WindowEvent::CloseRequested => {
                                elwt.exit();
                            }
                            _ => {}
                        }
                    }
                    Event::AboutToWait => {
                        // Removed window.request_redraw() to prevent continuous redraws
                        // window.request_redraw();
                    }
                    _ => {}
                }
            })
            .map_err(|e| anyhow::anyhow!("Event loop error: {}", e))
    }
}

fn map_mouse_button(button: MouseButton) -> Option<PointerButton> {
    match button {
        MouseButton::Left => Some(PointerButton::Primary),
        MouseButton::Right => Some(PointerButton::Secondary),
        MouseButton::Middle => Some(PointerButton::Middle),
        MouseButton::Other(id) => Some(PointerButton::Other(id as u8)),
        _ => None,
    }
}

fn build_pointer_event(
    state: ElementState,
    button: PointerButton,
    position: PhysicalPosition<f64>,
) -> Option<InputEvent> {
    let point = LayoutPoint::new(position.x as f32, position.y as f32);

    let pointer_event = match state {
        ElementState::Pressed => PointerEvent::Down { point, button },
        ElementState::Released => PointerEvent::Up { point, button },
    };

    Some(InputEvent::Pointer(pointer_event))
}
