pub mod text;
pub use text::VelloTextMeasurer;

use anyhow::Result;
use fission_render::{DisplayList, DisplayOp, Renderer};
use vello::kurbo::{Affine, Rect, RoundedRect, Stroke};
use vello::peniko::{Color, Fill, Mix};
use vello::Scene;

pub struct VelloRenderer<'a> {
    scene: &'a mut Scene,
    transform_stack: Vec<Affine>,
    current_transform: Affine,
    // Tracks how many Vello layers were pushed in the current Save/Restore scope
    layer_count_stack: Vec<usize>,
    current_layer_count: usize,
}

impl<'a> VelloRenderer<'a> {
    pub fn new(scene: &'a mut Scene) -> Self {
        Self {
            scene,
            transform_stack: Vec::new(),
            current_transform: Affine::IDENTITY,
            layer_count_stack: Vec::new(),
            current_layer_count: 0,
        }
    }
}

impl<'a> Renderer for VelloRenderer<'a> {
    fn render(&mut self, list: &DisplayList) -> Result<()> {
        for op in &list.ops {
            match op {
                DisplayOp::Save => {
                    self.transform_stack.push(self.current_transform);
                    self.layer_count_stack.push(self.current_layer_count);
                    self.current_layer_count = 0;
                }
                DisplayOp::Restore => {
                    // Pop all layers pushed in this scope
                    for _ in 0..self.current_layer_count {
                        self.scene.pop_layer();
                    }
                    
                    // Restore state
                    if let Some(t) = self.transform_stack.pop() {
                        self.current_transform = t;
                    }
                    if let Some(c) = self.layer_count_stack.pop() {
                        self.current_layer_count = c;
                    }
                }
                DisplayOp::Translate(pt) => {
                    let translation = Affine::translate((pt.x as f64, pt.y as f64));
                    self.current_transform = self.current_transform * translation;
                }
                DisplayOp::ClipRect(rect) => {
                    let r = Rect::new(
                        rect.origin.x as f64,
                        rect.origin.y as f64,
                        (rect.origin.x + rect.size.width) as f64,
                        (rect.origin.y + rect.size.height) as f64,
                    );
                    // Push a clip layer. Use current transform? 
                    // Clip applies to subsequent draws.
                    // Vello push_layer transform applies to the clip shape AND content?
                    // Usually clip shape is transformed by `transform`.
                    self.scene.push_layer(Mix::Normal, 1.0, self.current_transform, &r);
                    self.current_layer_count += 1;
                }
                DisplayOp::DrawRect {
                    rect,
                    fill,
                    stroke,
                    corner_radius,
                    ..
                } => {
                    let rect = Rect::new(
                        rect.origin.x as f64,
                        rect.origin.y as f64,
                        (rect.origin.x + rect.size.width) as f64,
                        (rect.origin.y + rect.size.height) as f64,
                    );
                    
                    let shape = RoundedRect::from_rect(rect, *corner_radius as f64);

                    if let Some(f) = fill {
                        let c = Color::rgba8(f.color.r, f.color.g, f.color.b, f.color.a);
                        self.scene.fill(Fill::NonZero, self.current_transform, c, None, &shape);
                    }
                    if let Some(s) = stroke {
                        let c = Color::rgba8(s.color.r, s.color.g, s.color.b, s.color.a);
                        self.scene.stroke(&Stroke::new(s.width as f64), self.current_transform, c, None, &shape);
                    }
                }
                // TODO: DrawText, DrawImage
                _ => {}
            }
        }
        Ok(())
    }
}