# Fission Charts Plan: Achieving ECharts Parity

This document outlines the roadmap to implement a comprehensive charting library (`fission-charts`) that rivals Apache ECharts in capability, while adhering to Fission's deterministic, "Zero-Closure" architecture.

## 1. Core Primitives Requirements

To support the visual fidelity of ECharts, `fission-ir` and the Renderer must be upgraded.

### 1.1 Gradients & Patterns (Critical)
The current `Fill` struct (`color: Color`) is insufficient.
**New Definition:**
```rust
pub enum FillKind {
    Solid(Color),
    LinearGradient {
        start: (f32, f32), // Relative or Absolute?
        end: (f32, f32),
        stops: Vec<(f32, Color)>, // Offset, Color
    },
    RadialGradient {
        center: (f32, f32),
        radius: f32,
        stops: Vec<(f32, Color)>,
    },
    // Pattern(ImageId, RepeatMode), // Future
}
```

### 1.2 Advanced Stroke (Critical)
Charts use dashed lines for grids, projections, and emphasis.
**New Definition:**
```rust
pub struct Stroke {
    pub fill: FillKind, // Stroke can have gradient!
    pub width: f32,
    pub dash_array: Option<Vec<f32>>, // e.g. [5.0, 5.0]
    pub line_cap: LineCap, // Butt, Round, Square
    pub line_join: LineJoin, // Miter, Round, Bevel
}
```

### 1.3 Clipping (High Priority)
To animate line charts (revealing from left) or handle zoom/pan, we need precise clipping.
**New Op:** `Op::Paint(PaintOp::Clip { path: String, child: NodeId })`. Or `LayoutOp::ClipPath`.

---

## 2. API Design Principles (`fission-charts`)

We will not blindly copy ECharts' JSON-bag config. We will use a strongly-typed, composable Rust API that feels like "ECharts but type-safe".

**Series-Based Model:**
```rust
Chart {
    title: Some("Sales".into()),
    tooltip: Tooltip::axis_trigger(),
    legend: Legend::top_right(),
    grid: Grid::default(),
    x_axis: Axis::category(vec!["Mon", "Tue", "Wed"]),
    y_axis: Axis::value(),
    series: vec![
        Series::Line(LineSeries {
            name: "Revenue".into(),
            data: vec![120.0, 200.0, 150.0],
            smooth: true,
            area_style: Some(AreaStyle::gradient(Color::BLUE, Color::TRANSPARENT)),
        }),
        Series::Bar(BarSeries {
            name: "Cost".into(),
            data: vec![80.0, 100.0, 90.0],
        })
    ],
    // Animations
    animate: true,
}
```

**Developer Ergonomics:**
- **`Vec<Series>`**: Polymorphic series handling.
- **Builders**: `LineSeries::new().smooth().color(...)`.
- **Determinism**: The chart logic compiles this config into a list of `DrawPath` / `DrawRect` / `DrawText` ops deterministically. No generic render callbacks inside the tree (unless using custom series).

---

## 3. Chart Types & Implementation Strategy

We aim for the "Complete Set". Here is the breakdown:

### Group A: Cartesian (X/Y)
*Foundation:* `Grid` (draws axes, tick marks, split lines) + Coordinate System logic.

1.  **Line Chart**:
    *   **Logic:** Map `(x, y)` to points. Generate SVG path (`M... L...`).
    *   **Features:** Smoothing (Bezier), Area fill (close path to axis), Stacked lines.
2.  **Bar Chart**:
    *   **Logic:** `DrawRect` for each datum.
    *   **Features:** Stacked bars (offset y), Grouped bars (offset x), Rounded corners (Bar race).
3.  **Scatter Chart**:
    *   **Logic:** `DrawPath` (Symbol) at `(x, y)`.
    *   **Features:** Bubble size encoding, color encoding.
4.  **Candlestick (K-Line)**:
    *   **Logic:** `DrawRect` (Body) + `DrawPath` (Wick line).
5.  **Boxplot**:
    *   **Logic:** `DrawRect` (Box) + Lines (Whiskers/Median).
6.  **Heatmap (Cartesian)**:
    *   **Logic:** Grid of `DrawRect`. Color mapping.
7.  **PictorialBar**:
    *   **Logic:** Bar chart where the bar shape is an SVG path (e.g. bottles, mountains).
    *   **Impl:** `DrawSvg` (stretched) or `DrawPath`.

### Group B: Radial / Polar
*Foundation:* `Polar` coordinate mapper (`angle`, `radius` -> `x`, `y`).

8.  **Pie Chart**:
    *   **Logic:** Calculate start/end angles. `DrawPath` (Arc segment).
    *   **Features:** Donut (inner radius > 0), Rose (varying radius).
9.  **Radar Chart**:
    *   **Logic:** Multiple axes radiating from center. Draw polygons connecting value on each axis.
10. **Gauge**:
    *   **Logic:** Arc track + Needle (rotated `DrawPath` or `DrawRect`).
11. **Sunburst**:
    *   **Logic:** Multi-level Pie.
12. **Polar Bar/Line**:
    *   **Logic:** Bar/Line using Polar coords (spiral bars).

### Group C: Graph & Relationships
*Foundation:* Force-directed layout engine (or static layout).

13. **Graph**:
    *   **Logic:** Nodes (Circles/Images) + Edges (Lines/Curves).
14. **Sankey**:
    *   **Logic:** Flow nodes + Bezier ribbons connecting them (varying stroke width or filled path).
15. **Tree**:
    *   **Logic:** Reingold-Tilford layout. Edges + Nodes.
16. **Treemap**:
    *   **Logic:** Recursive rectangle subdivision (Squarified).
17. **Funnel**:
    *   **Logic:** Trapezoids stacked vertically.

### Group D: Map & Geo
*Foundation:* GeoJSON parser + Projection.

18. **Map**:
    *   **Logic:** Project Lat/Lon -> X/Y. Draw paths.
    *   **Impl:** Need a lightweight GeoJSON -> Path converter.
19. **Lines (Geo)**:
    *   **Logic:** Curves connecting geo points (flight paths).

### Group E: Specialized
20. **Calendar**:
    *   **Logic:** Heatmap on a calendar grid.
21. **ThemeRiver**:
    *   **Logic:** Stacked area chart on a time axis, but centered/flowing.
22. **Parallel Coordinates**:
    *   **Logic:** Multiple Y axes. Lines connecting data across axes.

---

## 4. Interaction & Performance

### 4.1 Tooltips
*   **Problem:** Precise hit testing.
*   **Solution:**
    *   *Voronoi Tessellation* (for scatter/line) to find nearest point.
    *   *R-Tree* or *QuadTree* for fast spatial query.
    *   **Fission Impl:** The Chart widget, during `build`, calculates these spatial structures. On `PointerMove`, it queries the structure and renders a **Tooltip Overlay**.

### 4.2 Zoom & Pan
*   **Logic:** Update the `ViewWindow` (visible range of X/Y axis).
*   **Impl:** `GestureDetector` captures drag/wheel. Reducer updates `chart_view_state`. Widget re-builds with new scale.
*   **Perf:** Deterministic rebuild of paths is fast for < 10k points. For > 100k, we need downsampling (LTTB algorithm) in the `model` layer.

---

## 5. Implementation Roadmap

### Phase 1: Core Upgrades
1.  Add `FillKind` (Gradients) to `fission-ir`.
2.  Add `Stroke` properties (Dash, Cap, Join).
3.  Implement `ClipPath`.

### Phase 2: The Charting Engine (`fission-charts`)
1.  **`CoordSystem`**: Traits for mapping Data -> Pixel.
2.  **`Axis`**: Tick generation (Nice numbers algorithm).
3.  **`Legend`**: Layout logic.

### Phase 3: Priority Charts (Business Standard)
1.  Line
2.  Bar
3.  Pie
4.  Area (requires gradients)

### Phase 4: Advanced
1.  Scatter
2.  Candlestick
3.  Radar

### Phase 5: The Rest
Remaining ECharts parity.

---

## 6. Example API Usage

```rust
let chart = Chart::new()
    .x_axis(Axis::category(months))
    .y_axis(Axis::value().formatter("$val"))
    .series(vec![
        LineSeries::new("Sales")
            .data(sales_data)
            .color(Color::BLUE)
            .smooth(0.5)
            .into()
    ])
    .on_click(|params| println!("Clicked {:?}", params));

// Render
chart.build(ctx, view)
```
