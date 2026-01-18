//! LRU tile cache for GPU textures

use std::collections::HashMap;
use std::sync::Arc;

use super::tile::TileId;

/// Cached tile with GPU resources
pub struct CachedTile {
    /// GPU texture (kept for ownership, used via bind_group)
    #[allow(dead_code)]
    pub texture: wgpu::Texture,
    /// Texture view (kept for ownership, used via bind_group)
    #[allow(dead_code)]
    pub texture_view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
    pub memory_size: usize,
}

/// LRU cache for map tiles
pub struct TileCache {
    tiles: HashMap<TileId, Arc<CachedTile>>,
    access_order: Vec<TileId>,
    max_tiles: usize,
    current_memory: usize,
    max_memory: usize,
}

impl TileCache {
    /// Create a new tile cache
    /// - max_tiles: Maximum number of tiles to cache (e.g., 256)
    /// - max_memory: Maximum GPU memory in bytes (e.g., 64MB)
    pub fn new(max_tiles: usize, max_memory: usize) -> Self {
        Self {
            tiles: HashMap::with_capacity(max_tiles),
            access_order: Vec::with_capacity(max_tiles),
            max_tiles,
            current_memory: 0,
            max_memory,
        }
    }

    /// Check if tile exists in cache
    pub fn contains(&self, tile_id: &TileId) -> bool {
        self.tiles.contains_key(tile_id)
    }

    /// Get a tile without updating access order (for read-only checks)
    pub fn peek(&self, tile_id: &TileId) -> Option<Arc<CachedTile>> {
        self.tiles.get(tile_id).cloned()
    }

    /// Insert a new tile into cache, evicting old tiles if necessary
    pub fn insert(&mut self, tile_id: TileId, tile: CachedTile) {
        let memory_size = tile.memory_size;

        // Evict tiles if we're over capacity
        while self.should_evict(memory_size) {
            if !self.evict_oldest() {
                break;
            }
        }

        // Remove if already exists (update case)
        if let Some(old) = self.tiles.remove(&tile_id) {
            self.current_memory -= old.memory_size;
            self.access_order.retain(|id| id != &tile_id);
        }

        self.current_memory += memory_size;
        self.tiles.insert(tile_id, Arc::new(tile));
        self.access_order.push(tile_id);
    }

    /// Check if we need to evict tiles
    fn should_evict(&self, new_tile_memory: usize) -> bool {
        !self.tiles.is_empty()
            && (self.tiles.len() >= self.max_tiles
                || self.current_memory + new_tile_memory > self.max_memory)
    }

    /// Evict the oldest (least recently used) tile
    fn evict_oldest(&mut self) -> bool {
        if let Some(oldest_id) = self.access_order.first().cloned() {
            if let Some(tile) = self.tiles.remove(&oldest_id) {
                self.current_memory -= tile.memory_size;
                self.access_order.remove(0);
                log::debug!("Evicted tile {:?}", oldest_id);
                return true;
            }
        }
        false
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            tile_count: self.tiles.len(),
            max_tiles: self.max_tiles,
        }
    }
}

/// Cache statistics for debugging/UI
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub tile_count: usize,
    pub max_tiles: usize,
}

impl Default for TileCache {
    fn default() -> Self {
        // Default: 256 tiles, 64MB max
        Self::new(256, 64 * 1024 * 1024)
    }
}
