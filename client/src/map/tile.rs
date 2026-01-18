//! Tile coordinate system and conversions
//! Uses Web Mercator projection (EPSG:3857) compatible with OSM

use std::f64::consts::PI;

/// Unique identifier for a map tile
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct TileId {
    pub x: u32,
    pub y: u32,
    pub z: u8,
}

impl TileId {
    pub fn new(x: u32, y: u32, z: u8) -> Self {
        Self { x, y, z }
    }

    /// Build OSM tile URL
    pub fn to_osm_url(&self) -> String {
        format!(
            "https://tile.openstreetmap.org/{}/{}/{}.png",
            self.z, self.x, self.y
        )
    }
}

/// Convert longitude/latitude to fractional tile coordinates (for sub-tile positioning)
pub fn lon_lat_to_tile_f64(lon: f64, lat: f64, zoom: u8) -> (f64, f64) {
    let n = (1_u64 << zoom) as f64;

    let x = (lon + 180.0) / 360.0 * n;

    let lat_rad = lat.to_radians();
    let y = (1.0 - lat_rad.tan().asinh() / PI) / 2.0 * n;

    (x, y)
}

/// Wrap X coordinate for infinite horizontal scrolling
pub fn wrap_tile_x(x: i32, zoom: u8) -> u32 {
    let max_tiles = 1_i32 << zoom;
    ((x % max_tiles) + max_tiles) as u32 % max_tiles as u32
}

/// Check if Y coordinate is valid (no wrapping for latitude)
pub fn is_valid_tile_y(y: i32, zoom: u8) -> bool {
    let max_tiles = 1_i32 << zoom;
    y >= 0 && y < max_tiles
}

/// Normalize longitude to [-180, 180]
pub fn normalize_longitude(lon: f64) -> f64 {
    let mut l = lon;
    while l < -180.0 {
        l += 360.0;
    }
    while l > 180.0 {
        l -= 360.0;
    }
    l
}

/// Clamp latitude to valid Mercator range
pub fn clamp_latitude(lat: f64) -> f64 {
    lat.clamp(-85.05112878, 85.05112878)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_tile_x() {
        // At zoom 2, max tiles = 4 (0-3)
        assert_eq!(wrap_tile_x(4, 2), 0); // Wrap around
        assert_eq!(wrap_tile_x(-1, 2), 3); // Wrap negative
        assert_eq!(wrap_tile_x(2, 2), 2); // Normal
    }

    #[test]
    fn test_normalize_longitude() {
        assert!((normalize_longitude(190.0) - (-170.0)).abs() < 0.001);
        assert!((normalize_longitude(-190.0) - 170.0).abs() < 0.001);
    }
}
