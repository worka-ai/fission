use fission_core::Runtime;
use fission_layout::TextMeasurer;
use std::sync::Arc;

struct MockMeasurer;

impl TextMeasurer for MockMeasurer {
    fn measure(&self, text: &str, font_size: f32, _available_width: Option<f32>) -> (f32, f32) {
        // Simple mock: 10px per char
        let width = text.chars().count() as f32 * 10.0;
        (width, font_size)
    }
}

#[test]
fn test_caret_hit_test_precise() {
    let measurer = Arc::new(MockMeasurer);
    let runtime = Runtime::default().with_measurer(measurer);

    let text = "Hello";
    let font_size = 16.0;
    let viewport_x = 0.0;
    let viewport_w = 100.0;
    let content_w = 50.0;
    let scroll_offset = 0.0;

    // "H"  0-10
    // "e" 10-20
    // "l" 20-30
    // "l" 30-40
    // "o" 40-50

    // Click at 4.0 (Left of center of 'H') -> 0
    assert_eq!(runtime.caret_from_point_in_text(text, font_size, viewport_x, viewport_w, content_w, scroll_offset, 4.0), 0);
    
    // Click at 6.0 (Right of center of 'H') -> 1
    assert_eq!(runtime.caret_from_point_in_text(text, font_size, viewport_x, viewport_w, content_w, scroll_offset, 6.0), 1);
    
    // Click at 14.0 (Left of center of 'e') -> 1
    assert_eq!(runtime.caret_from_point_in_text(text, font_size, viewport_x, viewport_w, content_w, scroll_offset, 14.0), 1);
    
    // Click at 16.0 (Right of center of 'e') -> 2
    assert_eq!(runtime.caret_from_point_in_text(text, font_size, viewport_x, viewport_w, content_w, scroll_offset, 16.0), 2);

    // Click at 55.0 (Past end)
    assert_eq!(runtime.caret_from_point_in_text(text, font_size, viewport_x, viewport_w, content_w, scroll_offset, 55.0), 5);
}