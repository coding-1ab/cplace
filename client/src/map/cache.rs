//! LRU tile cache for GPU textures with zoom-level separation

use std::collections::HashMap;
use std::sync::Arc;

use super::tile::TileId;

/// Tile type for different use cases
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TileType {
    /// Standard OSM map tile (typically 256x256)
    #[default]
    MapTile,
    /// Pixel art overlay (variable size)
    PixelArt,
    /// High resolution tile (512x512)
    HighRes,
    /// Custom user-defined tile
    Custom,
}

/// Cached tile with GPU resources and metadata
pub struct CachedTile {
    /// GPU texture (kept for ownership, used via bind_group)
    #[allow(dead_code)]
    pub texture: wgpu::Texture,
    /// Texture view (kept for ownership, used via bind_group)
    #[allow(dead_code)]
    pub texture_view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
    /// Tile width in pixels
    pub width: u32,
    /// Tile height in pixels
    pub height: u32,
    /// GPU memory usage in bytes
    pub memory_size: usize,
    /// Type of tile content
    pub tile_type: TileType,
}

/// Configuration for a single zoom level cache
#[derive(Clone, Debug)]
pub struct ZoomLevelConfig {
    /// Maximum number of tiles for this zoom level
    pub max_tiles: usize,
    /// Eviction priority (higher = evict later, keep longer)
    pub priority: u8,
}

impl Default for ZoomLevelConfig {
    fn default() -> Self {
        Self {
            max_tiles: 128,
            priority: 5,
        }
    }
}

/// LRU cache for a single zoom level
struct ZoomLevelCache {
    tiles: HashMap<(u32, u32), Arc<CachedTile>>,
    access_order: Vec<(u32, u32)>,
    max_tiles: usize,
    priority: u8,
}

impl ZoomLevelCache {
    fn new(config: ZoomLevelConfig) -> Self {
        Self {
            tiles: HashMap::with_capacity(config.max_tiles),
            access_order: Vec::with_capacity(config.max_tiles),
            max_tiles: config.max_tiles,
            priority: config.priority,
        }
    }

    fn contains(&self, x: u32, y: u32) -> bool {
        self.tiles.contains_key(&(x, y))
    }

    fn peek(&self, x: u32, y: u32) -> Option<Arc<CachedTile>> {
        self.tiles.get(&(x, y)).cloned()
    }

    fn insert(&mut self, x: u32, y: u32, tile: CachedTile) {
        // Evict if needed
        while self.tiles.len() >= self.max_tiles {
            if !self.evict_oldest() {
                break;
            }
        }

        // Remove if exists (update case)
        let key = (x, y);
        if self.tiles.remove(&key).is_some() {
            self.access_order.retain(|id| *id != key);
        }

        self.tiles.insert(key, Arc::new(tile));
        self.access_order.push(key);
    }

    fn evict_oldest(&mut self) -> bool {
        if let Some(oldest) = self.access_order.first().cloned() {
            if self.tiles.remove(&oldest).is_some() {
                self.access_order.remove(0);
                log::debug!("Evicted tile at {:?}", oldest);
                return true;
            }
        }
        false
    }

    fn len(&self) -> usize {
        self.tiles.len()
    }

    fn max(&self) -> usize {
        self.max_tiles
    }

    /// Evict tiles until we're at half capacity (for distant zoom levels)
    fn shrink_to_half(&mut self) {
        let target = self.max_tiles / 2;
        while self.tiles.len() > target {
            if !self.evict_oldest() {
                break;
            }
        }
    }
}

/// Maximum supported zoom level
const MAX_ZOOM: usize = 20;

/// LRU cache for map tiles, separated by zoom level
pub struct TileCache {
    /// Per-zoom-level caches (lazy initialized)
    levels: [Option<Box<ZoomLevelCache>>; MAX_ZOOM],
    /// Default config for uninitialized levels
    default_configs: [ZoomLevelConfig; MAX_ZOOM],
}

impl TileCache {
    /// Create a new tile cache with default configuration
    pub fn new(_max_tiles: usize) -> Self {
        Self::with_default_configs()
    }

    /// Create with optimized default configurations per zoom level
    fn with_default_configs() -> Self {
        let default_configs = Self::create_default_configs();
        Self {
            levels: Default::default(),
            default_configs,
        }
    }

    /// Generate default configs optimized for map viewing
    /// Lower zoom levels (world view) have higher priority to prevent eviction
    fn create_default_configs() -> [ZoomLevelConfig; MAX_ZOOM] {
        [
            // Zoom 0-2: World/continent level - small cache, highest priority
            ZoomLevelConfig { max_tiles: 4, priority: 10 },
            ZoomLevelConfig { max_tiles: 4, priority: 10 },
            ZoomLevelConfig { max_tiles: 8, priority: 10 },
            // Zoom 3-5: Large regions - small cache, high priority
            ZoomLevelConfig { max_tiles: 16, priority: 9 },
            ZoomLevelConfig { max_tiles: 32, priority: 9 },
            ZoomLevelConfig { max_tiles: 64, priority: 8 },
            // Zoom 6-10: Country/city level - medium cache, high priority
            ZoomLevelConfig { max_tiles: 64, priority: 8 },
            ZoomLevelConfig { max_tiles: 96, priority: 7 },
            ZoomLevelConfig { max_tiles: 128, priority: 7 },
            ZoomLevelConfig { max_tiles: 128, priority: 6 },
            ZoomLevelConfig { max_tiles: 128, priority: 6 },
            // Zoom 11-15: Detail level - large cache, medium priority
            ZoomLevelConfig { max_tiles: 192, priority: 5 },
            ZoomLevelConfig { max_tiles: 256, priority: 4 },
            ZoomLevelConfig { max_tiles: 256, priority: 4 },
            ZoomLevelConfig { max_tiles: 256, priority: 3 },
            ZoomLevelConfig { max_tiles: 256, priority: 3 },
            // Zoom 16-19: Ultra detail - large cache, low priority (evict first)
            ZoomLevelConfig { max_tiles: 256, priority: 2 },
            ZoomLevelConfig { max_tiles: 256, priority: 2 },
            ZoomLevelConfig { max_tiles: 256, priority: 1 },
            ZoomLevelConfig { max_tiles: 256, priority: 1 },
        ]
    }

    /// Ensure a zoom level cache exists (lazy initialization)
    fn ensure_level(&mut self, z: u8) -> &mut ZoomLevelCache {
        let idx = (z as usize).min(MAX_ZOOM - 1);
        if self.levels[idx].is_none() {
            let config = self.default_configs[idx].clone();
            self.levels[idx] = Some(Box::new(ZoomLevelCache::new(config)));
        }
        self.levels[idx].as_mut().unwrap()
    }

    /// Get a zoom level cache if it exists
    fn get_level(&self, z: u8) -> Option<&ZoomLevelCache> {
        let idx = (z as usize).min(MAX_ZOOM - 1);
        self.levels[idx].as_ref().map(|b| b.as_ref())
    }

    /// Check if tile exists in cache
    pub fn contains(&self, tile_id: &TileId) -> bool {
        self.get_level(tile_id.z)
            .map(|cache| cache.contains(tile_id.x, tile_id.y))
            .unwrap_or(false)
    }

    /// Get a tile without updating access order (for read-only checks)
    pub fn peek(&self, tile_id: &TileId) -> Option<Arc<CachedTile>> {
        self.get_level(tile_id.z)
            .and_then(|cache| cache.peek(tile_id.x, tile_id.y))
    }

    /// Insert a new tile into cache, evicting old tiles if necessary
    pub fn insert(&mut self, tile_id: TileId, tile: CachedTile) {
        let cache = self.ensure_level(tile_id.z);
        cache.insert(tile_id.x, tile_id.y, tile);
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let mut tile_count = 0;
        let mut max_tiles = 0;

        for level in self.levels.iter().flatten() {
            tile_count += level.len();
            max_tiles += level.max();
        }

        CacheStats {
            tile_count,
            max_tiles,
        }
    }

    /// Get statistics for a specific zoom level
    pub fn level_stats(&self, z: u8) -> Option<ZoomLevelStats> {
        self.get_level(z).map(|cache| ZoomLevelStats {
            zoom: z,
            tile_count: cache.len(),
            max_tiles: cache.max(),
            priority: cache.priority,
        })
    }

    /// Optimize cache for current zoom level
    /// Shrinks distant zoom level caches to free memory for current view
    pub fn optimize_for_zoom(&mut self, current_zoom: u8) {
        for z in 0..MAX_ZOOM as u8 {
            let distance = (z as i8 - current_zoom as i8).abs() as u8;

            // Shrink caches that are far from current zoom (>3 levels away)
            if distance > 3 {
                let idx = z as usize;
                if let Some(cache) = self.levels[idx].as_mut() {
                    cache.shrink_to_half();
                }
            }
        }
    }

    /// Get all active zoom levels with their tile counts
    pub fn active_levels(&self) -> Vec<(u8, usize)> {
        self.levels
            .iter()
            .enumerate()
            .filter_map(|(z, cache)| {
                cache.as_ref().map(|c| (z as u8, c.len()))
            })
            .filter(|(_, count)| *count > 0)
            .collect()
    }
}

/// Cache statistics for debugging/UI
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub tile_count: usize,
    pub max_tiles: usize,
}

/// Statistics for a single zoom level
#[derive(Debug, Clone, Copy)]
pub struct ZoomLevelStats {
    pub zoom: u8,
    pub tile_count: usize,
    pub max_tiles: usize,
    pub priority: u8,
}

impl Default for TileCache {
    fn default() -> Self {
        Self::with_default_configs()
    }
}
