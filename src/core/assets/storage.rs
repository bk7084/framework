use crate::core::{
    assets::{AssetStorage, Handle},
    mesh::{GpuMesh, Mesh},
};
use range_alloc::RangeAllocator;
use std::{num::NonZeroU64, ops::Range, sync::Arc};

/// Initial size of the mesh data buffer. 32MB.
pub const INITIAL_MESH_DATA_SIZE: u64 = 1 << 25;

/// Storage for GPU meshes in a megabuffer.
///
/// This manages the allocation of mesh data on the GPU.
pub struct GpuMeshStorage {
    pub(crate) buffer: Arc<wgpu::Buffer>,
    allocator: RangeAllocator<u64>,
    pub(crate) data: Vec<Option<(Handle<GpuMesh>, GpuMesh)>>,
}

impl GpuMeshStorage {
    pub fn new(device: &wgpu::Device) -> Self {
        profiling::scope!("GpuMeshStorage::new");
        let buffer = create_gpu_mesh_storage_buffer(device, INITIAL_MESH_DATA_SIZE);
        let allocator = RangeAllocator::new(0..INITIAL_MESH_DATA_SIZE);

        Self {
            buffer,
            allocator,
            data: Vec::new(),
        }
    }

    pub fn add(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        mesh: &Mesh,
    ) -> GpuMesh {
        profiling::scope!("GpuMeshStorage::add");
        let index_count = mesh.indices.as_ref().map(|i| i.len()).unwrap_or(0);
        let vertex_count = mesh.attributes.vertex_count();

        if index_count == 0 && vertex_count == 0 {
            return GpuMesh::empty(mesh.topology);
        }

        let mut vertex_attribute_ranges = Vec::with_capacity(mesh.attributes.0.len());
        // Vertex attributes are stored separately in the buffer.
        for (attrib, data) in &mesh.attributes.0 {
            log::trace!(
                "-- Allocating vertex attribute {:?}, n_bytes: {}",
                attrib.name,
                data.n_bytes()
            );
            let range = self.allocate_range(device, encoder, data.n_bytes() as u64);
            log::debug!(
                "Allocating vertex attribute {:?} with size {} as range {:?}",
                attrib,
                data.n_bytes(),
                range
            );
            vertex_attribute_ranges.push((*attrib, range));
        }

        // Copy the mesh vertex data into the buffer.
        for (attrib, range) in vertex_attribute_ranges.iter() {
            let data = mesh.attributes.0.get(attrib).unwrap();
            let mut mapping = queue
                .write_buffer_with(
                    &self.buffer,
                    range.start,
                    NonZeroU64::new(data.n_bytes() as u64).unwrap(),
                )
                .unwrap();
            mapping.copy_from_slice(data.as_bytes());
        }

        let (index_format, index_range) = match mesh.indices.as_ref() {
            None => {
                // No indices, so we don't need to allocate an index buffer.
                (None, 0..0)
            }
            Some(indices) => {
                // Make sure the size of the indices is aligned to COPY_BUFFER_ALIGNMENT.
                let n_bytes = (indices.n_bytes() as u64 + wgpu::COPY_BUFFER_ALIGNMENT - 1)
                    & !(wgpu::COPY_BUFFER_ALIGNMENT - 1);
                let index_range = self.allocate_range(device, encoder, n_bytes);
                log::debug!(
                    "Allocating index buffer with size {} as range {:?} with padding {}",
                    indices.n_bytes(),
                    index_range,
                    n_bytes - indices.n_bytes() as u64
                );
                // Copy the mesh index data into the buffer.
                let mut mapping = queue
                    .write_buffer_with(
                        &self.buffer,
                        index_range.start,
                        NonZeroU64::new(n_bytes).unwrap(),
                    )
                    .unwrap();
                mapping[..indices.n_bytes()].copy_from_slice(indices.as_bytes());
                (Some(indices.format()), index_range)
            }
        };

        GpuMesh {
            mesh_id: mesh.id,
            mesh_path: mesh.path.clone(),
            topology: mesh.topology,
            vertex_attribute_ranges,
            vertex_count: vertex_count as u32,
            index_format,
            index_range,
            index_count: index_count as u32,
            sub_meshes: mesh.sub_meshes.clone(),
        }
    }
}

impl GpuMeshStorage {
    /// Allocates a range of the given size from the buffer.
    ///
    /// If the buffer is too small, it will be grown.
    fn allocate_range(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        n_bytes: u64,
    ) -> Range<u64> {
        log::trace!("Allocating {} bytes from mesh buffer", n_bytes);
        match self.allocator.allocate_range(n_bytes) {
            Ok(range) => range,
            Err(..) => {
                log::trace!(
                    "Buffer is too small ({}), growing...",
                    self.allocator.total_available()
                );
                // Desired allocation is too large, so we need to grow the buffer.
                self.grow_buffer(device, encoder, n_bytes);
                self.allocator.allocate_range(n_bytes).unwrap()
            }
        }
    }

    /// Deallocates a range of the given size from the buffer.
    fn deallocate_range(&mut self, range: Range<u64>) {
        if range.is_empty() {
            return;
        }
        self.allocator.free_range(range);
    }

    fn grow_buffer(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        desired: u64,
    ) {
        profiling::scope!("GpuMeshStorage::grow_buffers");

        let n_bytes = self
            .allocator
            .initial_range()
            .end
            .checked_add(desired)
            .unwrap()
            .next_power_of_two();

        let new_buffer = create_gpu_mesh_storage_buffer(device, n_bytes);

        // Copy the old buffer into the new buffer.
        encoder.copy_buffer_to_buffer(
            &self.buffer,
            0,
            &new_buffer,
            0,
            self.allocator.initial_range().end,
        );
        self.buffer = new_buffer;
        self.allocator.grow_to(n_bytes);
    }
}

impl AssetStorage for GpuMeshStorage {
    fn len(&self) -> usize {
        self.data.len()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

fn create_gpu_mesh_storage_buffer(device: &wgpu::Device, n_bytes: u64) -> Arc<wgpu::Buffer> {
    Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mesh_data_buffer"),
        size: n_bytes,
        usage: wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::INDEX
            | wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::VERTEX,
        mapped_at_creation: false,
    }))
}

pub struct MaterialStorage {
    // TODO: implement
}

impl MaterialStorage {
    pub fn new(_device: &wgpu::Device) -> Self {
        // TODO: implement
        Self {}
    }
}
