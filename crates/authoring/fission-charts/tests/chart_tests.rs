use fission_charts::chart::ChartLowerer;
use fission_charts::{
    Axis, BarSeries, BoxplotSeries, BubbleSeries, CalendarHeatmapSeries, CandlestickSeries, Chart,
    ChartAnimation, ChartAnimationKind, ChartBrush, ChartGraphic, ChartHitKind, ChartInteraction,
    ChartModel, EffectScatterSeries, FunnelSeries, GaugeSeries, GraphNode, GraphSeries, Grid,
    HeatmapSeries, LineSegment, LineSeries, LinesSeries, LiquidfillSeries, MapSeries, MarkArea,
    MarkLine, MarkPoint, ParallelSeries, PictorialBarSeries, PieSeries, PolarBarSeries,
    PolarLineSeries, RadarSeries, SankeySeries, ScatterSeries, SingleAxisSeries, SunburstSeries,
    ThemeRiverSeries, TreeSeries, TreemapNode, TreemapSeries, WordcloudSeries,
};
use fission_core::{env::Env, lowering::LoweringContext, ui::traits::LowerDyn};
use fission_ir::op::{Color, Fill, LayoutOp, PaintOp};

const SIMPLE_GEOJSON: &str = r#"
{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "properties": { "name": "North" },
      "geometry": {
        "type": "Polygon",
        "coordinates": [[[0, 0], [10, 0], [10, 10], [0, 10], [0, 0]]]
      }
    },
    {
      "type": "Feature",
      "properties": { "name": "South" },
      "geometry": {
        "type": "Polygon",
        "coordinates": [[[0, -10], [10, -10], [10, 0], [0, 0], [0, -10]]]
      }
    }
  ]
}
"#;

#[test]
fn test_all_chart_builders() {
    let mut chart = Chart::new()
        .width(800.0)
        .height(600.0)
        .title("Full Supported Chart Test")
        .x_axis(Axis::category(vec!["A", "B", "C"]))
        .y_axis(Axis::value())
        .grid(Grid::new())
        .animate(true);

    let series_list = vec![
        LineSeries::new("Line").data(vec![10.0, 20.0]).into(),
        BarSeries::new("Bar").data(vec![15.0, 25.0]).into(),
        ScatterSeries::new("Scatter")
            .data(vec![(1.0, 2.0), (3.0, 4.0)])
            .into(),
        BubbleSeries::new("Bubble")
            .data(vec![(1.0, 2.0, 12.0), (3.0, 4.0, 24.0)])
            .into(),
        PieSeries::new("Pie")
            .data(vec![("A", 10.0), ("B", 20.0)])
            .into(),
        BoxplotSeries::new("Boxplot")
            .data(vec![vec![1.0, 2.0, 3.0, 4.0, 5.0]])
            .into(),
        CandlestickSeries::new("Candlestick")
            .data(vec![vec![10.0, 20.0, 5.0, 25.0]])
            .into(),
        HeatmapSeries::new("Heatmap")
            .data(vec![(0, 0, 10.0), (0, 1, 20.0)])
            .into(),
        CalendarHeatmapSeries::new("Calendar")
            .range("2026-01-01", "2026-01-31")
            .data(vec![("2026-01-05", 8.0), ("2026-01-12", 13.0)])
            .into(),
        LinesSeries::new("Lines")
            .data(vec![LineSegment::new((0.0, 0.0), (10.0, 8.0), 12.0)])
            .into(),
        GraphSeries::new("Graph")
            .nodes(vec![GraphNode {
                id: "1".into(),
                name: "A".into(),
                value: 10.0,
            }])
            .into(),
        TreeSeries::new("Tree")
            .data(vec![TreemapNode {
                name: "Root".into(),
                value: 100.0,
                children: vec![TreemapNode {
                    name: "Child".into(),
                    value: 40.0,
                    children: vec![],
                }],
            }])
            .into(),
        TreemapSeries::new("Treemap")
            .data(vec![TreemapNode {
                name: "Root".into(),
                value: 100.0,
                children: vec![],
            }])
            .into(),
        RadarSeries::new("Radar")
            .data(vec![vec![10.0, 20.0, 30.0]])
            .into(),
        FunnelSeries::new("Funnel")
            .data(vec![("Stage 1", 100.0), ("Stage 2", 80.0)])
            .into(),
        GaugeSeries::new("Gauge").data(vec![("Speed", 65.0)]).into(),
        MapSeries::new("Map", "demo")
            .geojson(SIMPLE_GEOJSON)
            .data(vec![("North", 10.0), ("South", 20.0)])
            .into(),
        SankeySeries::new("Sankey")
            .nodes(vec![GraphNode {
                id: "1".into(),
                name: "A".into(),
                value: 10.0,
            }])
            .into(),
        ParallelSeries::new("Parallel")
            .data(vec![vec![1.0, 2.0, 3.0]])
            .into(),
        SunburstSeries::new("Sunburst")
            .data(vec![TreemapNode {
                name: "Root".into(),
                value: 100.0,
                children: vec![TreemapNode {
                    name: "Child".into(),
                    value: 40.0,
                    children: vec![],
                }],
            }])
            .into(),
        ThemeRiverSeries::new("River")
            .data(vec![
                ("2026-01-01", 10.0, "A"),
                ("2026-01-01", 20.0, "B"),
                ("2026-01-02", 30.0, "A"),
                ("2026-01-02", 12.0, "B"),
            ])
            .into(),
        PictorialBarSeries::new("Pictorial")
            .data(vec![10.0, 20.0])
            .symbol("rect")
            .into(),
        EffectScatterSeries::new("EffectScatter")
            .data(vec![(10.0, 20.0)])
            .into(),
        LiquidfillSeries::new("Liquidfill").data(vec![0.6]).into(),
        WordcloudSeries::new("Wordcloud")
            .data(vec![("Fission", 100.0), ("Rust", 80.0)])
            .into(),
        PolarBarSeries::new("PolarBar")
            .data(vec![("A", 10.0), ("B", 20.0)])
            .into(),
        PolarLineSeries::new("PolarLine")
            .data(vec![(0.0, 10.0), (90.0, 20.0)])
            .into(),
        SingleAxisSeries::new("SingleAxis")
            .data(vec![(1.0, 8.0), (4.0, 16.0)])
            .into(),
    ];

    chart = chart.series(series_list);

    assert_eq!(chart.title.unwrap(), "Full Supported Chart Test");
    assert_eq!(chart.series.len(), 28);
    assert!(chart.animate);
    assert_eq!(chart.width, Some(800.0));
    assert_eq!(chart.height, Some(600.0));
}

#[test]
fn unsupported_series_emit_diagnostics_instead_of_drawing() {
    let chart = Chart::new().series(vec![
        fission_charts::series::map::MapSeries::new("World", "world")
            .data(vec![("USA", 100.0)])
            .into(),
        fission_charts::series::custom::CustomSeries::new("Custom", "string-callback")
            .data(vec![1.0])
            .into(),
    ]);

    let model = ChartModel::from_chart(&chart);

    assert!(model.series.is_empty());
    assert_eq!(model.diagnostics.len(), 2);
    assert!(model.diagnostics[0].message.contains("GeoJSON"));
    assert!(model.diagnostics[1].message.contains("String-named"));
}

#[test]
fn chart_hit_testing_finds_series_items() {
    let chart = Chart::new()
        .width(400.0)
        .height(300.0)
        .x_axis(Axis::category(vec!["A", "B", "C"]))
        .y_axis(Axis::value())
        .interaction(ChartInteraction::new().emit_events(true))
        .series(vec![BarSeries::new("Orders")
            .data(vec![10.0, 20.0, 30.0])
            .into()]);

    let hit = chart
        .hit_test(400.0, 300.0, fission_layout::LayoutPoint::new(210.0, 120.0))
        .expect("bar item hit");

    assert_eq!(hit.kind, ChartHitKind::SeriesItem);
    assert_eq!(hit.series_name.as_deref(), Some("Orders"));
    assert_eq!(hit.data_index, Some(1));
}

#[test]
fn chart_hit_testing_falls_back_to_nearest_axis_item() {
    let chart = Chart::new()
        .width(400.0)
        .height(300.0)
        .x_axis(Axis::category(vec!["A", "B", "C"]))
        .y_axis(Axis::value())
        .series(vec![LineSeries::new("Revenue")
            .data(vec![10.0, 20.0, 30.0])
            .into()]);

    let hit = chart
        .hit_test(400.0, 300.0, fission_layout::LayoutPoint::new(210.0, 230.0))
        .expect("nearest line item hit");

    assert_eq!(hit.kind, ChartHitKind::SeriesItem);
    assert_eq!(hit.series_name.as_deref(), Some("Revenue"));
    assert_eq!(hit.data_index, Some(1));
}

#[test]
fn chart_hit_testing_supports_horizontal_bars_and_bubbles() {
    let horizontal = Chart::new()
        .width(420.0)
        .height(300.0)
        .x_axis(Axis::value())
        .y_axis(Axis::category(vec!["A", "B", "C"]))
        .interaction(ChartInteraction::new().emit_events(true))
        .series(vec![BarSeries::new("Population")
            .horizontal()
            .data(vec![10.0, 20.0, 30.0])
            .into()]);

    let hit = horizontal
        .hit_test(420.0, 300.0, fission_layout::LayoutPoint::new(210.0, 140.0))
        .expect("horizontal bar item hit");
    assert_eq!(hit.kind, ChartHitKind::SeriesItem);
    assert_eq!(hit.series_name.as_deref(), Some("Population"));

    let bubble = Chart::new()
        .width(420.0)
        .height(300.0)
        .x_axis(Axis::value())
        .y_axis(Axis::value())
        .series(vec![BubbleSeries::new("Markets")
            .data(vec![(10.0, 10.0, 40.0)])
            .radius_range(8.0, 24.0)
            .into()]);
    let hit = bubble
        .hit_test(420.0, 300.0, fission_layout::LayoutPoint::new(210.0, 155.0))
        .expect("bubble hit");
    assert_eq!(hit.series_name.as_deref(), Some("Markets"));
}

#[test]
fn data_zoom_filters_ordered_series_before_domain_resolution() {
    let chart = Chart::new()
        .x_axis(Axis::category(vec!["A", "B", "C", "D", "E"]))
        .y_axis(Axis::value())
        .data_zoom(
            fission_charts::DataZoom::new()
                .start_percent(20.0)
                .end_percent(80.0),
        )
        .series(vec![LineSeries::new("Revenue")
            .data(vec![10.0, 20.0, 30.0, 40.0, 50.0])
            .into()]);

    let model = ChartModel::from_chart(&chart);
    assert_eq!(model.x_categories, vec!["B", "C", "D"]);
    match &model.series[0] {
        fission_charts::ResolvedSeries::Line(line) => {
            assert_eq!(line.values, vec![20.0, 30.0, 40.0]);
        }
        _ => panic!("expected line"),
    }
}

#[test]
fn chart_animation_progress_applies_delay_stagger_and_easing() {
    let animation = ChartAnimation::enter(ChartAnimationKind::Grow)
        .duration_ms(100)
        .delay_ms(10)
        .stagger_ms(20);

    assert_eq!(animation.progress_at(0, 0), 0.0);
    assert_eq!(animation.progress_at(20, 1), 0.0);
    assert!(animation.progress_at(80, 1) > 0.0);
    assert_eq!(animation.progress_at(500, 4), 1.0);
}

#[test]
fn chart_theme_follows_dark_fission_env() {
    let chart = Chart::new()
        .width(420.0)
        .height(300.0)
        .title("Themed")
        .series(vec![PieSeries::new("Share").data(vec![("A", 1.0)]).into()]);

    let lowerer = ChartLowerer { chart };
    let mut env = Env::default();
    env.theme.tokens.colors.surface = Color {
        r: 30,
        g: 30,
        b: 30,
        a: 255,
    };
    env.theme.tokens.colors.background = Color {
        r: 18,
        g: 18,
        b: 18,
        a: 255,
    };
    env.theme.tokens.colors.text_primary = Color {
        r: 230,
        g: 230,
        b: 230,
        a: 255,
    };
    env.theme.tokens.colors.text_secondary = Color {
        r: 160,
        g: 160,
        b: 160,
        a: 255,
    };
    let runtime_state = fission_core::RuntimeState::default();
    let mut cx = LoweringContext::new(&env, &runtime_state, None, None);
    let root_id = cx.next_node_id();
    cx.push_scope(root_id);
    lowerer.lower_dyn(&mut cx);

    let has_dark_surface = cx.ir.nodes.values().any(|node| {
        matches!(
            &node.op,
            fission_ir::Op::Paint(PaintOp::DrawRect {
                fill: Some(Fill::Solid(Color {
                    r: 30,
                    g: 30,
                    b: 30,
                    a: 255
                })),
                ..
            })
        )
    });
    assert!(
        has_dark_surface,
        "chart background should use the active Fission theme"
    );
}

#[test]
fn chart_animation_adds_visible_progress_layer() {
    let base_chart = Chart::new()
        .width(420.0)
        .height(300.0)
        .title("Animated")
        .series(vec![LineSeries::new("Revenue")
            .data(vec![10.0, 20.0, 30.0])
            .into()]);
    let animated_chart = base_chart
        .clone()
        .animation(ChartAnimation::enter(ChartAnimationKind::Sweep).repeat(true));

    let count_rects = |chart: Chart| {
        let lowerer = ChartLowerer { chart };
        let env = Env::default();
        let runtime_state = fission_core::RuntimeState::default();
        let mut cx = LoweringContext::new(&env, &runtime_state, None, None);
        let root_id = cx.next_node_id();
        cx.push_scope(root_id);
        lowerer.lower_dyn(&mut cx);
        cx.ir
            .nodes
            .values()
            .filter(|node| matches!(node.op, fission_ir::Op::Paint(PaintOp::DrawRect { .. })))
            .count()
    };

    assert!(count_rects(animated_chart) >= count_rects(base_chart) + 2);
}

#[test]
fn map_lines_tree_sunburst_and_theme_river_lower_to_paths() {
    let chart = Chart::new().width(800.0).height(600.0).series(vec![
        MapSeries::new("Map", "demo")
            .geojson(SIMPLE_GEOJSON)
            .data(vec![("North", 10.0), ("South", 20.0)])
            .into(),
        LinesSeries::new("Lines")
            .data(vec![
                LineSegment::new((0.0, 0.0), (10.0, 10.0), 8.0),
                LineSegment::new((10.0, 0.0), (0.0, 10.0), 12.0),
            ])
            .effect(true)
            .into(),
        TreeSeries::new("Tree")
            .data(vec![TreemapNode {
                name: "Root".into(),
                value: 100.0,
                children: vec![
                    TreemapNode {
                        name: "Child A".into(),
                        value: 40.0,
                        children: vec![],
                    },
                    TreemapNode {
                        name: "Child B".into(),
                        value: 60.0,
                        children: vec![],
                    },
                ],
            }])
            .into(),
        SunburstSeries::new("Sunburst")
            .data(vec![TreemapNode {
                name: "Root".into(),
                value: 100.0,
                children: vec![TreemapNode {
                    name: "Child".into(),
                    value: 40.0,
                    children: vec![],
                }],
            }])
            .into(),
        ThemeRiverSeries::new("River")
            .data(vec![
                ("2026-01-01", 10.0, "A"),
                ("2026-01-01", 20.0, "B"),
                ("2026-01-02", 30.0, "A"),
                ("2026-01-02", 12.0, "B"),
                ("2026-01-03", 20.0, "A"),
                ("2026-01-03", 24.0, "B"),
            ])
            .into(),
    ]);

    let model = ChartModel::from_chart(&chart);
    assert!(model.diagnostics.is_empty());
    assert_eq!(model.series.len(), 5);

    let lowerer = ChartLowerer { chart };
    let env = Env::default();
    let runtime_state = fission_core::RuntimeState::default();
    let mut cx = LoweringContext::new(&env, &runtime_state, None, None);
    let root_id = cx.next_node_id();
    cx.push_scope(root_id);
    lowerer.lower_dyn(&mut cx);

    let path_count = cx
        .ir
        .nodes
        .values()
        .filter(|node| matches!(node.op, fission_ir::Op::Paint(PaintOp::DrawPath { .. })))
        .count();
    assert!(path_count >= 6);
}

#[test]
fn mark_components_lower_to_paint_nodes() {
    let chart = Chart::new()
        .width(800.0)
        .height(600.0)
        .x_axis(Axis::category(vec!["A", "B", "C"]))
        .y_axis(Axis::value())
        .interaction(
            ChartInteraction::new().brush(ChartBrush::rect().preview_rect(0.2, 0.2, 0.4, 0.4)),
        )
        .mark_area(MarkArea::y_range("Target band", 15.0, 28.0))
        .mark_line(MarkLine::y("Target", 22.0))
        .mark_point(MarkPoint::xy("Peak", 2.0, 30.0))
        .graphic(ChartGraphic::text(
            0.2,
            0.1,
            "annotation",
            fission_core::op::Color::BLUE,
        ))
        .series(vec![LineSeries::new("Revenue")
            .data(vec![10.0, 22.0, 30.0])
            .into()]);

    let lowerer = ChartLowerer { chart };
    let env = Env::default();
    let runtime_state = fission_core::RuntimeState::default();
    let mut cx = LoweringContext::new(&env, &runtime_state, None, None);
    let root_id = cx.next_node_id();
    cx.push_scope(root_id);
    lowerer.lower_dyn(&mut cx);

    let path_count = cx
        .ir
        .nodes
        .values()
        .filter(|node| matches!(node.op, fission_ir::Op::Paint(PaintOp::DrawPath { .. })))
        .count();
    let rect_count = cx
        .ir
        .nodes
        .values()
        .filter(|node| matches!(node.op, fission_ir::Op::Paint(PaintOp::DrawRect { .. })))
        .count();

    assert!(path_count >= 2);
    assert!(rect_count >= 3);
}

#[test]
fn test_chart_lowering() {
    let chart = Chart::new()
        .width(800.0)
        .height(600.0)
        .x_axis(Axis::category(vec!["A", "B", "C"]))
        .y_axis(Axis::value())
        .series(vec![
            BarSeries::new("Bar").data(vec![15.0, 25.0, 35.0]).into(),
            LineSeries::new("Line").data(vec![10.0, 20.0, 30.0]).into(),
        ]);

    let lowerer = ChartLowerer { chart };

    let env = Env::default();
    let runtime_state = fission_core::RuntimeState::default();
    let mut cx = LoweringContext::new(&env, &runtime_state, None, None);

    let root_id = cx.next_node_id();
    cx.push_scope(root_id);

    let generated_id = lowerer.lower_dyn(&mut cx);

    let ir = cx.ir;
    let root_node = ir.nodes.get(&generated_id).expect("Root node should exist");

    // Root should be a ZStack
    match &root_node.op {
        fission_ir::Op::Layout(LayoutOp::ZStack) => {}
        _ => panic!("Expected ZStack LayoutOp for Chart"),
    }

    assert!(
        ir.nodes.len() > 10,
        "Should generate grid, axes, and series nodes"
    );

    // Verify that PaintOps were generated for the Bar chart
    let has_rects = ir
        .nodes
        .values()
        .any(|n| matches!(n.op, fission_ir::Op::Paint(PaintOp::DrawRect { .. })));
    assert!(has_rects, "Bar chart should generate DrawRect PaintOps");

    // Verify that PaintOps were generated for the Line chart
    let has_paths = ir
        .nodes
        .values()
        .any(|n| matches!(n.op, fission_ir::Op::Paint(PaintOp::DrawPath { .. })));
    assert!(has_paths, "Line chart should generate DrawPath PaintOps");
}
