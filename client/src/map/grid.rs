use wgpu::{
    BindGroupDescriptor, BindGroupLayoutDescriptor, BindGroupLayoutEntry, include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    wgc::device::queue,
};

use crate::map::camera::MapCamera;

/// Grid vertex for colored quads
pub struct Grid {
    /// Render pipeline
    render_pipeline: wgpu::RenderPipeline,
    camera_position_buffer: wgpu::Buffer,
    viewport_size_buffer: wgpu::Buffer,
    zoom_level_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Grid {
    /// Create a new pixel grid overlay
    pub fn new(
        camera: &MapCamera,
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        // Load shaders
        let shader = device.create_shader_module(include_wgsl!("../shader/grid.wgsl"));

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("grid_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("grid_pipeline_layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("grid_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let camera_position_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera_position_buffer"),
            contents: bytemuck::cast_slice(&[camera.center.0, camera.center.1]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let viewport_size_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("viewport_size_buffer"),
            contents: bytemuck::cast_slice(&[camera.viewport_width, camera.viewport_height]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let zoom_level_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("zoom_level_buffer"),
            contents: bytemuck::cast_slice(&[camera.zoom as f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("grid_bind_group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_position_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: viewport_size_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: zoom_level_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            render_pipeline,
            camera_position_buffer,
            viewport_size_buffer,
            zoom_level_buffer,
            bind_group,
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }

    pub fn update(&self, queue: &wgpu::Queue, camera: &MapCamera) {
        // Update camera position
        let data = [camera.center.0 as f32, camera.center.1 as f32];
        let camera_position_data = bytemuck::cast_slice(&data);
        queue.write_buffer(&self.camera_position_buffer, 0, camera_position_data);

        // Update viewport size
        let data = [camera.viewport_width, camera.viewport_height];
        let viewport_size_data = bytemuck::cast_slice(&data);
        queue.write_buffer(&self.viewport_size_buffer, 0, viewport_size_data);

        // Update zoom level
        let data = [camera.zoom as f32];
        let zoom_level_data = bytemuck::cast_slice(&data);
        queue.write_buffer(&self.zoom_level_buffer, 0, zoom_level_data);
    }
}
