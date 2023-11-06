mod handle;
pub mod storage;

use crate::core::{assets::storage::GpuMeshStorage, mesh::GpuMesh, Material};
pub use handle::*;
use std::{
    fmt::Debug,
    ops::Index,
    sync::{Arc, RwLock},
};

/// Trait for representing an asset.
pub trait Asset: Send + Sync {}

/// A collection of assets of the same type.
pub struct Assets<A: Asset, S> {
    storage: Arc<RwLock<S>>,
    allocator: HandleAllocator<A>,
}

impl<A: Asset, S> Default for Assets<A, S>
where
    S: Default,
{
    fn default() -> Self {
        Self {
            storage: Arc::new(RwLock::new(S::default())),
            allocator: HandleAllocator::new(),
        }
    }
}

impl<A: Asset> Assets<A, Vec<Option<A>>> {
    /// Returns the number of assets in the storage.
    pub fn len(&self) -> usize {
        self.storage.read().unwrap().len()
    }

    /// Returns `true` if the asset storage is empty.
    pub fn is_empty(&self) -> bool {
        self.storage.read().unwrap().is_empty()
    }

    /// Adds a new asset to the storage and returns its handle.
    pub fn add(&mut self, asset: A) -> Handle<A> {
        let handle = self.allocator.reserve();
        self.flush();
        self.storage.write().unwrap()[handle.index as usize] = Some(asset);
        handle
    }

    /// Inserts a new asset into the storage at the given index.
    ///
    /// Returns true if the asset was inserted.
    pub fn insert(&mut self, handle: Handle<A>, asset: A) -> Option<A> {
        self.flush();
        self.storage.write().unwrap()[handle.index as usize].replace(asset)
    }

    /// Removes an asset from the storage at the given index and returns it.
    pub fn remove(&mut self, handle: Handle<A>) -> Option<A> {
        self.flush();
        match self.storage.write().unwrap()[handle.index as usize].take() {
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
        let mut storage = self.storage.write().unwrap();
        let new_len = self
            .allocator
            .next_index
            .load(std::sync::atomic::Ordering::Relaxed) as usize;
        storage.resize_with(new_len, || None);
        while let Ok(recycled) = self.allocator.recycle_receiver.try_recv() {
            storage[recycled.index as usize] = None;
        }
    }
}

pub type MeshAssets = Assets<GpuMesh, GpuMeshStorage>;

// Specialize the `Assets` type for `GpuMesh` as it needs a custom storage.
impl Assets<GpuMesh, GpuMeshStorage> {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            storage: Arc::new(RwLock::new(GpuMeshStorage::new(device))),
            allocator: HandleAllocator::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.storage.read().unwrap().len()
    }
}

pub type MaterialAssets = Assets<Material, Vec<Option<Material>>>;

impl Assets<Material, Vec<Option<Material>>> {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            storage: Arc::new(RwLock::new(Vec::new())),
            allocator: HandleAllocator::new(),
        }
    }
}
