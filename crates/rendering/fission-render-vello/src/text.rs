use fission_layout::{TextMeasurer, LineMetric};
use fission_ir::op::TextRun;
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
        
        let mut builder = layout_cx.ranged_builder(&mut font_cx, text, 1.0, false);
        
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

    fn measure_rich_text(&self, runs: &[TextRun], available_width: Option<f32>) -> (f32, f32) {
        if runs.is_empty() {
            return (0.0, 0.0);
        }
        
        let mut font_cx = self.font_cx.lock().unwrap();
        let mut layout_cx = self.layout_cx.lock().unwrap();
        
        let mut full_text = String::new();
        for run in runs {
            full_text.push_str(&run.text);
        }
        
        let mut builder = layout_cx.ranged_builder(&mut font_cx, &full_text, 1.0, false);
        builder.push_default(StyleProperty::FontStack(FontStack::Source(Cow::Borrowed("system-ui"))));
        builder.push_default(StyleProperty::FontSize(16.0));
        
        let mut offset = 0;
        for run in runs {
            let len = run.text.len();
            if len > 0 {
                let range = offset..(offset + len);
                builder.push(StyleProperty::FontSize(run.style.font_size), range.clone());
                let color = run.style.color;
                let brush = ParleyBrush([color.r, color.g, color.b, color.a]);
                builder.push(StyleProperty::Brush(brush), range.clone());
                
                offset += len;
            }
        }
        
        let mut layout = builder.build(&full_text);
        layout.break_all_lines(available_width);
        (layout.width(), layout.height())
    }

    fn hit_test(&self, text: &str, font_size: f32, available_width: Option<f32>, _x: f32, _y: f32) -> usize {
        let layout = self.layout(text, font_size, available_width);
        // Simplified hit test: find line, then glyph. 
        // For now, return 0 or implement properly if needed for selection.
        // Parley layout has hit testing methods?
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
                height: metrics.line_height, 
                width: metrics.advance, 
            }
        }).collect()
    }

    fn get_caret_position(&self, text: &str, font_size: f32, available_width: Option<f32>, caret_index: usize) -> (f32, f32) {
        let layout = self.layout(text, font_size, available_width);
        // Need to find position of char at caret_index.
        // Parley doesn't have simple get_caret_position?
        // It has `hit_test` (point -> index).
        // Index -> Point?
        // Iterating lines and items.
        // For now, return 0,0 (caret might be invisible).
        (0.0, 0.0)
    }
}