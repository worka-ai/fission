pub(crate) struct ChartCategory {
    pub name: &'static str,
    pub charts: &'static [&'static str],
}

pub(crate) const CATEGORIES: &[ChartCategory] = &[
    ChartCategory {
        name: "Foundational",
        charts: &[
            "Line & Bar",
            "Stacked Area",
            "Step Line",
            "Donut Pie",
            "Scatter Visual",
        ],
    },
    ChartCategory {
        name: "Cartesian Variants",
        charts: &[
            "Horizontal Bar",
            "Rounded Background Bar",
            "Negative Bar",
            "Bubble Scatter",
            "Large Line",
        ],
    },
    ChartCategory {
        name: "Statistical",
        charts: &["Boxplot", "Candlestick", "Heatmap", "Radar", "Funnel"],
    },
    ChartCategory {
        name: "Relationships + Geo",
        charts: &[
            "Graph",
            "Treemap",
            "Sunburst",
            "Sankey",
            "Theme River",
            "Parallel",
            "Choropleth Map",
            "Tree",
            "Radial Tree",
            "Lines",
        ],
    },
    ChartCategory {
        name: "Dynamic",
        charts: &[
            "Gauge",
            "PictorialBar",
            "EffectScatter",
            "Liquidfill",
            "Wordcloud",
        ],
    },
    ChartCategory {
        name: "3D + Dataset",
        charts: &[
            "Dataset Demo",
            "3D Scene",
            "3D Bar",
            "3D Scatter",
            "Surface Mesh",
            "3D Line",
            "Point Cloud",
            "Globe",
            "3D Graph",
            "Terrain",
        ],
    },
    ChartCategory {
        name: "Components",
        charts: &[
            "Markers",
            "Data Zoom",
            "Axis Tooltip",
            "Timeline",
            "Toolbox",
            "Brush Select",
            "Graphic Overlay",
        ],
    },
    ChartCategory {
        name: "Coordinates",
        charts: &["Polar Bar", "Polar Line", "Calendar Heatmap", "Single Axis"],
    },
];
