use std::{mem, num::NonZeroUsize};

use wgpu::wgc::device::queue;

use crate::map::{
    cache::{TileCache, TileType},
    loader::TileLoader,
    renderer::TileRenderer,
    tile::TileId,
};

/// Integrated map system
pub struct PixelArtSystem {
    tile_cache: TileCache,
    tile_loader: TileLoader,
    tile_renderer: TileRenderer,

    palette: wgpu::Texture,
    palette_sampler: wgpu::Sampler,

    /// Tiles to render this frame (calculated in update)
    /// id, (x, y), (width, height)
    render_tiles: Vec<(TileId, (f32, f32), (f32, f32))>,
}

const VGA: &[u8; 256 * 3] = include_bytes!("./palette.bin");

impl PixelArtSystem {
    /// Create a new pixel art map system
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let tile_cache = TileCache::new(NonZeroUsize::new(512).unwrap());
        let tile_loader = TileLoader::new(TileType::PixelArt);
        let tile_renderer = TileRenderer::new(device, texture_format);

        let palette = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Pixel Art Palette Texture"),
            size: wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let palette_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Pixel Art Palette Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &palette,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            VGA,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(mem::size_of::<u8>() as u32 * 256),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        Self {
            tile_cache,
            tile_loader,
            tile_renderer,
            palette,
            palette_sampler,
            render_tiles: Vec::new(),
        }
    }
}
