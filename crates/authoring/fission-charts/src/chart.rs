use fission_core::op::Color;
use fission_core::{BuildCtx, View, Widget};
use fission_core::ui::{Node, Container, CustomNode, ZStack, Positioned};
use fission_ir::op::{LayoutUnit, PaintOp, LayoutOp, Stroke, Fill, LineCap, LineJoin};
use crate::{Axis, Grid, Legend, Series, Tooltip};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    pub width: f32,
    pub height: f32,
    pub title: Option<String>,
    pub tooltip: Option<Tooltip>,
    pub legend: Option<Legend>,
    pub grid: Option<Grid>,
    pub x_axis: Option<Axis>,
    pub y_axis: Option<Axis>,
    pub series: Vec<Series>,
    pub animate: bool,
}

impl Chart {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            title: None,
            tooltip: None,
            legend: None,
            grid: None,
            x_axis: None,
            y_axis: None,
            series: Vec::new(),
            animate: false,
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.x_axis = Some(axis);
        self
    }

    pub fn y_axis(mut self, axis: Axis) -> Self {
        self.y_axis = Some(axis);
        self
    }

    pub fn series(mut self, series: Vec<Series>) -> Self {
        self.series = series;
        self
    }

    pub fn grid(mut self, grid: Grid) -> Self {
        self.grid = Some(grid);
        self
    }

    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }
}

impl<S: fission_core::AppState> Widget<S> for Chart {
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> Node {
        Container::new(
            Node::Custom(CustomNode {
                debug_tag: "fission_charts::Chart".into(),
                lowerer: Some(std::sync::Arc::new(ChartLowerer {
                    chart: self.clone(),
                })),
            })
        ).width(self.width).height(self.height).into_node()
    }
}

#[derive(Debug)]
pub struct ChartLowerer {
    pub chart: Chart,
}

impl fission_core::ui::traits::LowerDyn for ChartLowerer {
    fn lower_dyn(&self, cx: &mut fission_core::lowering::LoweringContext) -> fission_ir::NodeId {
        let node_id = cx.next_node_id();
        let mut builder = fission_core::lowering::NodeBuilder::new(node_id, fission_ir::Op::Layout(fission_ir::op::LayoutOp::ZStack));
        
        let w = self.chart.width;
        let h = self.chart.height;
        let pad = 40.0; // Simple margin
        let inner_w = w - pad * 2.0;
        let inner_h = h - pad * 2.0;

        // 1. Draw Grid (Background)
        let grid_id = cx.next_node_id();
        let mut grid_b = fission_core::lowering::NodeBuilder::new(grid_id, fission_ir::Op::Paint(PaintOp::DrawRect {
            fill: Some(Fill::Solid(Color { r: 245, g: 245, b: 245, a: 255 })),
            stroke: Some(Stroke {
                fill: Fill::Solid(Color { r: 200, g: 200, b: 200, a: 255 }),
                width: 1.0,
                dash_array: None,
                line_cap: LineCap::Butt,
                line_join: LineJoin::Miter,
            }),
            corner_radius: 0.0,
            shadow: None,
        }));
        
        let grid_wrap_id = cx.next_node_id();
        let mut grid_wrap = fission_core::lowering::NodeBuilder::new(grid_wrap_id, fission_ir::Op::Layout(LayoutOp::Positioned {
            left: Some(pad),
            top: Some(pad),
            width: Some(inner_w),
            height: Some(inner_h),
            right: None,
            bottom: None,
        }));
        grid_wrap.add_child(grid_b.build(cx));
        builder.add_child(grid_wrap.build(cx));

        // 2. Draw Series
        for series in &self.chart.series {
            match series {
                Series::Line(line) => {
                    if line.data.is_empty() { continue; }
                    let mut path = format!("M {} {}", pad, pad + inner_h - (line.data[0] / 100.0) * inner_h);
                    let step = inner_w / (line.data.len().max(2) - 1) as f32;
                    for (i, val) in line.data.iter().enumerate().skip(1) {
                        let x = pad + (i as f32) * step;
                        let y = pad + inner_h - (val / 100.0) * inner_h; // Assuming 0-100 scale for mock
                        path.push_str(&format!(" L {} {}", x, y));
                    }
                    
                    let path_id = cx.next_node_id();
                    let mut path_b = fission_core::lowering::NodeBuilder::new(path_id, fission_ir::Op::Paint(PaintOp::DrawPath {
                        path,
                        fill: None,
                        stroke: Some(Stroke {
                            fill: Fill::Solid(line.color),
                            width: 2.0,
                            dash_array: None,
                            line_cap: LineCap::Round,
                            line_join: LineJoin::Round,
                        }),
                    }));
                    builder.add_child(path_b.build(cx));
                }
                Series::Bar(bar) => {
                    let step = inner_w / (bar.data.len().max(1) as f32);
                    let bar_w = step * 0.6;
                    for (i, val) in bar.data.iter().enumerate() {
                        let x = pad + (i as f32) * step + (step - bar_w) / 2.0;
                        let bar_h = (val / 100.0) * inner_h;
                        let y = pad + inner_h - bar_h;
                        
                        let rect_id = cx.next_node_id();
                        let mut rect_b = fission_core::lowering::NodeBuilder::new(rect_id, fission_ir::Op::Paint(PaintOp::DrawRect {
                            fill: Some(Fill::Solid(bar.color)),
                            stroke: None,
                            corner_radius: 2.0,
                            shadow: None,
                        }));
                        
                        let pos_id = cx.next_node_id();
                        let mut pos_b = fission_core::lowering::NodeBuilder::new(pos_id, fission_ir::Op::Layout(LayoutOp::Positioned {
                            left: Some(x),
                            top: Some(y),
                            width: Some(bar_w),
                            height: Some(bar_h),
                            right: None,
                            bottom: None,
                        }));
                        pos_b.add_child(rect_b.build(cx));
                        builder.add_child(pos_b.build(cx));
                    }
                }
                Series::Scatter(scatter) => {
                    for (dx, dy) in &scatter.data {
                        let x = pad + (dx / 100.0) * inner_w;
                        let y = pad + inner_h - (dy / 100.0) * inner_h;
                        
                        let r = 4.0;
                        let rect_id = cx.next_node_id();
                        let mut rect_b = fission_core::lowering::NodeBuilder::new(rect_id, fission_ir::Op::Paint(PaintOp::DrawRect {
                            fill: Some(Fill::Solid(scatter.color)),
                            stroke: None,
                            corner_radius: r,
                            shadow: None,
                        }));
                        
                        let pos_id = cx.next_node_id();
                        let mut pos_b = fission_core::lowering::NodeBuilder::new(pos_id, fission_ir::Op::Layout(LayoutOp::Positioned {
                            left: Some(x - r),
                            top: Some(y - r),
                            width: Some(r * 2.0),
                            height: Some(r * 2.0),
                            right: None,
                            bottom: None,
                        }));
                        pos_b.add_child(rect_b.build(cx));
                        builder.add_child(pos_b.build(cx));
                    }
                }
                _ => {}
            }
        }
        
        builder.build(cx)
    }
}
