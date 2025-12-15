use anyhow::Result;
use fission_layout::TextMeasurer;
use fission_render::{
    BoxShadow, Color as RenderColor, DisplayList, DisplayOp, Fill, ImageFit, Renderer, Stroke,
};
use skia_safe::wrapper::NativeTransmutableWrapper;
use skia_safe::{
    BlurStyle, Canvas, Color as SkColor, FontMgr, MaskFilter, Paint, RRect, Rect, Vector,
};
use std::fs;

pub struct SkiaRenderer<'a> {
    canvas: &'a Canvas,
    font_mgr: FontMgr,
}

impl<'a> SkiaRenderer<'a> {
    pub fn new(canvas: &'a Canvas) -> Self {
        Self {
            canvas,
            font_mgr: FontMgr::new(),
        }
    }
}

pub struct SkiaTextMeasurer;

impl TextMeasurer for SkiaTextMeasurer {
    fn measure(&self, text: &str, font_size: f32, _available_width: Option<f32>) -> (f32, f32) {
        (text.len() as f32 * font_size * 0.6, font_size * 1.2)
    }
}

impl<'r> Renderer for SkiaRenderer<'r> {
    fn render(&mut self, display_list: &DisplayList) -> Result<()> {
        self.canvas.clear(SkColor::WHITE);

        for op in &display_list.ops {
            match op {
                DisplayOp::Save => {
                    self.canvas.save();
                }
                DisplayOp::Restore => {
                    self.canvas.restore();
                }
                DisplayOp::ClipRect(rect) => {
                    self.canvas.clip_rect(
                        Rect::new(rect.x(), rect.y(), rect.right(), rect.bottom()),
                        skia_safe::ClipOp::Intersect,
                        true,
                    );
                }
                DisplayOp::Translate(point) => {
                    self.canvas.translate((point.x, point.y));
                }
                DisplayOp::DrawRect {
                    rect,
                    fill,
                    stroke,
                    corner_radius,
                    shadow,
                    bounds,
                    node_id,
                } => {
                    if let Some(shadow) = shadow {
                        let mut shadow_paint = Paint::default();
                        shadow_paint.set_color(SkColor::from_argb(
                            shadow.color.a,
                            shadow.color.r,
                            shadow.color.g,
                            shadow.color.b,
                        ));
                        shadow_paint.set_mask_filter(MaskFilter::blur(
                            BlurStyle::Normal,
                            shadow.blur_radius,
                            None,
                        ));

                        let shadow_rect = Rect::new(
                            rect.x() + shadow.offset.0,
                            rect.y() + shadow.offset.1,
                            rect.right() + shadow.offset.0,
                            rect.bottom() + shadow.offset.1,
                        );

                        if *corner_radius > 0.0 {
                            self.canvas.draw_rrect(
                                RRect::new_rect_xy(shadow_rect, *corner_radius, *corner_radius),
                                &shadow_paint,
                            );
                        } else {
                            self.canvas.draw_rect(shadow_rect, &shadow_paint);
                        }
                    }

                    if let Some(fill) = fill {
                        let mut paint = Paint::default();
                        paint.set_color(SkColor::from_argb(
                            fill.color.a,
                            fill.color.r,
                            fill.color.g,
                            fill.color.b,
                        ));

                        if *corner_radius > 0.0 {
                            self.canvas.draw_rrect(
                                RRect::new_rect_xy(
                                    Rect::new(rect.x(), rect.y(), rect.right(), rect.bottom()),
                                    *corner_radius,
                                    *corner_radius,
                                ),
                                &paint,
                            );
                        } else {
                            self.canvas.draw_rect(
                                Rect::new(rect.x(), rect.y(), rect.right(), rect.bottom()),
                                &paint,
                            );
                        }
                    }

                    if let Some(stroke) = stroke {
                        let mut paint = Paint::default();
                        paint.set_style(skia_safe::PaintStyle::Stroke);
                        paint.set_color(SkColor::from_argb(
                            stroke.color.a,
                            stroke.color.r,
                            stroke.color.g,
                            stroke.color.b,
                        ));
                        paint.set_stroke_width(stroke.width);

                        if *corner_radius > 0.0 {
                            self.canvas.draw_rrect(
                                RRect::new_rect_xy(
                                    Rect::new(rect.x(), rect.y(), rect.right(), rect.bottom()),
                                    *corner_radius,
                                    *corner_radius,
                                ),
                                &paint,
                            );
                        } else {
                            self.canvas.draw_rect(
                                Rect::new(rect.x(), rect.y(), rect.right(), rect.bottom()),
                                &paint,
                            );
                        }
                    }
                }
                DisplayOp::DrawText {
                    text,
                    position,
                    size,
                    color,
                    bounds,
                    ..
                } => {
                    let mut paint = Paint::default();
                    paint.set_color(SkColor::from_argb(color.a, color.r, color.g, color.b));
                    paint.set_anti_alias(true);

                    if let Some(typeface) = self
                        .font_mgr
                        .match_family_style("Arial", skia_safe::FontStyle::normal())
                    {
                        let font = skia_safe::Font::from_typeface(typeface, *size);
                        let adjusted_y = position.y + size;
                        self.canvas
                            .draw_str(text, (position.x, adjusted_y), &font, &paint);
                    }
                }
                DisplayOp::DrawImage {
                    rect,
                    source,
                    fit,
                    bounds,
                    node_id,
                } => {
                    if let Ok(data) = fs::read(source) {
                        if let Some(image) =
                            skia_safe::Image::from_encoded(skia_safe::Data::new_copy(&data))
                        {
                            let src_rect =
                                Rect::from_wh(image.width() as f32, image.height() as f32);
                            let dst_rect =
                                Rect::new(rect.x(), rect.y(), rect.right(), rect.bottom());
                            self.canvas.draw_image_rect(
                                &image,
                                Some((&src_rect, skia_safe::canvas::SrcRectConstraint::Strict)),
                                dst_rect,
                                &Paint::default(),
                            );
                        }
                    }
                }
                DisplayOp::DrawSurface {
                    rect,
                    surface_id,
                    position,
                    ..
                } => {
                    let mut paint = Paint::default();
                    let r = ((surface_id * 50 + position / 20) % 255) as u8;
                    let g = ((surface_id * 30 + position / 30) % 255) as u8;
                    let b = ((surface_id * 70 + position / 40) % 255) as u8;
                    paint.set_color(SkColor::from_rgb(r, g, b));

                    self.canvas.draw_rect(
                        Rect::new(rect.x(), rect.y(), rect.right(), rect.bottom()),
                        &paint,
                    );
                }
            }
        }
        Ok(())
    }
}
