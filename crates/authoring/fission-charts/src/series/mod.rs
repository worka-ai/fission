pub mod line;
pub mod bar;
pub mod scatter;
pub mod pie;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Series {
    Line(line::LineSeries),
    Bar(bar::BarSeries),
    Scatter(scatter::ScatterSeries),
    Pie(pie::PieSeries),
}
