use fission_layout::{TextMeasurer, LineMetric};

pub struct VelloTextMeasurer;

impl TextMeasurer for VelloTextMeasurer {
    fn measure(&self, text: &str, _font_size: f32, available_width: Option<f32>) -> (f32, f32) {
        let char_width = 8.0;
        let line_height = 16.0;
        
        // Simple wrapping logic
        if let Some(width) = available_width {
            let max_chars = (width / char_width).floor() as usize;
            if max_chars == 0 { return (0.0, 0.0); }
            
            let char_count = text.chars().count();
            if char_count == 0 { return (0.0, line_height); }
            
            let lines = (char_count + max_chars - 1) / max_chars;
            let final_width = if lines > 1 { width } else { char_count as f32 * char_width };
            
            (final_width, lines as f32 * line_height)
        } else {
            // No wrapping
            (text.chars().count() as f32 * char_width, line_height)
        }
    }

    fn hit_test(&self, text: &str, _font_size: f32, available_width: Option<f32>, x: f32, y: f32) -> usize {
        let char_width = 8.0;
        let line_height = 16.0;
        
        let line_idx = (y / line_height).floor() as usize;
        let char_idx_in_line = (x / char_width).floor() as usize;
        
        if let Some(width) = available_width {
            let chars_per_line = (width / char_width).floor() as usize;
            if chars_per_line == 0 { return 0; }
            
            let idx = line_idx * chars_per_line + char_idx_in_line;
            idx.min(text.chars().count())
        } else {
            if line_idx > 0 { return text.chars().count(); }
            char_idx_in_line.min(text.chars().count())
        }
    }

    fn get_line_metrics(&self, text: &str, _font_size: f32, available_width: Option<f32>) -> Vec<LineMetric> {
        let char_width = 8.0;
        let line_height = 16.0;
        
        if let Some(width) = available_width {
            let chars_per_line = (width / char_width).floor() as usize;
            if chars_per_line == 0 { return vec![]; }
            
            let total_chars = text.chars().count();
            let mut metrics = Vec::new();
            let mut start = 0;
            
            while start < total_chars {
                let end = (start + chars_per_line).min(total_chars);
                metrics.push(LineMetric {
                    start_index: start,
                    end_index: end,
                    baseline: line_height * 0.8, // Approx baseline
                    height: line_height,
                    width: (end - start) as f32 * char_width,
                });
                start = end;
            }
            if metrics.is_empty() {
                 metrics.push(LineMetric {
                    start_index: 0,
                    end_index: 0,
                    baseline: line_height * 0.8,
                    height: line_height,
                    width: 0.0,
                });
            }
            metrics
        } else {
            vec![LineMetric {
                start_index: 0,
                end_index: text.chars().count(),
                baseline: line_height * 0.8,
                height: line_height,
                width: text.chars().count() as f32 * char_width,
            }]
        }
    }

    fn get_caret_position(&self, text: &str, _font_size: f32, available_width: Option<f32>, caret_index: usize) -> (f32, f32) {
        let char_width = 8.0;
        let line_height = 16.0;
        
        if let Some(width) = available_width {
            let chars_per_line = (width / char_width).floor() as usize;
            if chars_per_line == 0 { return (0.0, 0.0); }
            
            let line = caret_index / chars_per_line;
            let col = caret_index % chars_per_line;
            
            (col as f32 * char_width, line as f32 * line_height)
        } else {
            (caret_index as f32 * char_width, 0.0)
        }
    }
}
