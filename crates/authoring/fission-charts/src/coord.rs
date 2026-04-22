pub trait CoordSystem {
    fn map(&self, x: f32, y: f32) -> (f32, f32);
}
