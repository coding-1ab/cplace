//! Map system with tile rendering, caching, and pixel grid overlay

pub mod cache;
pub mod camera;
pub mod grid;
pub mod loader;
pub mod renderer;
pub mod tile;

use crate::map::cache::TileType;
use cache::TileCache;
use camera::MapCamera;
use loader::{TileLoadResult, TileLoader};
use renderer::{TileRenderer, screen_to_ndc, size_to_ndc};
use std::num::NonZeroUsize;
use tile::TileId;

/// Integrated map system
pub struct MapSystem {
    tile_cache: TileCache,
    tile_loader: TileLoader,
    tile_renderer: TileRenderer,

    /// Tiles to render this frame (calculated in update)
    /// id, (x, y), (width, height)
    render_tiles: Vec<(TileId, (f32, f32), (f32, f32))>,
}

impl MapSystem {
    /// Create a new map system
    pub fn new(
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
        tiles: NonZeroUsize,
    ) -> Self {
        let tile_cache = TileCache::new(tiles);
        let tile_loader = TileLoader::default();
        let tile_renderer = TileRenderer::new(device, texture_format);

        Self {
            tile_cache,
            tile_loader,
            tile_renderer,
            render_tiles: Vec::new(),
        }
    }

    /// Update the map system (call each frame)
    pub fn update(&mut self, camera: &MapCamera, device: &wgpu::Device, queue: &wgpu::Queue) {
        // 1. Get visible tiles
        let visible = camera.visible_tiles_with_buffer(1);

        // 2. Request loading for tiles not in cache
        for tile_id in &visible {
            if !self.tile_cache.contains(tile_id) && !self.tile_loader.is_loading(tile_id) {
                self.tile_loader.request(*tile_id);
            }
        }

        // 3. Process completed loads
        while let Some(result) = self.tile_loader.poll() {
            match result {
                TileLoadResult::Success(id, data) => {
                    let cached = self.tile_renderer.create_cached_tile_with_type(
                        device,
                        queue,
                        &data,
                        TileType::MapTile,
                    );
                    log::debug!("Loaded tile {:?}", id);
                    self.tile_cache.insert(id, cached);
                }
                TileLoadResult::Failed(id, err) => {
                    log::warn!("Failed to load tile {:?}: {}", id, err);
                }
            }
        }

        // 4. Build render list with screen positions
        self.render_tiles.clear();
        let tile_size = camera.tile_screen_size();

        for tile_id in &visible {
            // Only add to render list if cached
            if self.tile_cache.contains(tile_id) {
                let (x, y) = camera.tile_to_screen(tile_id);

                // Convert to NDC
                let (ndc_x, ndc_y) = screen_to_ndc(
                    x,
                    y,
                    camera.viewport_width,
                    camera.viewport_height,
                );
                let (ndc_w, ndc_h) = size_to_ndc(
                    tile_size,
                    camera.viewport_width,
                    camera.viewport_height,
                );

                self.render_tiles
                    .push((*tile_id, (ndc_x, ndc_y), (ndc_w, ndc_h)));
            }
        }
    }

    /// Render the map
    pub fn render<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>, device: &wgpu::Device) {
        // Render tiles
        self.tile_renderer.render(
            render_pass,
            device,
            &self.render_tiles,
            &mut self.tile_cache,
        );
    }

    /// Get cache statistics
    pub fn cache_stats(&self, zoom_label: usize) -> cache::CacheStats {
        self.tile_cache.stats(zoom_label)
    }

    /// Get pending tile count
    pub fn pending_tiles(&self) -> usize {
        self.tile_loader.pending_count()
    }
}
