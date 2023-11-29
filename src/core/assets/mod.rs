mod handle;
pub mod storage;

use crate::core::{
    assets::storage::GpuMeshStorage,
    mesh::{GpuMesh, Mesh},
    texture::Texture,
    MaterialBundle, SmlString, TextureBundle,
};
pub use handle::*;
use std::path::Path;
use wgpu::TextureFormat;

/// Trait for representing an asset.
pub trait Asset: Send + Sync {}

pub trait AssetStorage {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

/// A collection of assets of the same type.
pub struct AssetBundle<A: Asset, S: AssetStorage> {
    storage: S,
    allocator: HandleAllocator<A>,
}

impl<A: Asset, S: AssetStorage> AssetBundle<A, S> {
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }
}

impl<A: Asset, S: AssetStorage> Default for AssetBundle<A, S>
where
    S: Default,
{
    fn default() -> Self {
        Self {
            storage: S::default(),
            allocator: HandleAllocator::new(),
        }
    }
}

impl<A: Asset> AssetStorage for Vec<Option<A>> {
    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<A: Asset> AssetBundle<A, Vec<Option<A>>> {
    /// Adds a new asset to the storage and returns its handle.
    pub fn add(&mut self, asset: A) -> Handle<A> {
        let handle = self.allocator.reserve();
        self.flush();
        self.storage[handle.index as usize] = Some(asset);
        log::debug!(
            "add({:?}), len: {}",
            std::any::type_name::<A>(),
            self.storage.len()
        );
        handle
    }

    /// Returns the asset with the given handle.
    pub fn get(&self, handle: Handle<A>) -> Option<&A> {
        self.storage[handle.index as usize].as_ref()
    }

    /// Inserts a new asset into the storage at the given index.
    ///
    /// Returns true if the asset was inserted.
    pub fn insert(&mut self, handle: Handle<A>, asset: A) -> Option<A> {
        self.flush();
        self.storage[handle.index as usize].replace(asset)
    }

    /// Removes an asset from the storage at the given index and returns it.
    pub fn remove(&mut self, handle: Handle<A>) -> Option<A> {
        self.flush();
        match self.storage[handle.index as usize].take() {
            Some(asset) => {
                self.allocator.recycle(handle);
                Some(asset)
            }
            None => None,
        }
    }

    /// Flushes the asset storage, removing those assets of which the handle
    /// is recycled.
    pub fn flush(&mut self) {
        let new_len = self
            .allocator
            .next_index
            .load(std::sync::atomic::Ordering::Relaxed) as usize;
        if new_len != self.storage.len() {
            self.storage.resize_with(new_len, || None);
        }
        while let Ok(recycled) = self.allocator.recycle_receiver.try_recv() {
            self.storage[recycled.index as usize] = None;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &A> {
        self.storage.iter().filter_map(|a| a.as_ref())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut A> {
        self.storage.iter_mut().filter_map(|a| a.as_mut())
    }
}

/// A collection of GPU meshes.
pub type GpuMeshAssets = AssetBundle<GpuMesh, GpuMeshStorage>;

/// Returns true if the given GPU mesh is created from the given mesh.
fn same_mesh(a: &Mesh, b: &GpuMesh) -> bool {
    a.id == b.mesh_id || (a.path.is_some() && a.path == b.mesh_path)
}

// Specialize the `Assets` type for `GpuMesh` as it needs a custom storage.
impl AssetBundle<GpuMesh, GpuMeshStorage> {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            storage: GpuMeshStorage::new(device),
            allocator: HandleAllocator::new(),
        }
    }

    pub fn add(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh: &Mesh,
    ) -> (Handle<GpuMesh>, bool) {
        for gpu_mesh in self.storage.data.iter() {
            if let Some((hdl, gpu_mesh)) = gpu_mesh {
                if same_mesh(mesh, gpu_mesh) {
                    log::info!("Found existing mesh: {:?}", gpu_mesh.mesh_id);
                    return (*hdl, true);
                }
            }
        }

        let handle = self.allocator.reserve();
        self.flush();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("mesh_add"),
        });

        let gpu_mesh = self.storage.add(device, queue, &mut encoder, mesh);
        self.storage.data[handle.index as usize] = Some((handle, gpu_mesh));

        queue.submit(std::iter::once(encoder.finish()));

        (handle, false)
    }

    pub fn get(&self, handle: Handle<GpuMesh>) -> Option<&GpuMesh> {
        self.storage.data[handle.index as usize]
            .as_ref()
            .map(|(_, mesh)| mesh)
    }

    pub fn remove(&mut self, handle: Handle<GpuMesh>) -> Option<GpuMesh> {
        self.flush();
        match self.storage.data[handle.index as usize].take() {
            Some(mesh) => {
                self.allocator.recycle(handle);
                Some(mesh.1)
            }
            None => None,
        }
    }

    /// Returns the buffer containing the mesh data.
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.storage.buffer
    }

    /// Flushes the asset storage, removing those assets of which the handle
    /// is recycled.
    pub fn flush(&mut self) {
        let new_len = self
            .allocator
            .next_index
            .load(std::sync::atomic::Ordering::Relaxed) as usize;
        if new_len != self.storage.len() {
            self.storage.data.resize_with(new_len, || None);
        }
        while let Ok(recycled) = self.allocator.recycle_receiver.try_recv() {
            self.storage.data[recycled.index as usize] = None;
        }
    }
}

/// A collection of materials.
pub type MaterialBundleAssets = AssetBundle<MaterialBundle, Vec<Option<MaterialBundle>>>;

impl AssetBundle<MaterialBundle, Vec<Option<MaterialBundle>>> {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            allocator: HandleAllocator::new(),
        }
    }
}

/// A collection of meshes (CPU-side).
pub type MeshAssets = AssetBundle<Mesh, Vec<Option<Mesh>>>;

/// A collection of textures.
pub type TextureAssets = AssetBundle<Texture, Vec<Option<Texture>>>;

impl AssetBundle<Texture, Vec<Option<Texture>>> {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            allocator: HandleAllocator::new(),
        }
    }

    pub fn load_from_bytes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        path: Option<&Path>,
        format: Option<wgpu::TextureFormat>,
    ) -> Handle<Texture> {
        let img = image::load_from_memory(bytes)
            .map_err(|e| eprintln!("Failed to load texture: {:?} from {:?}", e, path))
            .unwrap()
            .to_rgba8();
        let dims = img.dimensions();
        let size = wgpu::Extent3d {
            width: dims.0,
            height: dims.1,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format.unwrap_or(TextureFormat::Rgba8UnormSrgb),
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[TextureFormat::Rgba8UnormSrgb],
        };
        let raw = device.create_texture(&desc);
        let view = raw.create_view(&wgpu::TextureViewDescriptor::default());
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &raw,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &img,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dims.0),
                rows_per_image: Some(dims.1),
            },
            size,
        );
        let texture = Texture {
            raw,
            view,
            size,
            sampler: SmlString::from("default"),
        };
        self.add(texture)
    }

    /// Creates a new texture by loading it from a file.
    pub fn load_from_file(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        filepath: &Path,
        format: Option<wgpu::TextureFormat>,
    ) -> Handle<Texture> {
        log::debug!("---- Loaded image from: {:?}", filepath);
        let bytes = std::fs::read(filepath).expect(&format!(
            "Failed to read texture file: {}",
            filepath.display()
        ));
        self.load_from_bytes(device, queue, &bytes, Some(filepath), format)
    }
}

pub type TextureBundleAssets = AssetBundle<TextureBundle, Vec<Option<TextureBundle>>>;

impl AssetBundle<TextureBundle, Vec<Option<TextureBundle>>> {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            allocator: HandleAllocator::new(),
        }
    }
}
