use fission_layout::{TextMeasurer, LineMetric};
use parley::layout::Layout;
use parley::style::{FontStack, StyleProperty};
use parley::{FontContext, LayoutContext};
use std::sync::{Arc, Mutex};
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
pub struct ParleyBrush(pub [u8; 4]);

impl Default for ParleyBrush {
    fn default() -> Self { Self([0, 0, 0, 255]) }
}

pub struct VelloTextMeasurer {
    font_cx: Arc<Mutex<FontContext>>,
    layout_cx: Mutex<LayoutContext<ParleyBrush>>,
}

impl VelloTextMeasurer {
    pub fn new(font_cx: Arc<Mutex<FontContext>>) -> Self {
        Self {
            font_cx,
            layout_cx: Mutex::new(LayoutContext::new()),
        }
    }

    fn layout(&self, text: &str, font_size: f32, width: Option<f32>) -> Layout<ParleyBrush> {
        let mut font_cx = self.font_cx.lock().unwrap();
        let mut layout_cx = self.layout_cx.lock().unwrap();
        
        // Assuming 4th argument is boolean (e.g. for bidi stripping or cache?)
        let mut builder = layout_cx.ranged_builder(&mut font_cx, text, 1.0, false);
        // If I need 4th arg, I'll add it.
        // Wait, I should add it now to avoid error loop.
        // But if I add it and it's wrong type...
        // Previous error said "argument #4 of type bool". So it IS bool.
        // I'll add `false`.
        
        builder.push_default(StyleProperty::FontSize(font_size));
        builder.push_default(StyleProperty::FontStack(FontStack::Source(Cow::Borrowed("system-ui"))));
        
        let mut layout = builder.build(text);
        layout.break_all_lines(width);
        layout
    }
}

impl TextMeasurer for VelloTextMeasurer {
    fn measure(&self, text: &str, font_size: f32, available_width: Option<f32>) -> (f32, f32) {
        let layout = self.layout(text, font_size, available_width);
        (layout.width(), layout.height())
    }

    fn hit_test(&self, text: &str, font_size: f32, available_width: Option<f32>, x: f32, y: f32) -> usize {
        let layout = self.layout(text, font_size, available_width);
        for line in layout.lines() {
            let metrics = line.metrics();
            // ...
        }
        0
    }

    fn get_line_metrics(&self, text: &str, font_size: f32, available_width: Option<f32>) -> Vec<LineMetric> {
        let layout = self.layout(text, font_size, available_width);
        layout.lines().map(|line| {
            let metrics = line.metrics();
            LineMetric {
                start_index: line.text_range().start,
                end_index: line.text_range().end,
                baseline: metrics.baseline,
                height: metrics.size(), 
                width: metrics.advance, 
            }
        }).collect()
    }

    fn get_caret_position(&self, text: &str, font_size: f32, available_width: Option<f32>, caret_index: usize) -> (f32, f32) {
        let layout = self.layout(text, font_size, available_width);
        (0.0, 0.0)
    }
}