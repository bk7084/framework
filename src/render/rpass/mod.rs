mod blph;
#[allow(dead_code)]
mod skybox;

use crate::{
    render::{Pipelines, RenderParams, RenderTarget, Renderer},
    scene::Scene,
};
pub use blph::*;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use std::num::NonZeroU32;

crate::impl_size_constant!(
    Globals,
    Locals,
    ShadowPassLocals,
    PConsts,
    PConstsShadowPass,
    GpuLight,
    LightArray
);

/// The global uniforms for the rendering passes.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Globals {
    /// The view matrix.
    pub view: [f32; 16],
    /// The projection matrix.
    pub proj: [f32; 16],
}

/// The local information (per entity/instance) for the rendering passes.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Locals {
    /// The model matrix.
    model: [f32; 16],
    /// The transpose of the inverse of the model-view matrix.
    model_view_it: [f32; 16],
    /// The material index in case of overriding the material.
    material_index: [u32; 4],
}

impl Locals {
    pub const fn identity() -> Self {
        Self {
            model: Mat4::IDENTITY.to_cols_array(),
            model_view_it: Mat4::IDENTITY.to_cols_array(),
            material_index: [u32::MAX; 4],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ShadowPassLocals {
    /// The model matrix.
    pub model: [f32; 16],
}

impl ShadowPassLocals {
    pub const fn identity() -> Self {
        Self {
            model: Mat4::IDENTITY.to_cols_array(),
        }
    }
}

impl InstanceLocals for Locals {
    const SIZE: usize = Self::SIZE;
    const BUFFER_SIZE: Option<wgpu::BufferSize> = Self::BUFFER_SIZE;
}

impl InstanceLocals for ShadowPassLocals {
    const SIZE: usize = Self::SIZE;
    const BUFFER_SIZE: Option<wgpu::BufferSize> = Self::BUFFER_SIZE;
}

/// Push constants for the shading pipeline.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct PConsts {
    /// Index of the first instance in the instance buffer.
    instance_base_index: u32,
    /// Material index.
    material_index: u32,
    /// Whether the shadow mapping is enabled.
    enable_shadows: u32,
    /// Whether the lighting is enabled.
    enable_lighting: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct PConstsShadowPass {
    instance_base_index: u32,
    light_index: u32,
}

/// Depth format for the rendering passes.
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// The binding group for the global uniforms.
///
/// See [`Globals`] for the uniforms.
pub struct GlobalsBindGroup {
    /// The bind group.
    pub group: wgpu::BindGroup,
    /// The layout of the bind group.
    pub layout: wgpu::BindGroupLayout,
    /// The uniform buffer containing the global uniforms.
    pub buffer: wgpu::Buffer,
}

impl<'a> Into<Option<&'a wgpu::BindGroup>> for &'a GlobalsBindGroup {
    fn into(self) -> Option<&'a wgpu::BindGroup> {
        Some(&self.group)
    }
}

/// The local information (per entity/instance) for the rendering passes.
pub trait InstanceLocals {
    const SIZE: usize;
    const BUFFER_SIZE: Option<wgpu::BufferSize>;
}

/// The binding group for the local information (per entity/instance).
///
/// See [`Locals`] for the uniforms.
pub struct LocalsBindGroup<L: InstanceLocals> {
    /// The bind group.
    pub group: wgpu::BindGroup,
    /// The layout of the bind group.
    pub layout: wgpu::BindGroupLayout,
    /// The storage buffer containing the locals for all entities.
    pub buffer: wgpu::Buffer,
    /// Maximum number of instances in the storage buffer.
    capacity: u32,
    _marker: std::marker::PhantomData<[L]>,
}

impl<L: InstanceLocals> LocalsBindGroup<L> {
    /// Initial instance capacity for mesh entities.
    pub const INITIAL_INSTANCE_CAPACITY: usize = 1024;
}

impl<'a, L: InstanceLocals> Into<Option<&'a wgpu::BindGroup>> for &'a LocalsBindGroup<L> {
    fn into(self) -> Option<&'a wgpu::BindGroup> {
        Some(&self.group)
    }
}

/// Light information for the shader.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct GpuLight {
    pub dir_or_pos: [f32; 4],
    pub color: [f32; 4],
    pub w2l: [f32; 16],
}

/// Array of lights passed to the shader as a storage buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LightArray {
    pub len: [u32; 4], // with padding to make sure the array is 16-byte aligned.
    pub lights: [GpuLight; BlinnPhongRenderPass::MAX_LIGHTS],
}

impl Default for LightArray {
    fn default() -> Self {
        Self {
            len: [0; 4],
            lights: [GpuLight::default(); BlinnPhongRenderPass::MAX_LIGHTS],
        }
    }
}

impl LightArray {
    /// Only reset the length of the array.
    pub fn clear(&mut self) {
        self.len = [0; 4];
    }

    pub fn is_empty(&self) -> bool {
        self.len[0] == 0
    }

    pub fn len(&self) -> usize {
        self.len[0] as usize
    }
}

/// The binding group for the lights.
pub struct LightsBindGroup {
    /// The bind group.
    pub group: wgpu::BindGroup,
    /// The layout of the bind group.
    pub layout: wgpu::BindGroupLayout,
    /// The storage buffer containing the lights.
    /// See [`LightArray`].
    pub lights_buffer: wgpu::Buffer,
    /// Cached lights of each frame to avoid unnecessary allocation.
    lights: LightArray,
}

impl<'a> Into<Option<&'a wgpu::BindGroup>> for &'a LightsBindGroup {
    fn into(self) -> Option<&'a wgpu::BindGroup> {
        Some(&self.group)
    }
}

pub trait RenderingPass {
    fn record(
        &mut self,
        renderer: &Renderer,
        target: &RenderTarget,
        params: &RenderParams,
        scene: &Scene,
        encoder: &mut wgpu::CommandEncoder,
    );
}

/// Helper struct managing the shadow maps of the same size to minimize the
/// number of textures and memory usage.
///
/// Shadow maps are 2D texture arrays, each of which contains multiple layers
/// of depth textures. The number of layers in each texture is limited by the
/// device. Therefore, if the number of shadow maps is large, we need to create
/// multiple textures to store all the shadow maps.
///
/// Bind group layout:
///
/// ```wgsl
/// var shadow_maps: binding_array<texture_depth_2d_array>;
/// ```
///
/// ```glsl
/// uniform texture2D textures[10] (GLSL)
/// ```
pub struct ShadowMaps {
    /// The size of the shadow map in pixels.
    pub shadow_map_size: (u32, u32),
    /// The number of shadow maps, which is the number of lights casting
    /// shadows.
    pub shadow_map_count: u32,
    /// The textures storing the shadow maps. Each texture is a 2D texture
    /// array with `layers_per_texture` layers, and the last texture may have
    /// less layers.
    pub depth_textures: Vec<(wgpu::Texture, wgpu::TextureView)>,
    /// The sampler for the shadow maps.
    pub depth_sampler: wgpu::Sampler,
    /// The texture views for the shadow maps to be used as the depth
    /// attachment.
    shadow_map_views: Vec<wgpu::TextureView>,
    /// The bind group for the shadow maps. In case the number or the size of
    /// the shadow maps changes, the bind group and the bind group layout need
    /// to be recreated. See [`update`] for more.
    pub bind_group: wgpu::BindGroup,
    /// The bind group layout for the shadow maps.
    pub bind_group_layout: wgpu::BindGroupLayout,

    /// The number of layers per texture.
    layers_per_texture: u32,

    #[cfg(all(debug_assertions, feature = "debug-shadow-map"))]
    /// The storage buffers for the shadow maps. Each buffer stores the
    /// shadow map for a light. Used for debugging only.
    pub storage_buffers: Vec<wgpu::Buffer>,
}

impl ShadowMaps {
    /// Create a set of shadow maps.
    ///
    /// # Arguments
    ///
    /// * `renderer` - The renderer.
    /// * `width` - The width of the shadow maps.
    /// * `height` - The height of the shadow maps.
    /// * `count` - The number of shadow maps.
    pub fn new(
        device: &wgpu::Device,
        limits: &wgpu::Limits,
        width: u32,
        height: u32,
        count: u32,
    ) -> Self {
        debug_assert!(
            width <= limits.max_texture_dimension_1d,
            "Shadow map width exceeds the limit."
        );
        debug_assert!(
            height <= limits.max_texture_dimension_1d,
            "Shadow map height exceeds the limit."
        );

        let layers_per_texture = limits.max_texture_array_layers;
        let n_textures = (count + layers_per_texture - 1) / layers_per_texture;
        let last_texture_layers = count % layers_per_texture;

        // Create the depth textures, each of which is a 2D texture array with
        // `layers_per_texture` layers, and the last texture may have less layers.
        let depth_textures = (0..n_textures)
            .map(|n| {
                let layer_count = if n == n_textures - 1 {
                    last_texture_layers
                } else {
                    layers_per_texture
                };
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("shadow_maps_depth_texture"),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: layer_count,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: DEPTH_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });
                let view = texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("shadow_maps_depth_texture_view"),
                    format: Some(DEPTH_FORMAT),
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    aspect: wgpu::TextureAspect::All,
                    base_array_layer: 0,
                    array_layer_count: Some(layer_count),
                    ..Default::default()
                });
                (texture, view)
            })
            .collect::<Vec<_>>();

        #[cfg(all(debug_assertions, feature = "debug-shadow-map"))]
        let storage_buffers = (0..count)
            .map(|_| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("shadow_maps_storage_buffer"),
                    size: (width * height * std::mem::size_of::<f32>() as u32) as u64,
                    usage: wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC
                        | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                })
            })
            .collect::<Vec<_>>();

        let depth_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow_maps_depth_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow_maps_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: NonZeroU32::new(n_textures),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });

        // Create the bind group for using the shadow maps in the main pass.
        let views = depth_textures
            .iter()
            .map(|(_, view)| view)
            .collect::<Vec<_>>();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow_maps_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&views),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&depth_sampler),
                },
            ],
        });

        let shadow_map_views = (0..count)
            .map(|i| {
                let texture_index = i / layers_per_texture;
                let layer_index = i % layers_per_texture;
                depth_textures[texture_index as usize]
                    .0
                    .create_view(&wgpu::TextureViewDescriptor {
                        label: Some(&format!("shadow_map_view_{}", i)),
                        format: Some(DEPTH_FORMAT),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        aspect: wgpu::TextureAspect::DepthOnly,
                        base_array_layer: layer_index,
                        array_layer_count: Some(1),
                        ..Default::default()
                    })
            })
            .collect::<Vec<_>>();

        Self {
            depth_textures,
            bind_group,
            bind_group_layout,
            shadow_map_size: (width, height),
            shadow_map_count: count,
            shadow_map_views,
            depth_sampler,
            layers_per_texture,
            #[cfg(all(debug_assertions, feature = "debug-shadow-map"))]
            storage_buffers,
        }
    }

    /// Returns the underlying texture of the shadow map at the given index
    /// together with the index of the layer in the texture.
    pub fn texture(&self, index: usize) -> (&wgpu::Texture, u32) {
        let texture_index = index / self.layers_per_texture as usize;
        let layer_index = index % self.layers_per_texture as usize;
        (&self.depth_textures[texture_index].0, layer_index as u32)
    }

    pub fn shadow_maps(&self) -> &[wgpu::TextureView] {
        &self.shadow_map_views
    }

    /// Recreate the shadow maps in case the size or the number of shadow maps
    /// changes.
    pub fn update(
        &mut self,
        device: &wgpu::Device,
        limits: &wgpu::Limits,
        width: u32,
        height: u32,
        count: u32,
    ) {
        if self.shadow_map_size.0 != width
            || self.shadow_map_size.1 != height
            || self.shadow_map_count != count
        {
            *self = Self::new(device, limits, width, height, count);
        }
    }

    /// Copys the shadow maps to the storage buffers.
    #[cfg(all(debug_assertions, feature = "debug-shadow-map"))]
    pub fn update_storage_buffers(&mut self, encoder: &mut wgpu::CommandEncoder) {
        for (i, buffer) in self.storage_buffers.iter().enumerate() {
            let (texture, layer_idx) = self.texture(i);
            encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer_idx,
                    },
                    aspect: wgpu::TextureAspect::DepthOnly,
                },
                wgpu::ImageCopyBuffer {
                    buffer: &buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(self.shadow_map_size.0 * 4),
                        rows_per_image: Some(self.shadow_map_size.1),
                    },
                },
                wgpu::Extent3d {
                    width: self.shadow_map_size.0,
                    height: self.shadow_map_size.1,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    /// Write the shadow maps to files for debugging.
    #[cfg(all(debug_assertions, feature = "debug-shadow-map"))]
    pub fn write_shadow_maps(&self, device: &wgpu::Device) {
        for (i, buffer) in self.storage_buffers.iter().enumerate() {
            let buffer_slice = buffer.slice(..);
            let (sender, receiver) = flume::bounded(1);
            buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
            device.poll(wgpu::Maintain::Wait);
            pollster::block_on(async {
                receiver.recv_async().await.unwrap().unwrap();
            });
            {
                let buffer_view = buffer_slice.get_mapped_range();
                let (_, data, _) = unsafe { buffer_view.align_to::<f32>() };
                let (width, height) = self.shadow_map_size;
                let mut img = image::ImageBuffer::new(width, height);
                for (x, y, pixel) in img.enumerate_pixels_mut() {
                    let idx = (y * width + x) as usize;
                    *pixel = image::Luma([(data[idx] * 255.0) as u8]);
                }
                img.save(format!(
                    "shadow_map_{}_{:?}.png",
                    i,
                    std::time::Instant::now()
                ))
                .unwrap();
            }
            buffer.unmap();
        }
    }
}

/// The render pass for the blinn-phong shading.
pub struct BlinnPhongRenderPass {
    /// The depth attachment.
    pub depth_att: Option<(wgpu::Texture, wgpu::TextureView)>,
    /// The global uniforms bind group.
    pub globals_bind_group: GlobalsBindGroup,
    /// The local information (per entity/instance) bind group for visible
    /// entities.
    pub locals_bind_group: LocalsBindGroup<Locals>,
    /// The local information (per entity/instance) bind group for entities
    /// casting shadows.
    pub shadow_pass_locals_bind_group: LocalsBindGroup<ShadowPassLocals>,
    pub materials_bind_group_layout: wgpu::BindGroupLayout,
    pub textures_bind_group_layout: wgpu::BindGroupLayout,
    /// The lights bind group.
    pub lights_bind_group: LightsBindGroup,
    /// The shadow maps.
    pub shadow_maps: ShadowMaps,
    /// The pipelines.
    pub pipelines: Pipelines,
}

impl BlinnPhongRenderPass {
    /// Maximum number of directional lights.
    pub const MAX_DIR_LIGHTS: usize = 64;
    /// Maximum number of point lights.
    pub const MAX_PNT_LIGHTS: usize = 448;
    /// Maximum number of lights.
    pub const MAX_LIGHTS: usize = Self::MAX_DIR_LIGHTS + Self::MAX_PNT_LIGHTS;
    /// Maximum number of textures in a texture binding array.
    pub const MAX_TEXTURE_ARRAY_LEN: usize = 64;
    /// Maximum number of texture sampler in a texture sampler bindingr array.
    pub const MAX_SAMPLER_ARRAY_LEN: usize = 8;
}
