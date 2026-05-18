use fission::prelude::fission_reducer;
use fission_3d::{Point3D, Primitive3D, Scene3D};
use fission_charts::{
    Axis, BarSeries, BoxplotSeries, CandlestickSeries, Chart, DataValue, DataZoom, Dataset,
    EffectScatterSeries, Encode, FunnelSeries, GaugeSeries, GraphNode, GraphSeries, HeatmapSeries,
    Legend, LineSeries, LiquidfillSeries, MapSeries, ParallelSeries, PictorialBarSeries, PieSeries,
    RadarSeries, SankeySeries, ScatterSeries, SunburstSeries, ThemeRiverSeries, Tooltip,
    TreemapNode, TreemapSeries, VisualMap, WordcloudSeries,
};
use fission_core::op::Color;
use fission_core::ui::{Button, ButtonVariant, Column, Container, Node, Row, Scroll, Text};
use fission_core::{with_reducer, ActionEnvelope, AppState, BuildCtx, View, Widget};
use fission_shell_desktop::DesktopApp;
use serde::{Deserialize, Serialize};

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
      "properties": { "name": "West" },
      "geometry": {
        "type": "Polygon",
        "coordinates": [[[-10, -8], [0, -8], [0, 0], [-10, 0], [-10, -8]]]
      }
    },
    {
      "type": "Feature",
      "properties": { "name": "East" },
      "geometry": {
        "type": "Polygon",
        "coordinates": [[[0, -8], [10, -8], [10, 0], [0, 0], [0, -8]]]
      }
    }
  ]
}
"#;

const SHOWCASE_CATEGORY: usize = usize::MAX;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GalleryState {
    pub selected_category: usize,
    pub selected_chart: usize,
    pub smooth: bool,
    pub gap: f32,
    pub data_scale: f32,
}

impl Default for GalleryState {
    fn default() -> Self {
        Self {
            selected_category: SHOWCASE_CATEGORY,
            selected_chart: 0,
            smooth: true,
            gap: 10.0,
            data_scale: 1.0,
        }
    }
}

impl AppState for GalleryState {}

#[fission_reducer(SelectChart)]
fn select_chart(state: &mut GalleryState, category: usize, chart: usize) {
    state.selected_category = category;
    state.selected_chart = chart;
}

#[fission_reducer(ToggleSmooth)]
fn toggle_smooth(state: &mut GalleryState, _value: bool) {
    state.smooth = !state.smooth;
}

#[fission_reducer(UpdateScale, no_eq)]
fn update_scale(state: &mut GalleryState, value: f32) {
    state.data_scale = value;
}

struct GalleryApp;

impl Widget<GalleryState> for GalleryApp {
    fn build(&self, ctx: &mut BuildCtx<GalleryState>, view: &View<GalleryState>) -> Node {
        let viewport_width = view.viewport_size().width.max(0.0);
        let sidebar_width = (viewport_width * 0.22).clamp(180.0, 260.0);
        let select_chart_id = with_reducer!(ctx, SelectChart(0, 0), select_chart).id;

        let toggle_smooth_id = with_reducer!(ctx, ToggleSmooth(false), toggle_smooth).id;

        let update_scale_id = with_reducer!(ctx, UpdateScale(0.0), update_scale).id;

        // Sidebar labels — indices MUST match the (category, chart) match arms below.
        let categories = vec![
            (
                "Foundational",
                vec![
                    "Line & Bar",
                    "Stacked Area",
                    "Step Line",
                    "Donut Pie",
                    "Scatter Visual",
                ],
            ),
            (
                "Statistical",
                vec!["Boxplot", "Candlestick", "Heatmap", "Radar", "Funnel"],
            ),
            (
                "Relationships + Geo",
                vec![
                    "Graph",
                    "Treemap",
                    "Sunburst",
                    "Sankey",
                    "Theme River",
                    "Parallel",
                    "Choropleth Map",
                ],
            ),
            (
                "Dynamic",
                vec![
                    "Gauge",
                    "PictorialBar",
                    "EffectScatter",
                    "Liquidfill",
                    "Wordcloud",
                ],
            ),
            ("3D + Dataset", vec!["Dataset Demo", "3D Scene"]),
        ];

        let mut sidebar_items = vec![
            Text::new("Chart Gallery")
                .size(24.0)
                .color(Color::WHITE)
                .into_node(),
            Button {
                variant: if view.state.selected_category == SHOWCASE_CATEGORY {
                    ButtonVariant::Filled
                } else {
                    ButtonVariant::Ghost
                },
                on_press: Some(ActionEnvelope {
                    id: select_chart_id,
                    payload: serde_json::to_vec(&SelectChart(SHOWCASE_CATEGORY, 0)).unwrap(),
                }),
                child: Some(Box::new(
                    Text::new("Showcase overview")
                        .size(13.0)
                        .color(Color::WHITE)
                        .into_node(),
                )),
                ..Default::default()
            }
            .into_node(),
            fission_widgets::Spacer {
                height: Some(16.0),
                ..Default::default()
            }
            .into_node(),
        ];

        for (cat_idx, (cat_name, charts)) in categories.iter().enumerate() {
            sidebar_items.push(
                Text::new(*cat_name)
                    .size(14.0)
                    .color(Color {
                        r: 180,
                        g: 180,
                        b: 180,
                        a: 255,
                    })
                    .into_node(),
            );

            for (chart_idx, chart_name) in charts.iter().enumerate() {
                let is_selected = view.state.selected_category == cat_idx
                    && view.state.selected_chart == chart_idx;

                sidebar_items.push(
                    Button {
                        variant: ButtonVariant::Ghost,
                        on_press: Some(ActionEnvelope {
                            id: select_chart_id,
                            payload: serde_json::to_vec(&SelectChart(cat_idx, chart_idx)).unwrap(),
                        }),
                        child: Some(Box::new(
                            Text::new(*chart_name)
                                .size(13.0)
                                .color(if is_selected {
                                    Color::WHITE
                                } else {
                                    Color {
                                        r: 160,
                                        g: 160,
                                        b: 160,
                                        a: 255,
                                    }
                                })
                                .into_node(),
                        )),
                        ..Default::default()
                    }
                    .into_node(),
                );
            }
            sidebar_items.push(
                fission_widgets::Spacer {
                    height: Some(8.0),
                    ..Default::default()
                }
                .into_node(),
            );
        }

        let sidebar = Container::new(
            Scroll {
                direction: fission_core::FlexDirection::Column,
                child: Some(Box::new(
                    Column {
                        children: sidebar_items,
                        gap: Some(4.0),
                        ..Default::default()
                    }
                    .into_node(),
                )),
                show_scrollbar: true,
                flex_grow: 1.0,
                ..Default::default()
            }
            .into_node(),
        )
        .width(sidebar_width)
        .padding_all(12.0)
        .bg(Color {
            r: 30,
            g: 30,
            b: 30,
            a: 255,
        })
        .flex_shrink(0.0)
        .into_node();

        let s = view.state.data_scale;
        let content_width = (viewport_width - sidebar_width - 64.0).max(360.0);

        let chart_node = match (view.state.selected_category, view.state.selected_chart) {
            (SHOWCASE_CATEGORY, 0) => build_showcase(ctx, view, content_width, s),
            (0, 0) => Chart::new()
                .title("Foundational: Line & Bar")
                .x_axis(Axis::category(vec![
                    "Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun",
                ]))
                .y_axis(Axis::value())
                .legend(Legend::top_right())
                .tooltip(Tooltip::axis_trigger())
                .series(vec![
                    BarSeries::new("Direct")
                        .data(vec![
                            320.0 * s,
                            332.0 * s,
                            301.0 * s,
                            334.0 * s,
                            390.0 * s,
                            330.0 * s,
                            320.0 * s,
                        ])
                        .color(Color {
                            r: 84,
                            g: 112,
                            b: 198,
                            a: 255,
                        })
                        .into(),
                    LineSeries::new("Email")
                        .data(vec![
                            120.0 * s,
                            132.0 * s,
                            101.0 * s,
                            134.0 * s,
                            90.0 * s,
                            230.0 * s,
                            210.0 * s,
                        ])
                        .color(Color {
                            r: 145,
                            g: 204,
                            b: 117,
                            a: 255,
                        })
                        .smooth(view.state.smooth)
                        .into(),
                ])
                .build(ctx, view),
            (0, 1) => Chart::new()
                .title("Foundational: Stacked Area")
                .x_axis(Axis::category(vec![
                    "Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun",
                ]))
                .y_axis(Axis::value())
                .legend(Legend::top_right())
                .data_zoom(DataZoom::new().start_percent(10.0).end_percent(90.0))
                .series(vec![
                    LineSeries::new("Email")
                        .stack("total")
                        .area_style(Color {
                            r: 145,
                            g: 204,
                            b: 117,
                            a: 100,
                        })
                        .data(vec![
                            120.0 * s,
                            132.0 * s,
                            101.0 * s,
                            134.0 * s,
                            90.0 * s,
                            230.0 * s,
                            210.0 * s,
                        ])
                        .color(Color {
                            r: 145,
                            g: 204,
                            b: 117,
                            a: 255,
                        })
                        .smooth(view.state.smooth)
                        .into(),
                    LineSeries::new("Video Ads")
                        .stack("total")
                        .area_style(Color {
                            r: 84,
                            g: 112,
                            b: 198,
                            a: 100,
                        })
                        .data(vec![
                            150.0 * s,
                            232.0 * s,
                            201.0 * s,
                            154.0 * s,
                            190.0 * s,
                            330.0 * s,
                            410.0 * s,
                        ])
                        .color(Color {
                            r: 84,
                            g: 112,
                            b: 198,
                            a: 255,
                        })
                        .smooth(view.state.smooth)
                        .into(),
                ])
                .build(ctx, view),
            (0, 2) => Chart::new()
                .title("Foundational: Step Line")
                .x_axis(Axis::category(vec![
                    "Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun",
                ]))
                .y_axis(Axis::value())
                .series(vec![LineSeries::new("Step Start")
                    .step("middle")
                    .data(vec![
                        120.0 * s,
                        132.0 * s,
                        101.0 * s,
                        134.0 * s,
                        90.0 * s,
                        230.0 * s,
                        210.0 * s,
                    ])
                    .color(Color {
                        r: 250,
                        g: 200,
                        b: 88,
                        a: 255,
                    })
                    .into()])
                .build(ctx, view),
            (0, 3) => Chart::new()
                .title("Foundational: Donut Pie")
                .legend(Legend::top_right())
                .tooltip(Tooltip::item_trigger())
                .series(vec![PieSeries::new("Access Source")
                    .inner_radius(52.0)
                    .data(vec![
                        ("Search Engine", 1048.0 * s),
                        ("Direct", 735.0 * s),
                        ("Email", 580.0 * s),
                        ("Union Ads", 484.0 * s),
                        ("Video Ads", 300.0 * s),
                    ])
                    .into()])
                .build(ctx, view),
            (0, 4) => Chart::new()
                .title("Foundational: Scatter Visual")
                .x_axis(Axis::value())
                .y_axis(Axis::value())
                .visual_map(
                    VisualMap::new()
                        .min(6.0 * s)
                        .max(11.0 * s)
                        .in_range_colors(vec![
                            Color {
                                r: 59,
                                g: 130,
                                b: 246,
                                a: 255,
                            },
                            Color {
                                r: 250,
                                g: 204,
                                b: 21,
                                a: 255,
                            },
                            Color {
                                r: 239,
                                g: 68,
                                b: 68,
                                a: 255,
                            },
                        ]),
                )
                .series(vec![ScatterSeries::new("Data")
                    .data(vec![
                        (10.0 * s, 8.04 * s),
                        (8.0 * s, 6.95 * s),
                        (13.0 * s, 7.58 * s),
                        (9.0 * s, 8.81 * s),
                        (11.0 * s, 8.33 * s),
                        (14.0 * s, 9.96 * s),
                    ])
                    .color(Color {
                        r: 250,
                        g: 200,
                        b: 88,
                        a: 255,
                    })
                    .into()])
                .build(ctx, view),
            (1, 0) => Chart::new()
                .title("Statistical: Boxplot")
                .x_axis(Axis::category(vec!["expr 1", "expr 2", "expr 3"]))
                .y_axis(Axis::value())
                .series(vec![BoxplotSeries::new("Boxplot")
                    .data(vec![
                        vec![850.0 * s, 960.0 * s, 1060.0 * s, 1080.0 * s, 1100.0 * s],
                        vec![800.0 * s, 850.0 * s, 900.0 * s, 930.0 * s, 980.0 * s],
                        vec![750.0 * s, 800.0 * s, 850.0 * s, 900.0 * s, 1000.0 * s],
                    ])
                    .color(Color {
                        r: 115,
                        g: 192,
                        b: 222,
                        a: 255,
                    })
                    .into()])
                .build(ctx, view),
            (1, 1) => {
                Chart::new()
                    .title("Statistical: Candlestick")
                    .x_axis(Axis::category(vec![
                        "2017-10-24",
                        "2017-10-25",
                        "2017-10-26",
                        "2017-10-27",
                    ]))
                    .y_axis(Axis::value())
                    .series(vec![CandlestickSeries::new("Data")
                        .data(vec![
                            vec![20.0 * s, 34.0 * s, 10.0 * s, 38.0 * s], // open, close, lowest, highest
                            vec![40.0 * s, 35.0 * s, 30.0 * s, 50.0 * s],
                            vec![31.0 * s, 38.0 * s, 33.0 * s, 44.0 * s],
                            vec![38.0 * s, 15.0 * s, 5.0 * s, 42.0 * s],
                        ])
                        .into()])
                    .build(ctx, view)
            }
            (1, 2) => Chart::new()
                .title("Statistical: Heatmap")
                .x_axis(Axis::category(vec!["12a", "1a", "2a", "3a"]))
                .y_axis(Axis::category(vec!["Sat", "Fri", "Thu"]))
                .visual_map(VisualMap::new().min(0.0).max(8.0 * s).in_range_colors(vec![
                    Color {
                        r: 219,
                        g: 234,
                        b: 254,
                        a: 255,
                    },
                    Color {
                        r: 96,
                        g: 165,
                        b: 250,
                        a: 255,
                    },
                    Color {
                        r: 30,
                        g: 64,
                        b: 175,
                        a: 255,
                    },
                ]))
                .series(vec![HeatmapSeries::new("Punch Card")
                    .data(vec![
                        (0, 0, 5.0 * s),
                        (0, 1, 1.0 * s),
                        (0, 2, 0.0 * s),
                        (1, 0, 3.0 * s),
                        (1, 1, 0.0 * s),
                        (1, 2, 0.0 * s),
                        (2, 0, 4.0 * s),
                        (2, 1, 2.0 * s),
                        (2, 2, 0.0 * s),
                        (3, 0, 1.0 * s),
                        (3, 1, 0.0 * s),
                        (3, 2, 8.0 * s),
                    ])
                    .into()])
                .build(ctx, view),
            (1, 3) => Chart::new()
                .title("Statistical: Radar")
                .series(vec![RadarSeries::new("Budget vs spending")
                    .data(vec![
                        vec![42.0 * s, 30.0 * s, 20.0 * s, 35.0 * s, 50.0 * s, 18.0 * s],
                        vec![50.0 * s, 14.0 * s, 28.0 * s, 26.0 * s, 42.0 * s, 21.0 * s],
                    ])
                    .into()])
                .build(ctx, view),
            (1, 4) => Chart::new()
                .title("Statistical: Funnel")
                .series(vec![FunnelSeries::new("Expected")
                    .data(vec![
                        ("Visit", 100.0 * s),
                        ("Inquiry", 80.0 * s),
                        ("Order", 60.0 * s),
                        ("Click", 40.0 * s),
                        ("Return", 20.0 * s),
                    ])
                    .into()])
                .build(ctx, view),
            (2, 0) => Chart::new()
                .title("Relationships: Graph")
                .series(vec![GraphSeries::new("Les Miserables")
                    .nodes(vec![
                        GraphNode {
                            id: "0".into(),
                            name: "Myriel".into(),
                            value: 28.6 * s,
                        },
                        GraphNode {
                            id: "1".into(),
                            name: "Napoleon".into(),
                            value: 10.0 * s,
                        },
                        GraphNode {
                            id: "2".into(),
                            name: "Mlle.Baptistine".into(),
                            value: 15.0 * s,
                        },
                        GraphNode {
                            id: "3".into(),
                            name: "Valjean".into(),
                            value: 42.0 * s,
                        },
                    ])
                    .edges(vec![
                        fission_charts::series::graph::GraphEdge {
                            source: "1".into(),
                            target: "0".into(),
                        },
                        fission_charts::series::graph::GraphEdge {
                            source: "2".into(),
                            target: "0".into(),
                        },
                        fission_charts::series::graph::GraphEdge {
                            source: "3".into(),
                            target: "0".into(),
                        },
                    ])
                    .into()])
                .build(ctx, view),
            (2, 1) => Chart::new()
                .title("Relationships: Treemap")
                .series(vec![TreemapSeries::new("Disk Usage")
                    .data(vec![
                        TreemapNode {
                            name: "System".into(),
                            value: 120.0 * s,
                            children: vec![],
                        },
                        TreemapNode {
                            name: "Users".into(),
                            value: 450.0 * s,
                            children: vec![],
                        },
                        TreemapNode {
                            name: "Applications".into(),
                            value: 310.0 * s,
                            children: vec![],
                        },
                    ])
                    .into()])
                .build(ctx, view),
            (2, 2) => Chart::new()
                .title("Relationships: Sunburst")
                .series(vec![SunburstSeries::new("Spend")
                    .data(vec![
                        TreemapNode {
                            name: "Product".into(),
                            value: 0.0,
                            children: vec![
                                TreemapNode {
                                    name: "Design".into(),
                                    value: 32.0 * s,
                                    children: vec![],
                                },
                                TreemapNode {
                                    name: "Build".into(),
                                    value: 54.0 * s,
                                    children: vec![],
                                },
                            ],
                        },
                        TreemapNode {
                            name: "Growth".into(),
                            value: 0.0,
                            children: vec![
                                TreemapNode {
                                    name: "Sales".into(),
                                    value: 42.0 * s,
                                    children: vec![],
                                },
                                TreemapNode {
                                    name: "Success".into(),
                                    value: 28.0 * s,
                                    children: vec![],
                                },
                            ],
                        },
                    ])
                    .into()])
                .build(ctx, view),
            (2, 3) => Chart::new()
                .title("Relationships: Sankey")
                .series(vec![SankeySeries::new("Energy Flow")
                    .nodes(vec![
                        GraphNode {
                            id: "a".into(),
                            name: "Solar".into(),
                            value: 0.0,
                        },
                        GraphNode {
                            id: "b".into(),
                            name: "Grid".into(),
                            value: 0.0,
                        },
                    ])
                    .edges(vec![fission_charts::series::graph::GraphEdge {
                        source: "a".into(),
                        target: "b".into(),
                    }])
                    .into()])
                .build(ctx, view),
            (2, 4) => Chart::new()
                .title("Relationships: Theme River")
                .legend(Legend::top_right())
                .series(vec![ThemeRiverSeries::new("Demand")
                    .data(vec![
                        ("2026-01", 18.0 * s, "Search"),
                        ("2026-01", 12.0 * s, "Direct"),
                        ("2026-01", 8.0 * s, "Partner"),
                        ("2026-02", 26.0 * s, "Search"),
                        ("2026-02", 14.0 * s, "Direct"),
                        ("2026-02", 14.0 * s, "Partner"),
                        ("2026-03", 20.0 * s, "Search"),
                        ("2026-03", 28.0 * s, "Direct"),
                        ("2026-03", 12.0 * s, "Partner"),
                        ("2026-04", 34.0 * s, "Search"),
                        ("2026-04", 20.0 * s, "Direct"),
                        ("2026-04", 18.0 * s, "Partner"),
                    ])
                    .into()])
                .build(ctx, view),
            (2, 5) => Chart::new()
                .title("Relationships: Parallel")
                .series(vec![ParallelSeries::new("Data")
                    .data(vec![
                        vec![12.99 * s, 100.0 * s, 82.0 * s, 90.0 * s],
                        vec![9.99 * s, 150.0 * s, 56.0 * s, 80.0 * s],
                    ])
                    .into()])
                .build(ctx, view),
            (2, 6) => Chart::new()
                .title("Geographic: Choropleth Map")
                .visual_map(VisualMap::new().min(10.0 * s).max(44.0 * s))
                .series(vec![MapSeries::new("Regions", "demo")
                    .geojson(SIMPLE_GEOJSON)
                    .data(vec![
                        ("North", 44.0 * s),
                        ("West", 18.0 * s),
                        ("East", 30.0 * s),
                    ])
                    .into()])
                .build(ctx, view),
            // --- Dynamic (category 3) ---
            (3, 0) => Chart::new()
                .title("Dynamic: Gauge")
                .series(vec![GaugeSeries::new("Speed")
                    .data(vec![("km/h", 50.0 * s)])
                    .into()])
                .build(ctx, view),
            (3, 1) => Chart::new()
                .title("Dynamic: PictorialBar")
                .x_axis(Axis::category(vec!["January", "February", "March"]))
                .y_axis(Axis::value())
                .series(vec![PictorialBarSeries::new("Spirits")
                    .data(vec![120.0 * s, 200.0 * s, 150.0 * s])
                    .into()])
                .build(ctx, view),
            (3, 2) => Chart::new()
                .title("Dynamic: EffectScatter")
                .x_axis(Axis::value())
                .y_axis(Axis::value())
                .series(vec![EffectScatterSeries::new("Effects")
                    .data(vec![
                        (10.0 * s, 8.0 * s),
                        (8.0 * s, 7.0 * s),
                        (13.0 * s, 7.5 * s),
                    ])
                    .into()])
                .build(ctx, view),
            (3, 3) => Chart::new()
                .title("Dynamic: Liquidfill")
                .series(vec![LiquidfillSeries::new("Water Level")
                    .data(vec![0.6 * s, 0.5 * s, 0.4 * s])
                    .into()])
                .build(ctx, view),
            (3, 4) => Chart::new()
                .title("Dynamic: Wordcloud")
                .series(vec![WordcloudSeries::new("Words")
                    .data(vec![
                        ("Rust", 100.0 * s),
                        ("Fission", 80.0 * s),
                        ("GPU", 60.0 * s),
                        ("Vello", 40.0 * s),
                    ])
                    .into()])
                .build(ctx, view),
            // --- 3D + Dataset (category 4) ---
            (4, 0) => Chart::new()
                .title("Dataset Engine: Encoded Line & Bar")
                .dataset(
                    Dataset::new()
                        .dimensions(vec![
                            "product".into(),
                            "2015".into(),
                            "2016".into(),
                            "2017".into(),
                        ])
                        .source(vec![
                            vec![
                                DataValue::String("Matcha Latte".into()),
                                DataValue::Number(43.3 * s),
                                DataValue::Number(85.8 * s),
                                DataValue::Number(93.7 * s),
                            ],
                            vec![
                                DataValue::String("Milk Tea".into()),
                                DataValue::Number(83.1 * s),
                                DataValue::Number(73.4 * s),
                                DataValue::Number(55.1 * s),
                            ],
                            vec![
                                DataValue::String("Cheese Cocoa".into()),
                                DataValue::Number(86.4 * s),
                                DataValue::Number(65.2 * s),
                                DataValue::Number(82.5 * s),
                            ],
                            vec![
                                DataValue::String("Walnut Brownie".into()),
                                DataValue::Number(72.4 * s),
                                DataValue::Number(53.9 * s),
                                DataValue::Number(39.1 * s),
                            ],
                        ]),
                )
                .x_axis(Axis::category(vec![
                    "Matcha Latte",
                    "Milk Tea",
                    "Cheese Cocoa",
                    "Walnut Brownie",
                ]))
                .y_axis(Axis::value())
                .legend(Legend::top_right())
                .series(vec![
                    BarSeries::new("2015")
                        .encode(Encode::new().x("product").y("2015"))
                        .color(Color {
                            r: 84,
                            g: 112,
                            b: 198,
                            a: 255,
                        })
                        .into(),
                    BarSeries::new("2016")
                        .encode(Encode::new().x("product").y("2016"))
                        .color(Color {
                            r: 145,
                            g: 204,
                            b: 117,
                            a: 255,
                        })
                        .into(),
                    LineSeries::new("2017")
                        .encode(Encode::new().x("product").y("2017"))
                        .color(Color {
                            r: 250,
                            g: 204,
                            b: 20,
                            a: 255,
                        })
                        .smooth(view.state.smooth)
                        .into(),
                ])
                .build(ctx, view),
            (4, 1) => Scene3D::new()
                .add_primitive(Primitive3D::Cube {
                    center: Point3D::new(0.0, 0.0, 0.0),
                    size: 2.0,
                    color: Color::RED,
                })
                .add_primitive(Primitive3D::Sphere {
                    center: Point3D::new(3.0, 3.0, 3.0),
                    radius: 1.5,
                    color: Color::BLUE,
                })
                .build(ctx, view),
            _ => Container::new(
                Text::new("Select a chart from the gallery")
                    .color(Color {
                        r: 150,
                        g: 150,
                        b: 150,
                        a: 255,
                    })
                    .into_node(),
            )
            .into_node(),
        };

        let controls = Row {
            children: vec![
                Text::new("Smooth Lines:").color(Color::WHITE).into_node(),
                fission_widgets::Switch {
                    checked: view.state.smooth,
                    on_toggle: Some(ActionEnvelope {
                        id: toggle_smooth_id,
                        payload: serde_json::to_vec(&ToggleSmooth(false)).unwrap(),
                    }),
                    ..Default::default()
                }
                .into_node(),
                fission_widgets::Spacer {
                    width: Some(32.0),
                    ..Default::default()
                }
                .into_node(),
                Text::new("Data Scale:").color(Color::WHITE).into_node(),
                fission_widgets::Slider {
                    value: view.state.data_scale,
                    min: 0.1,
                    max: 2.0,
                    on_change: Some(ActionEnvelope {
                        id: update_scale_id,
                        payload: vec![],
                    }), // Payload is overwritten by SliderController
                    ..Default::default()
                }
                .into_node(),
            ],
            gap: Some(12.0),
            align_items: fission_core::op::AlignItems::Center,
            ..Default::default()
        }
        .into_node();

        let content = Container::new(
            Column {
                children: vec![
                    Row {
                        children: vec![
                            Text::new(if view.state.selected_category == SHOWCASE_CATEGORY {
                                "Chart Showcase"
                            } else {
                                "Interactive Demo"
                            })
                            .size(24.0)
                            .color(Color::WHITE)
                            .into_node(),
                            fission_widgets::Spacer {
                                flex_grow: 1.0,
                                ..Default::default()
                            }
                            .into_node(),
                        ],
                        ..Default::default()
                    }
                    .into_node(),
                    fission_widgets::Spacer {
                        height: Some(24.0),
                        ..Default::default()
                    }
                    .into_node(),
                    chart_node,
                    fission_widgets::Spacer {
                        height: Some(24.0),
                        ..Default::default()
                    }
                    .into_node(),
                    controls,
                ],
                flex_grow: 1.0,
                ..Default::default()
            }
            .into_node(),
        )
        .padding_all(32.0)
        .bg(Color {
            r: 20,
            g: 20,
            b: 20,
            a: 255,
        })
        .flex_grow(1.0)
        .into_node();

        Row {
            children: vec![sidebar, content],
            flex_grow: 1.0,
            ..Default::default()
        }
        .into_node()
    }
}

fn build_showcase(
    ctx: &mut BuildCtx<GalleryState>,
    view: &View<GalleryState>,
    content_width: f32,
    s: f32,
) -> Node {
    let two_columns = content_width >= 900.0;
    let gap = 18.0;
    let card_width = if two_columns {
        ((content_width - gap) / 2.0).max(320.0)
    } else {
        content_width.max(340.0)
    };
    let metric_width = if content_width >= 960.0 {
        ((content_width - gap * 2.0) / 3.0).max(220.0)
    } else {
        content_width.max(340.0)
    };

    let metric_nodes = vec![
        metric_card(
            "Available now",
            "21 chart surfaces",
            "Core cartesian, radial, statistical, relationship, map, and status charts render through the native chart lowerer.",
            teal(),
            metric_width,
        ),
        metric_card(
            "Data model",
            "Dataset + encode",
            "Series can read direct vectors or named dataset dimensions, so app code scales beyond one-off arrays.",
            blue(),
            metric_width,
        ),
        metric_card(
            "Next",
            "WASM gallery",
            "This desktop gallery is the source surface for the future browser demo and editable chart examples.",
            amber(),
            metric_width,
        ),
    ];

    let mut children = vec![
        chart_row(
            vec![
                chart_card(
                    ctx,
                    view,
                    "Revenue composition",
                    "Stacked area, smooth interpolation, legend, and data zoom presentation.",
                    Chart::new()
                        .title("Revenue by channel")
                        .x_axis(Axis::category(vec![
                            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug",
                        ]))
                        .y_axis(Axis::value())
                        .legend(Legend::top_right())
                        .data_zoom(DataZoom::new().start_percent(14.0).end_percent(92.0))
                        .series(vec![
                            LineSeries::new("Product")
                                .stack("total")
                                .area_style(rgb(20, 184, 166).with_alpha(112))
                                .data(vec![
                                    120.0 * s,
                                    132.0 * s,
                                    164.0 * s,
                                    156.0 * s,
                                    190.0 * s,
                                    230.0 * s,
                                    268.0 * s,
                                    302.0 * s,
                                ])
                                .color(teal())
                                .smooth(view.state.smooth)
                                .into(),
                            LineSeries::new("Services")
                                .stack("total")
                                .area_style(rgb(37, 99, 235).with_alpha(96))
                                .data(vec![
                                    80.0 * s,
                                    96.0 * s,
                                    120.0 * s,
                                    132.0 * s,
                                    138.0 * s,
                                    160.0 * s,
                                    172.0 * s,
                                    188.0 * s,
                                ])
                                .color(blue())
                                .smooth(view.state.smooth)
                                .into(),
                        ]),
                    card_width,
                    292.0,
                    teal(),
                ),
                chart_card(
                    ctx,
                    view,
                    "Regional demand",
                    "GeoJSON-backed choropleth map, visual map coloring, and region labels.",
                    Chart::new()
                        .title("Regional demand")
                        .visual_map(VisualMap::new().min(10.0 * s).max(44.0 * s))
                        .series(vec![MapSeries::new("Regions", "demo")
                            .geojson(SIMPLE_GEOJSON)
                            .data(vec![
                                ("North", 44.0 * s),
                                ("West", 18.0 * s),
                                ("East", 30.0 * s),
                            ])
                            .into()]),
                    card_width,
                    292.0,
                    blue(),
                ),
            ],
            two_columns,
        ),
        chart_row(
            vec![
                chart_card(
                    ctx,
                    view,
                    "Product hierarchy",
                    "Sunburst layout for nested product and growth categories.",
                    Chart::new()
                        .title("Spend allocation")
                        .series(vec![SunburstSeries::new("Spend")
                            .data(vec![
                                TreemapNode {
                                    name: "Product".into(),
                                    value: 0.0,
                                    children: vec![
                                        TreemapNode {
                                            name: "Design".into(),
                                            value: 32.0 * s,
                                            children: vec![],
                                        },
                                        TreemapNode {
                                            name: "Build".into(),
                                            value: 54.0 * s,
                                            children: vec![],
                                        },
                                    ],
                                },
                                TreemapNode {
                                    name: "Growth".into(),
                                    value: 0.0,
                                    children: vec![
                                        TreemapNode {
                                            name: "Sales".into(),
                                            value: 42.0 * s,
                                            children: vec![],
                                        },
                                        TreemapNode {
                                            name: "Success".into(),
                                            value: 28.0 * s,
                                            children: vec![],
                                        },
                                    ],
                                },
                            ])
                            .into()]),
                    card_width,
                    292.0,
                    amber(),
                ),
                chart_card(
                    ctx,
                    view,
                    "Traffic stream",
                    "Theme river bands show changing mix across ordered time buckets.",
                    Chart::new()
                        .title("Traffic mix")
                        .legend(Legend::top_right())
                        .series(vec![ThemeRiverSeries::new("Traffic")
                            .data(vec![
                                ("Jan", 18.0 * s, "Search"),
                                ("Jan", 12.0 * s, "Direct"),
                                ("Jan", 8.0 * s, "Partner"),
                                ("Feb", 26.0 * s, "Search"),
                                ("Feb", 14.0 * s, "Direct"),
                                ("Feb", 14.0 * s, "Partner"),
                                ("Mar", 20.0 * s, "Search"),
                                ("Mar", 28.0 * s, "Direct"),
                                ("Mar", 12.0 * s, "Partner"),
                                ("Apr", 34.0 * s, "Search"),
                                ("Apr", 20.0 * s, "Direct"),
                                ("Apr", 18.0 * s, "Partner"),
                            ])
                            .into()]),
                    card_width,
                    292.0,
                    rgb(168, 85, 247),
                ),
            ],
            two_columns,
        ),
        chart_row(
            vec![
                chart_card(
                    ctx,
                    view,
                    "Conversion flow",
                    "Sankey bands for source-to-target movement in a product funnel.",
                    Chart::new()
                        .title("Energy flow")
                        .series(vec![SankeySeries::new("Energy")
                            .nodes(vec![
                                GraphNode {
                                    id: "a".into(),
                                    name: "Solar".into(),
                                    value: 0.0,
                                },
                                GraphNode {
                                    id: "b".into(),
                                    name: "Grid".into(),
                                    value: 0.0,
                                },
                                GraphNode {
                                    id: "c".into(),
                                    name: "Home".into(),
                                    value: 0.0,
                                },
                            ])
                            .edges(vec![
                                fission_charts::series::graph::GraphEdge {
                                    source: "a".into(),
                                    target: "c".into(),
                                },
                                fission_charts::series::graph::GraphEdge {
                                    source: "b".into(),
                                    target: "c".into(),
                                },
                            ])
                            .into()]),
                    card_width,
                    292.0,
                    rgb(244, 114, 182),
                ),
                chart_card(
                    ctx,
                    view,
                    "Operations heat",
                    "Heatmap with visual-map scale for dense categorical intensity.",
                    Chart::new()
                        .title("Support load")
                        .x_axis(Axis::category(vec!["12a", "1a", "2a", "3a", "4a", "5a"]))
                        .y_axis(Axis::category(vec!["Sat", "Fri", "Thu", "Wed"]))
                        .visual_map(VisualMap::new().min(0.0).max(10.0 * s))
                        .series(vec![HeatmapSeries::new("Load")
                            .data(vec![
                                (0, 0, 5.0 * s),
                                (1, 0, 8.0 * s),
                                (2, 0, 3.0 * s),
                                (3, 0, 7.0 * s),
                                (4, 0, 2.0 * s),
                                (5, 0, 10.0 * s),
                                (0, 1, 2.0 * s),
                                (1, 1, 4.0 * s),
                                (2, 1, 9.0 * s),
                                (3, 1, 5.0 * s),
                                (4, 1, 1.0 * s),
                                (5, 1, 6.0 * s),
                                (0, 2, 6.0 * s),
                                (1, 2, 3.0 * s),
                                (2, 2, 7.0 * s),
                                (3, 2, 8.0 * s),
                                (4, 2, 4.0 * s),
                                (5, 2, 2.0 * s),
                                (0, 3, 1.0 * s),
                                (1, 3, 5.0 * s),
                                (2, 3, 4.0 * s),
                                (3, 3, 9.0 * s),
                                (4, 3, 7.0 * s),
                                (5, 3, 3.0 * s),
                            ])
                            .into()]),
                    card_width,
                    292.0,
                    rgb(96, 165, 250),
                ),
            ],
            two_columns,
        ),
        chart_row(metric_nodes, content_width >= 960.0),
    ];

    children.push(
        Container::new(
            Text::new("Use the sidebar to inspect the single-chart examples. The overview intentionally renders several chart families together so visual regressions are obvious.")
                .size(13.0)
                .color(rgb(148, 163, 184))
                .into_node(),
        )
        .padding_all(14.0)
        .border_radius(16.0)
        .bg(rgb(15, 23, 42))
        .border(rgb(51, 65, 85), 1.0)
        .into_node(),
    );

    Scroll {
        direction: fission_core::FlexDirection::Column,
        child: Some(Box::new(
            Column {
                children,
                gap: Some(18.0),
                ..Default::default()
            }
            .into_node(),
        )),
        show_scrollbar: true,
        flex_grow: 1.0,
        ..Default::default()
    }
    .into_node()
}

fn chart_card(
    ctx: &mut BuildCtx<GalleryState>,
    view: &View<GalleryState>,
    title: &str,
    subtitle: &str,
    chart: Chart,
    width: f32,
    chart_height: f32,
    accent: Color,
) -> Node {
    Container::new(
        Column {
            children: vec![
                Row {
                    children: vec![
                        Container::new(Text::new("").into_node())
                            .width(8.0)
                            .height(32.0)
                            .border_radius(8.0)
                            .bg(accent)
                            .into_node(),
                        Column {
                            children: vec![
                                Text::new(title).size(18.0).color(Color::WHITE).into_node(),
                                Text::new(subtitle)
                                    .size(12.0)
                                    .color(rgb(148, 163, 184))
                                    .into_node(),
                            ],
                            gap: Some(4.0),
                            ..Default::default()
                        }
                        .into_node(),
                    ],
                    gap: Some(10.0),
                    ..Default::default()
                }
                .into_node(),
                chart
                    .width((width - 32.0).max(260.0))
                    .height(chart_height)
                    .build(ctx, view),
            ],
            gap: Some(12.0),
            ..Default::default()
        }
        .into_node(),
    )
    .width(width)
    .padding_all(16.0)
    .border_radius(24.0)
    .bg(rgb(11, 18, 32))
    .border(rgb(51, 65, 85), 1.0)
    .into_node()
}

fn metric_card(title: &str, value: &str, detail: &str, accent: Color, width: f32) -> Node {
    Container::new(
        Column {
            children: vec![
                Text::new(title).size(12.0).color(accent).into_node(),
                Text::new(value).size(22.0).color(Color::WHITE).into_node(),
                Text::new(detail)
                    .size(12.0)
                    .color(rgb(148, 163, 184))
                    .into_node(),
            ],
            gap: Some(7.0),
            ..Default::default()
        }
        .into_node(),
    )
    .width(width)
    .padding_all(16.0)
    .border_radius(18.0)
    .bg(rgb(11, 18, 32))
    .border(rgb(51, 65, 85), 1.0)
    .into_node()
}

fn chart_row(children: Vec<Node>, row: bool) -> Node {
    if row {
        Row {
            children,
            gap: Some(18.0),
            ..Default::default()
        }
        .into_node()
    } else {
        Column {
            children,
            gap: Some(18.0),
            ..Default::default()
        }
        .into_node()
    }
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color { r, g, b, a: 255 }
}

fn teal() -> Color {
    rgb(20, 184, 166)
}

fn blue() -> Color {
    rgb(37, 99, 235)
}

fn amber() -> Color {
    rgb(245, 158, 11)
}

fn main() -> anyhow::Result<()> {
    let app = DesktopApp::new(GalleryApp)
        .with_title("Fission Chart Gallery")
        .with_sync_env(|_state: &GalleryState, env: &mut fission_core::Env| {
            env.theme = fission_theme::Theme::dark();
        });

    app.run()
}
