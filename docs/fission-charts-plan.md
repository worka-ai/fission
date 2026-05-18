# Fission Charts Product Plan

Fission Charts should be a first-class Rust charting system for production Fission apps. The goal is a deep, polished, typed charting library with strong rendering quality, predictable behavior, and examples that demonstrate real application use.

## 1. Architecture

Charts are rendered through a deterministic pipeline:

```text
Chart API -> ChartModel -> layout models -> display model -> Fission IR
```

The public API stays strongly typed and Rust-native. The renderer must not use string-named callbacks or pretend that unsupported chart types are implemented. If a series needs runtime behavior, that behavior should be expressed as typed Fission actions, reducers, and state.

## 2. Current Foundation

The first production slice is focused on real rendering for:

- cartesian line, area, step line, bar, scatter, boxplot, candlestick, heatmap, pictorial bar, and effect scatter;
- radial pie/donut, radar, funnel, gauge, and liquid fill;
- relationship charts through graph, treemap, sankey, and parallel coordinates;
- word cloud layout;
- dataset + encode mapping for line and bar series;
- visual map, legend, data zoom presentation, and unsupported-series diagnostics.

Map, sunburst, theme-river, and string-callback custom series remain intentionally unrendered until they have real production implementations.

## 3. Next Implementation Milestones

1. Finish component behavior: tooltip state, axis pointer, legend toggling, data zoom filtering, brush selection, and click/hover actions.
2. Finish data modeling: row/column dataset layout, typed dimensions, transforms, derived fields, null handling, and multi-axis encode mappings.
3. Finish chart depth: multiple axes, log/time scales, labels, label collision, rich labels, mark line/point/area, and responsive layout.
4. Finish advanced series: real GeoJSON map/geo lines, sunburst hierarchy layout, theme river stream layout, tree/chord/calendar, and richer custom series via typed Fission render hooks.
5. Add performance paths: retained geometry caches, decimation for large line series, spatial indexes for scatter hit testing, and stable animation state.

## 4. Gallery And Tests

The existing `examples/chart-gallery` is the canonical showcase. Every gallery item should be backed by real rendering, tests, and clear app-facing code. Unsupported or experimental chart types should not appear in the gallery as if they are production-ready.

Required verification for each supported feature:

- unit tests for data mapping and layout;
- lowering tests that assert concrete Fission IR output;
- live gallery smoke tests;
- screenshot review for visual regressions;
- interaction tests once tooltip, legend, zoom, and brush behavior are wired.
