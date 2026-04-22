use fission_charts::{Chart, Series, LineSeries, BarSeries, Axis};
use fission_core::op::Color;

#[test]
fn test_chart_builder() {
    let chart = Chart::new(800.0, 600.0)
        .title("Test Chart")
        .x_axis(Axis::category(vec!["A", "B", "C"]))
        .y_axis(Axis::value())
        .series(vec![
            LineSeries::new("Revenue")
                .data(vec![10.0, 20.0, 30.0])
                .color(Color::BLUE)
                .into(),
            BarSeries::new("Cost")
                .data(vec![5.0, 15.0, 25.0])
                .color(Color::RED)
                .into(),
        ])
        .animate(true);

    assert_eq!(chart.title.unwrap(), "Test Chart");
    assert_eq!(chart.series.len(), 2);
    assert_eq!(chart.animate, true);
    assert_eq!(chart.width, 800.0);
    assert_eq!(chart.height, 600.0);
}
