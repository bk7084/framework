mod blph;
mod skybox;

use crate::{
    render::{Pipelines, RenderParams, RenderTarget, Renderer},
    scene::Scene,
};
pub use blph::*;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use std::ops::Deref;

crate::impl_size_constant!(Globals, Locals, PConsts, GpuLight, LightArray);

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
struct Locals {
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

/// Push constants for the shading pipeline.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct PConsts {
    /// Index of the first instance in the instance buffer.
    instance_base_index: u32,
    /// Material index.
    material_index: u32,
}

/// Depth format for the rendering passes.
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

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

impl Deref for GlobalsBindGroup {
    type Target = wgpu::BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.group
    }
}

/// The binding group for the local information (per entity/instance).
///
/// See [`Locals`] for the uniforms.
pub struct LocalsBindGroup {
    /// The bind group.
    pub group: wgpu::BindGroup,
    /// The layout of the bind group.
    pub layout: wgpu::BindGroupLayout,
    /// The storage buffer containing the locals for all entities.
    pub buffer: wgpu::Buffer,
    /// Maximum number of instances in the storage buffer.
    capacity: u32,
}

impl Deref for LocalsBindGroup {
    type Target = wgpu::BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.group
    }
}

impl LocalsBindGroup {
    /// Initial instance capacity for mesh entities.
    pub const INITIAL_INSTANCE_CAPACITY: usize = 1024;
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

impl Deref for LightsBindGroup {
    type Target = wgpu::BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.group
    }
}

pub trait RenderingPass {
    fn record(
        &mut self,
        renderer: &Renderer,
        target: &RenderTarget,
        params: &RenderParams,
        scene: &Scene,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    );
}

/// The render pass for the blinn-phong shading.
pub struct BlinnPhongRenderPass {
    /// The depth attachment.
    pub depth_att: Option<(wgpu::Texture, wgpu::TextureView)>,
    /// The global uniforms bind group.
    pub globals_bind_group: GlobalsBindGroup,
    /// The local information (per entity/instance) bind group.
    pub locals_bind_group: LocalsBindGroup,
    pub materials_bind_group_layout: wgpu::BindGroupLayout,
    pub textures_bind_group_layout: wgpu::BindGroupLayout,
    /// The lights bind group.
    pub lights_bind_group: LightsBindGroup,
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
    pub const MAX_TEXTURE_ARRAY_LEN: usize = 128;
    /// Maximum number of texture sampler in a texture sampler binding array.
    pub const MAX_SAMPLER_ARRAY_LEN: usize = 16;
}
