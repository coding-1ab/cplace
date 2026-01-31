//! LRU tile cache for GPU textures with zoom-level separation

use super::tile::TileId;
use arrayvec::ArrayVec;
use log::warn;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;

/// Tile type for different use cases
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TileType {
    /// Standard OSM map tile (typically 256x256)
    #[default]
    MapTile,
    /// Pixel art overlay (32x32)
    PixelArt,
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
    pub width: u32,
    /// Type of tile content
    pub tile_type: TileType,
}

/// LRU cache for a single zoom level
#[derive(Clone, Debug)]
struct ZoomLevelCache(LruCache<TileId, Arc<CachedTile>>);

// TODO Share this with server
/// Maximum supported zoom level
pub const MAX_ZOOM: usize = 14;

/// LRU cache for map tiles, separated by zoom level
#[derive(Debug)]
pub struct TileCache {
    /// Per-zoom-level caches (lazy initialized)
    levels: [ZoomLevelCache; MAX_ZOOM + 1],
}

impl TileCache {
    /// Create a new tile cache with default configuration
    pub fn new(tiles_per_level: NonZeroUsize) -> Self {
        let levels: ArrayVec<_, { MAX_ZOOM + 1 }> = (0..=MAX_ZOOM)
            .map(|_| ZoomLevelCache(LruCache::new(tiles_per_level)))
            .collect();
        let levels = levels.into_inner().unwrap();
        Self { levels }
    }

    /// Check if tile exists in cache
    pub fn contains(&self, tile_id: &TileId) -> bool {
        self.levels
            .get(tile_id.z as usize)
            .map(|cache| cache.0.contains(tile_id))
            .unwrap_or(false)
    }

    pub fn get(&mut self, tile_id: &TileId) -> Option<Arc<CachedTile>> {
        self.levels.get_mut(tile_id.z as usize)
            .and_then(|cache| cache.0.get(tile_id).cloned())
    }

    /// Insert a new tile into cache, evicting old tiles if necessary
    pub fn insert(&mut self, tile_id: TileId, tile: CachedTile) {
        let Some(cache) = self.levels.get_mut(tile_id.z as usize) else {
            warn!("Invalid Zoom Level: {}", tile_id.z);
            return;
        };

        cache.0.push(tile_id, Arc::new(tile));
    }

    /// Get cache statistics
    pub fn stats(&self, zoom_level: usize) -> CacheStats {
        let mut tile_count = 0;
        let mut max_tiles = 0;

        let level = &self.levels[zoom_level].0;
        tile_count += level.len();
        max_tiles += level.cap().get();

        CacheStats {
            tile_count,
            max_tiles: NonZeroUsize::new(max_tiles).unwrap(),
        }
    }
}

/// Cache statistics for debugging/UI
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub tile_count: usize,
    pub max_tiles: NonZeroUsize,
}
