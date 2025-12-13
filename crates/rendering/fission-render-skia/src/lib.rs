use fission_render::{Renderer, DisplayList, DisplayOp, Color, LayoutRect};
use skia_safe::{Canvas, Paint, Rect, Color as SkColor, Color4f};
use anyhow::Result;

pub struct SkiaRenderer<'a> {
    canvas: &'a Canvas, // Changed from &mut Canvas
}

impl<'a> SkiaRenderer<'a> {
    pub fn new(canvas: &'a Canvas) -> Self { // Changed from &mut Canvas
        Self { canvas }
    }
}

// Helper to convert Fission types to Skia types
fn to_skia_rect(r: &LayoutRect) -> Rect {
    Rect::from_xywh(r.x(), r.y(), r.width(), r.height())
}

fn to_skia_color(c: &Color) -> SkColor {
    SkColor::from_argb(c.a, c.r, c.g, c.b)
}

impl<'a> Renderer for SkiaRenderer<'a> {
    fn render(&mut self, display_list: &DisplayList) -> Result<()> {
        // In skia-safe 0.75, canvas methods take &self (interior mutability via C++ pointer)
        self.canvas.clear(SkColor::WHITE); 

        for op in &display_list.ops {
            match op {
                DisplayOp::DrawRect { rect, fill, stroke, .. } => {
                    let sk_rect = to_skia_rect(rect);
                    
                    if let Some(f) = fill {
                        let mut paint = Paint::default();
                        paint.set_color(to_skia_color(&f.color));
                        paint.set_style(skia_safe::paint::Style::Fill);
                        self.canvas.draw_rect(sk_rect, &paint);
                    }

                    if let Some(s) = stroke {
                        let mut paint = Paint::default();
                        paint.set_color(to_skia_color(&s.color));
                        paint.set_style(skia_safe::paint::Style::Stroke);
                        paint.set_stroke_width(s.width);
                        self.canvas.draw_rect(sk_rect, &paint);
                    }
                }
                DisplayOp::Save => {
                    self.canvas.save();
                }
                DisplayOp::Restore => {
                    self.canvas.restore();
                }
                DisplayOp::Translate(pt) => {
                    self.canvas.translate((pt.x, pt.y));
                }
                DisplayOp::ClipRect(rect) => {
                    self.canvas.clip_rect(to_skia_rect(rect), skia_safe::ClipOp::Intersect, true);
                }
                _ => {
                    // Implement other ops as needed
                }
            }
        }
        Ok(())
    }
}