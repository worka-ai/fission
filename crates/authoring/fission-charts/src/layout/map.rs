use crate::series::map::MapSeries;

pub struct MapLayout;

impl MapLayout {
    pub fn compute_geojson(
        _series: &MapSeries,
        _width: f32,
        _height: f32,
    ) -> Vec<(String, String)> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_layout_does_not_emit_regions_without_geojson() {
        let map = MapSeries::new("World", "world");
        let paths = MapLayout::compute_geojson(&map, 800.0, 600.0);
        assert!(paths.is_empty());
    }
}
