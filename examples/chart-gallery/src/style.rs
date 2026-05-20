use fission::core::op::Color;

pub(crate) fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color { r, g, b, a: 255 }
}

pub(crate) fn teal() -> Color {
    rgb(20, 184, 166)
}

pub(crate) fn blue() -> Color {
    rgb(37, 99, 235)
}

pub(crate) fn amber() -> Color {
    rgb(245, 158, 11)
}
