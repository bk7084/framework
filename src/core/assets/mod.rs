mod handle;
pub mod storage;

use crate::core::{
    assets::storage::GpuMeshStorage,
    mesh::{GpuMesh, Mesh},
    Material,
};
pub use handle::*;
use std::{
    fmt::Debug,
    ops::Index,
    sync::{Arc, RwLock},
};

/// Trait for representing an asset.
pub trait Asset: Send + Sync {}
pub trait AssetStorage {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

/// A collection of assets of the same type.
pub struct Assets<A: Asset, S: AssetStorage> {
    storage: S,
    allocator: HandleAllocator<A>,
}

impl<A: Asset, S: AssetStorage> Assets<A, S> {
    fn len(&self) -> usize {
        self.storage.len()
    }

    fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }
}

impl<A: Asset, S: AssetStorage> Default for Assets<A, S>
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

impl<A: Asset> Assets<A, Vec<Option<A>>> {
    /// Adds a new asset to the storage and returns its handle.
    pub fn add(&mut self, asset: A) -> Handle<A> {
        let handle = self.allocator.reserve();
        self.flush();
        self.storage[handle.index as usize] = Some(asset);
        handle
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
}

pub type MeshAssets = Assets<GpuMesh, GpuMeshStorage>;

// Specialize the `Assets` type for `GpuMesh` as it needs a custom storage.
impl Assets<GpuMesh, GpuMeshStorage> {
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
    ) -> Handle<GpuMesh> {
        let handle = self.allocator.reserve();
        self.flush();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("mesh_add"),
        });

        let gpu_mesh = self.storage.add(device, queue, &mut encoder, mesh);
        self.storage.data[handle.index as usize] = Some(gpu_mesh);

        queue.submit(std::iter::once(encoder.finish()));

        handle
    }

    pub fn get(&self, handle: Handle<GpuMesh>) -> Option<&GpuMesh> {
        self.storage.data[handle.index as usize].as_ref()
    }

    pub fn remove(&mut self, handle: Handle<GpuMesh>) -> Option<GpuMesh> {
        self.flush();
        match self.storage.data[handle.index as usize].take() {
            Some(mesh) => {
                self.allocator.recycle(handle);
                Some(mesh)
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

pub type MaterialAssets = Assets<Material, Vec<Option<Material>>>;

impl Assets<Material, Vec<Option<Material>>> {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            storage: Vec::new(),
            allocator: HandleAllocator::new(),
        }
    }
}
