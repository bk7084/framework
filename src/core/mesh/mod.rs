use glam::{Vec3, Vec4};
use rustc_hash::FxHashMap;
use std::{
    fmt::Debug,
    ops::Range,
    path::{Path, PathBuf},
    sync::atomic::AtomicU64,
};

mod attribute;

#[path = "mesh_py.rs"]
pub mod py;

use crate::core::{
    assets::{Asset, Handle},
    Alignment, Material, MaterialBundle, SmlString, TextureBundle,
};
pub use attribute::*;

use super::Color;

pub trait IndexType: Copy + Debug {
    fn as_u32(&self) -> u32;
    fn as_usize(&self) -> usize;
}

impl IndexType for u32 {
    fn as_u32(&self) -> u32 {
        *self
    }

    fn as_usize(&self) -> usize {
        *self as usize
    }
}

impl IndexType for u16 {
    fn as_u32(&self) -> u32 {
        *self as u32
    }

    fn as_usize(&self) -> usize {
        *self as usize
    }
}

/// Indices of a mesh.
#[derive(Clone, Debug)]
pub enum Indices {
    U32(Vec<u32>),
    U16(Vec<u16>),
}

impl Indices {
    /// Returns the number of indices in the index buffer.
    pub fn len(&self) -> usize {
        match self {
            Self::U32(indices) => indices.len(),
            Self::U16(indices) => indices.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of bytes required to store the index buffer.
    pub fn n_bytes(&self) -> usize {
        match self {
            Self::U32(indices) => indices.len() * std::mem::size_of::<u32>(),
            Self::U16(indices) => indices.len() * std::mem::size_of::<u16>(),
        }
    }

    /// Returns the index buffer as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::U32(indices) => bytemuck::cast_slice(indices),
            Self::U16(indices) => bytemuck::cast_slice(indices),
        }
    }

    /// Returns the index buffer format as a wgpu::IndexFormat.
    pub fn format(&self) -> wgpu::IndexFormat {
        match self {
            Self::U32(_) => wgpu::IndexFormat::Uint32,
            Self::U16(_) => wgpu::IndexFormat::Uint16,
        }
    }
}

/// A submesh is a range of indices, it specifies a range of indices to be
/// rendered with a specific material.
#[pyo3::pyclass]
#[derive(Clone, Debug)]
pub struct SubMesh {
    /// Range of indices/vertices of the submesh.
    pub range: Range<u32>,
    /// Material of the submesh (index into the material array of the mesh).
    /// If the material is None, the submesh uses the default material.
    pub material: Option<u32>,
}

#[pyo3::pymethods]
impl SubMesh {
    /// Creates a new submesh from a range of indices of triangles.
    #[new]
    pub fn new_py(start: u32, end: u32, index: u32) -> Self {
        Self {
            range: start * 3..end * 3,
            material: Some(index),
        }
    }
}

impl SubMesh {
    /// Creates a new submesh.
    ///
    /// Note: the range is in number of indices, not vertices.
    ///
    /// # Arguments
    ///
    /// * `start` - The start index of the submesh.
    /// * `end` - The end index of the submesh.
    /// * `index` - The material index of the submesh (index into the material
    /// array of the mesh).
    pub fn new(start: u32, end: u32, index: u32) -> Self {
        Self {
            range: start..end,
            material: Some(index),
        }
    }

    /// Returns the number of indices in the submesh.
    pub fn len(&self) -> usize {
        self.range.end as usize - self.range.start as usize
    }
}

/// Mesh id counter. 0 and 1 are reserved for the default cube and quad.
static MESH_ID: AtomicU64 = AtomicU64::new(0);

/// A mesh is a collection of vertices with optional indices and materials.
/// Vertices can have different attributes such as position, normal, uv, etc.
#[pyo3::pyclass(subclass)]
#[derive(Clone)]
pub struct Mesh {
    /// Unique name of the mesh.
    pub(crate) name: SmlString,
    /// Topology of the mesh primitive.
    pub(crate) topology: wgpu::PrimitiveTopology,
    /// Vertex attributes of the mesh.
    pub(crate) attributes: VertexAttributes,
    /// Indices of the mesh.
    pub(crate) indices: Option<Indices>,
    /// Sub-meshes of the mesh. If the mesh has no sub-meshes, it is assumed
    /// that the entire mesh is using the default material.
    pub(crate) sub_meshes: Option<Vec<SubMesh>>,
    /// Path to the mesh file, if it's loaded from a file.
    pub(crate) path: Option<PathBuf>,
    /// Materials of the mesh.
    pub(crate) materials: Option<Vec<Material>>,
}

impl Asset for Mesh {}

pub struct VertexBufferLayout {
    pub array_stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<VertexAttribute>,
}

impl Mesh {
    pub fn new(topology: wgpu::PrimitiveTopology) -> Self {
        Self {
            name: SmlString::from(format!(
                "mesh_{}",
                MESH_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            )),
            topology,
            attributes: Default::default(),
            indices: None,
            sub_meshes: None,
            path: None,
            materials: None,
        }
    }

    pub fn new_with_name(name: &str, topology: wgpu::PrimitiveTopology) -> Self {
        Self {
            name: SmlString::from(name),
            topology,
            attributes: Default::default(),
            indices: None,
            sub_meshes: None,
            path: None,
            materials: None,
        }
    }

    #[rustfmt::skip]
    /// Creates a unit cube of side length 1 centered at the origin.
    pub fn cube(length: f32) -> Self {
        let mut attributes = VertexAttributes::default();
        let half = length * 0.5;
        // Vertex positions for a unit cube centered at the origin.
        let vertices: [[f32; 3]; 24] = [
            // front (0.0, 0.0, 0.5)
            [-half, -half, half], [half, -half, half], [half, half, half], [-half, half, half],
            // back (0.0, 0.0, -half)
            [-half, -half, -half], [half, -half, -half], [half, half, -half], [-half, half, -half],
            // right (half, 0.0, 0.0)
            [half, -half, -half], [half, half, -half], [half, half, half], [half, -half, half],
            // left (-half, 0.0, 0.0)
            [-half, -half, half], [-half, half, half], [-half, half, -half], [-half, -half, -half],
            // top (0.0, half, 0.0)
            [half, half, -half], [-half, half, -half], [-half, half, half], [half, half, half],
            // bottom (0.0, -half, 0.0)
            [half, -half, half], [-half, -half, half], [-half, -half, -half], [half, -half, -half],
        ];
        // Vertex normals for a unit cube centered at the origin. Per vertex normals.
        let normals: [[f32; 3]; 24] = [
            // front (0.0, 0.0, 1.0)
            [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
            // back (0.0, 0.0, -1.0)
            [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
            // right (1.0, 0.0, 0.0)
            [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
            // left (-1.0, 0.0, 0.0)
            [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0],
            // top (0.0, 1.0, 0.0)
            [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],
            // bottom (0.0, -1.0, 0.0)
            [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0],
        ];
        // Vertex indices for a unit cube centered at the origin.
        let indices: Vec<u16> = vec![
            0, 1, 2, 2, 3, 0, // front
            4, 7, 6, 4, 6, 5, // 6, 5, 4, // back
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // top
            20, 21, 22, 22, 23, 20, // bottom */
        ];
        // Vertex UVs for a unit cube centered at the origin.
        let uvs: Vec<[f32; 2]> = vec![
            // front
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // back
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // right
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // left
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // top
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // bottom
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        ];
        attributes.insert(VertexAttribute::POSITION, AttribContainer::new(&vertices));
        attributes.insert(VertexAttribute::NORMAL, AttribContainer::new(&normals));
        attributes.insert(VertexAttribute::UV, AttribContainer::new(&uvs));
        let mut mesh = Mesh::new(wgpu::PrimitiveTopology::TriangleList);
        mesh.attributes = attributes;
        mesh.indices = Some(Indices::U16(indices));
        mesh.compute_tangents();
        mesh
    }

    #[rustfmt::skip]
    /// Creates a unit quad of side length 1 centered at the origin.
    pub fn plane(length: f32, align: Alignment) -> Self {
        let mut attributes = VertexAttributes::default();
        let half = length * 0.5;
        let (vertices, normals) = match align {
            Alignment::XY => {
                let vertices: [[f32; 3]; 4] = [
                    [-half, -half, 0.0], [half, -half, 0.0], [half, half, 0.0], [-half, half, 0.0],
                ];
                let normals: [[f32; 3]; 4] = [
                    [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
                ];
                (vertices, normals)
            }
            Alignment::XZ => {
                let vertices: [[f32; 3]; 4] = [
                    [-half, 0.0, -half], [half, 0.0, -half], [half, 0.0, half], [-half, 0.0, half],
                ];
                let normals: [[f32; 3]; 4] = [
                    [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],
                ];
                (vertices, normals)
            }
            Alignment::YZ => {
                let vertices: [[f32; 3]; 4] = [
                    [0.0, -half, -half], [0.0, half, -half], [0.0, half, half], [0.0, -half, half],
                ];
                let normals: [[f32; 3]; 4] = [
                    [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
                ];
                (vertices, normals)
            }
        };
        // Vertex indices for a unit quad centered at the origin.
        let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];
        // Vertex UVs for a unit quad centered at the origin.
        let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        attributes.insert(VertexAttribute::POSITION, AttribContainer::new(&vertices));
        attributes.insert(VertexAttribute::NORMAL, AttribContainer::new(&normals));
        attributes.insert(VertexAttribute::UV, AttribContainer::new(&uvs));
        let mut mesh = Mesh::new(wgpu::PrimitiveTopology::TriangleList);
        mesh.attributes = attributes;
        mesh.indices = Some(Indices::U16(indices));
        mesh.compute_tangents();
        mesh
    }

    /// Creates a grid of size (w, h) with spacing (sx, sy).
    pub fn grid(w: f32, h: f32, spacing: (f32, f32), align: Alignment, _color: Color) -> Self {
        let mut attributes = VertexAttributes::default();
        let (sx, sy) = spacing;
        // Generate the vertices of lines in the grid.
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        // TODO: Compute tangents.
        let tangents = [[0.0f32; 4]; 4];
        // Note: created to satisfy the mesh validation.
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let n_w = (w / sx).ceil() as usize;
        let n_h = (h / sy).ceil() as usize;
        let half_w = w * 0.5;
        let half_h = h * 0.5;
        for i in 0..=n_w {
            let a = -half_w + i as f32 * sx;
            match align {
                Alignment::XY => {
                    vertices.push([a, -half_h, 0.0f32]);
                    vertices.push([a, half_h, 0.0]);
                    normals.push([0.0f32, 0.0, 1.0]);
                }
                Alignment::XZ => {
                    vertices.push([a, 0.0, -half_h]);
                    vertices.push([a, 0.0, half_h]);
                    normals.push([0.0, 1.0, 0.0]);
                }
                Alignment::YZ => {
                    vertices.push([0.0, a, -half_h]);
                    vertices.push([0.0, a, half_h]);
                    normals.push([1.0, 0.0, 0.0]);
                }
            }
        }
        for i in 0..=n_h {
            let b = -half_h + i as f32 * sy;
            match align {
                Alignment::XY => {
                    vertices.push([-half_w, b, 0.0]);
                    vertices.push([half_w, b, 0.0]);
                }
                Alignment::XZ => {
                    vertices.push([-half_w, 0.0, b]);
                    vertices.push([half_w, 0.0, b]);
                }
                Alignment::YZ => {
                    vertices.push([0.0, -half_w, b]);
                    vertices.push([0.0, half_w, b]);
                }
            }
        }
        for i in 0..vertices.len() {
            indices.push(i as u16);
            uvs.push([0.0, 0.0]);
        }
        attributes.insert(VertexAttribute::POSITION, AttribContainer::new(&vertices));
        attributes.insert(VertexAttribute::NORMAL, AttribContainer::new(&normals));
        attributes.insert(VertexAttribute::UV, AttribContainer::new(&uvs));
        attributes.insert(VertexAttribute::TANGENT, AttribContainer::new(&tangents));
        let mut mesh = Mesh::new(wgpu::PrimitiveTopology::LineList);
        mesh.name = SmlString::from("bkfw_inner_grid");
        mesh.attributes = attributes;
        mesh.indices = Some(Indices::U16(indices));
        let mut mat = Material::new();
        mat.name = SmlString::from("bkfw_inner_grid");
        // mat.diffuse = Some([color.r as f32, color.g as f32, color.b as f32]);
        mat.diffuse = Some([1.0, 0.0, 0.0]);
        mat.illumination_model = Some(11);
        mesh.set_material(mat);
        mesh
    }

    /// Creates a sphere of centered at the origin.
    ///
    /// # Arguments
    ///
    /// * `radius` - Radius of the sphere.
    /// * `segments` - Number of segments around the sphere.
    /// * `rings` - Number of rings from the top to the bottom of the sphere.
    pub fn sphere(radius: f32, segments: u32, rings: u32) -> Mesh {
        let mut attributes = VertexAttributes::default();
        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices = Vec::new();

        // Create the vertices.
        for ring in 0..=rings {
            let v = ring as f32 / rings as f32;
            let theta = v * std::f32::consts::PI;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for segment in 0..=segments {
                let u = segment as f32 / segments as f32;
                let phi = u * std::f32::consts::PI * 2.0;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let x = cos_phi * sin_theta;
                let y = cos_theta;
                let z = sin_phi * sin_theta;

                vertices.push([radius * x, radius * y, radius * z]);
                normals.push([x, y, z]);
                uvs.push([u, v]);
            }
        }

        // Create the indices.
        for ring in 0..rings {
            for segment in 0..segments {
                let next_segment = segment + 1;
                let next_ring = ring + 1;

                indices.push(ring * (segments + 1) + segment);
                indices.push(next_ring * (segments + 1) + next_segment);
                indices.push(next_ring * (segments + 1) + segment);

                indices.push(ring * (segments + 1) + segment);
                indices.push(ring * (segments + 1) + next_segment);
                indices.push(next_ring * (segments + 1) + next_segment);
            }
        }

        attributes.insert(VertexAttribute::POSITION, AttribContainer::new(&vertices));
        attributes.insert(VertexAttribute::NORMAL, AttribContainer::new(&normals));
        attributes.insert(VertexAttribute::UV, AttribContainer::new(&uvs));
        let mut mesh = Mesh::new(wgpu::PrimitiveTopology::TriangleList);
        mesh.attributes = attributes;
        mesh.indices = Some(Indices::U32(indices));
        mesh.compute_tangents();
        mesh
    }

    /// Creates a triangle with user defined vertices.
    pub fn triangle(vertices: &[Vec3]) -> Mesh {
        assert_eq!(vertices.len(), 3, "Triangle must have 3 vertices.");
        let mut attributes = VertexAttributes::default();

        // Create the normals.
        let v0 = vertices[0];
        let v1 = vertices[1];
        let v2 = vertices[2];
        let u = v1 - v0;
        let v = v2 - v0;
        let normal = u.cross(v).normalize();

        let normals: Vec<[f32; 3]> = vec![normal.into(); 3];
        let vertices: Vec<[f32; 3]> = vertices.iter().map(|v| (*v).into()).collect();
        let indices = vec![0u32, 1, 2];
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];

        attributes.insert(VertexAttribute::POSITION, AttribContainer::new(&vertices));
        attributes.insert(VertexAttribute::NORMAL, AttribContainer::new(&normals));
        attributes.insert(VertexAttribute::UV, AttribContainer::new(&uvs));
        let mut mesh = Mesh::new(wgpu::PrimitiveTopology::TriangleList);
        mesh.attributes = attributes;
        mesh.indices = Some(Indices::U32(indices));
        mesh.compute_tangents();
        mesh
    }

    /// Validates the mesh.
    ///
    /// A mesh is valid if it has a position attribute, uv attribute, and
    /// indices. If the mesh has not normals, they are computed.
    pub fn validate(&mut self) {
        log::info!("Validating mesh: {}.", self.name);
        for attr in [VertexAttribute::POSITION, VertexAttribute::UV] {
            if !self.attributes.0.contains_key(&attr) {
                panic!("Mesh must have a {:?} attribute.", attr);
            }
        }
        if self.indices.is_none() {
            panic!("Mesh must have indices.");
        }
        if !self.attributes.0.contains_key(&VertexAttribute::NORMAL) {
            log::warn!("Mesh has no normals. Computing normals.");
            self.compute_normals();
        }
        if !self.attributes.0.contains_key(&VertexAttribute::TANGENT) {
            log::warn!("Mesh has no tangents. Computing tangents.");
            self.compute_tangents();
        }
    }

    /// Loads a mesh from a wavefront obj file.
    pub fn load_from_obj<P: AsRef<Path> + Debug + Copy>(path: P) -> Self {
        log::debug!("Loading mesh from {}.", path.as_ref().display());
        let options = tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ignore_points: true,
            ignore_lines: true,
        };
        let (models, materials) = tobj::load_obj(path, &options)
            .map_err(|err| {
                log::error!("Failed to load mesh from {:?}: {}", path, err);
            })
            .unwrap();
        let materials = materials.expect("Failed to load materials.");
        log::debug!("- Loaded {} models.", models.len());
        log::debug!("- Loaded {} materials.", materials.len());
        log::debug!("-- Loaded materials: {:?}", materials);
        let mut attributes = VertexAttributes::default();
        let mut vertices: Vec<f32> = Vec::new();
        let mut normals: Vec<f32> = Vec::new();
        let mut uvs: Vec<f32> = Vec::new();
        let mut indices = Vec::new();

        // Classify the submeshes by material.
        let mut sub_meshes = Vec::new();
        let mut sub_meshes_by_material = FxHashMap::default();
        for model in models.iter() {
            let mesh = &model.mesh;
            sub_meshes_by_material
                .entry(mesh.material_id)
                .or_insert_with(Vec::new)
                .push(mesh);
        }

        let mut index_start = 0;
        for (material_id, meshes) in sub_meshes_by_material.iter() {
            let mut sub_mesh = SubMesh {
                range: index_start..index_start,
                material: material_id.map(|id| id as u32),
            };
            for mesh in meshes.iter() {
                let mut mesh_indices = mesh.indices.clone();
                let index_offset = vertices.len() as u32 / 3;
                for idx in mesh_indices.iter_mut() {
                    *idx += index_offset;
                }
                let index_count = mesh_indices.len() as u32;
                vertices.append(&mut mesh.positions.clone());
                normals.append(&mut mesh.normals.clone());
                uvs.append(&mut mesh.texcoords.clone());
                indices.append(&mut mesh_indices);
                index_start += index_count;
            }
            sub_mesh.range.end = index_start;
            sub_meshes.push(sub_mesh);
        }

        attributes.insert(VertexAttribute::POSITION, AttribContainer::new(&vertices));

        if !normals.is_empty() {
            attributes.insert(VertexAttribute::NORMAL, AttribContainer::new(&normals));
        }

        if !uvs.is_empty() {
            attributes.insert(VertexAttribute::UV, AttribContainer::new(&uvs));
        }

        let id = MESH_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        log::debug!("- Loaded mesh with id: {}.", id);

        let materials = materials
            .iter()
            .map(|m| Material::from_tobj_material(m.clone(), path.as_ref()))
            .collect();

        log::debug!("- Processed materials: {:?}", materials);
        log::debug!("- Loaded submeshes: {:?}", sub_meshes);

        let mut mesh = Mesh::new(wgpu::PrimitiveTopology::TriangleList);
        mesh.name = SmlString::from(path.as_ref().file_name().unwrap().to_str().unwrap());
        mesh.attributes = attributes;
        mesh.indices = if !indices.is_empty() {
            Some(Indices::U32(indices))
        } else {
            None
        };
        mesh.sub_meshes = Some(sub_meshes);
        mesh.materials = Some(materials);
        mesh.path = Some(path.as_ref().to_path_buf());
        mesh.compute_tangents();
        mesh
    }

    /// Computes per vertex normals for the mesh.
    pub fn compute_normals(&mut self) {
        if self.attributes.0.contains_key(&VertexAttribute::NORMAL) {
            log::warn!("Mesh already has normals. Skipping normal computation.");
            return;
        }
        if self.indices.is_none() {
            panic!("Indices are required to compute the normals");
        }
        let vertices = self
            .attributes
            .0
            .get(&VertexAttribute::POSITION)
            .unwrap()
            .as_slice::<[f32; 3]>();
        let mut normals: Vec<Vec3> = vec![Vec3::ZERO; vertices.len()];
        match &self.indices {
            None => {
                panic!("Indices are required to compute the normals");
            }
            Some(indices) => match indices {
                Indices::U32(indices) => {
                    compute_normals(vertices, indices, &mut normals);
                }
                Indices::U16(indices) => {
                    compute_normals(vertices, indices, &mut normals);
                }
            },
        }
        let normals_raw: Vec<[f32; 3]> = unsafe { std::mem::transmute(normals) };
        self.attributes
            .insert(VertexAttribute::NORMAL, AttribContainer::new(&normals_raw));
        // Recompute tangents.
        self.attributes.0.remove(&VertexAttribute::TANGENT);
        self.compute_tangents();
    }

    /// Computes per vertex tangents for the mesh from the UVs.
    pub fn compute_tangents(&mut self) {
        if self.attributes.0.contains_key(&VertexAttribute::TANGENT) {
            log::warn!("Mesh already has tangents and bitangents. Skipping tangent computation.");
            return;
        }
        let vertices = self
            .attributes
            .0
            .get(&VertexAttribute::POSITION)
            .unwrap()
            .as_slice::<[f32; 3]>();
        let uvs = self
            .attributes
            .0
            .get(&VertexAttribute::UV)
            .expect("Mesh must have UVs to compute the tangents")
            .as_slice::<[f32; 2]>();
        let normals = self
            .attributes
            .0
            .get(&VertexAttribute::NORMAL)
            .expect("Mesh must have normals to compute the tangents")
            .as_slice::<[f32; 3]>();
        let mut tangents: Vec<Vec4> = vec![Vec4::ZERO; vertices.len()];
        match &self.indices {
            None => {
                panic!("Indices are required to compute the bi/tangents");
            }
            Some(indices) => match indices {
                Indices::U32(indices) => {
                    compute_tangents(vertices, indices, uvs, normals, &mut tangents);
                }
                Indices::U16(indices) => {
                    compute_tangents(vertices, indices, uvs, normals, &mut tangents)
                }
            },
        }
        let tangents_raw: Vec<[f32; 4]> = unsafe { std::mem::transmute(tangents) };
        self.attributes.insert(
            VertexAttribute::TANGENT,
            AttribContainer::new(&tangents_raw),
        );
    }
}

/// A mesh on the GPU.
pub struct GpuMesh {
    /// Name of the GpuMesh inherited from the Mesh.
    pub name: SmlString,
    /// Path to the mesh file, if it's loaded from a file.
    pub path: Option<PathBuf>,
    /// Topology of the mesh primitive.
    pub topology: wgpu::PrimitiveTopology,
    /// Vertex attributes of the mesh.
    pub vertex_attribute_ranges: Vec<(VertexAttribute, Range<u64>)>,
    /// Vertex count of the mesh.
    pub vertex_count: u32,
    /// Index buffer format of the mesh.
    pub index_format: Option<wgpu::IndexFormat>,
    /// Index buffer range inside the mesh data buffer. 0..0 if no index buffer.
    pub index_range: Range<u64>,
    /// Number of indices in the index buffer.
    pub index_count: u32,
    /// Sub-meshes of the mesh.
    pub sub_meshes: Option<Vec<SubMesh>>,
}

impl Asset for GpuMesh {}

impl GpuMesh {
    /// Creates a new empty gpu mesh.
    pub fn empty(topology: wgpu::PrimitiveTopology) -> Self {
        Self {
            name: SmlString::from("empty"),
            path: None,
            topology,
            vertex_attribute_ranges: Vec::new(),
            vertex_count: 0,
            index_format: None,
            index_range: 0..0,
            index_count: 0,
            sub_meshes: None,
        }
    }

    /// Returns the range of the vertex attribute in the mesh data buffer, if it
    /// exists.
    pub fn get_vertex_attribute_range(&self, attribute: VertexAttribute) -> Option<Range<u64>> {
        self.vertex_attribute_ranges
            .iter()
            .find_map(|(attrib, range)| (*attrib == attribute).then_some(range.clone()))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct MeshBundle {
    pub mesh: Handle<GpuMesh>,
    pub aesthetic: AestheticBundle,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct AestheticBundle {
    pub textures: Handle<TextureBundle>,
    pub materials: Handle<MaterialBundle>,
}

fn compute_tangents<T: IndexType>(
    positions: &[[f32; 3]],
    indices: &[T],
    uvs: &[[f32; 2]],
    normals: &[[f32; 3]],
    tangents: &mut [Vec4],
) {
    let mut bitangents = vec![Vec3::ZERO; positions.len()];
    for tri in indices.chunks(3) {
        let (tri0, tri1, tri2) = (tri[0].as_usize(), tri[1].as_usize(), tri[2].as_usize());
        let v0 = glam::Vec3::from(positions[tri0]);
        let v1 = glam::Vec3::from(positions[tri1]);
        let v2 = glam::Vec3::from(positions[tri2]);
        let uv0 = glam::Vec2::from(uvs[tri0]);
        let uv1 = glam::Vec2::from(uvs[tri1]);
        let uv2 = glam::Vec2::from(uvs[tri2]);

        // Calculate the edges of the triangle
        let e1 = v1 - v0;
        let e2 = v2 - v0;

        // This will give us a direction to calculate the
        // tangent and bitangent
        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        // Solving the following system of equations
        //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
        //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        let tangent = (e1 * delta_uv2.y - e2 * delta_uv1.y) * r;
        let bitangent = (-e1 * delta_uv2.x + e2 * delta_uv1.x) * r;
        tangents[tri0] = Vec4::new(
            tangent.x + tangents[tri0].x,
            tangent.y + tangents[tri0].y,
            tangent.z + tangents[tri0].z,
            0.0,
        );
        tangents[tri1] = Vec4::new(
            tangent.x + tangents[tri1].x,
            tangent.y + tangents[tri1].y,
            tangent.z + tangents[tri1].z,
            0.0,
        );
        tangents[tri2] = Vec4::new(
            tangent.x + tangents[tri2].x,
            tangent.y + tangents[tri2].y,
            tangent.z + tangents[tri2].z,
            0.0,
        );
        bitangents[tri0] += bitangent;
        bitangents[tri1] += bitangent;
        bitangents[tri2] += bitangent;
    }

    // Average the tangents and bitangents
    for i in 0..positions.len() {
        let t = tangents[i].truncate().normalize();
        let b = bitangents[i].normalize();
        let n = Vec3::from(normals[i]);
        let t_perp = t - n * t.dot(n);
        tangents[i] = Vec4::from((t_perp, n.dot(t.cross(b)).signum()));
    }
}

fn compute_normals<T: IndexType>(positions: &[[f32; 3]], indices: &[T], normals: &mut [Vec3]) {
    for tri in indices.chunks(3) {
        let (tri0, tri1, tri2) = (tri[0].as_usize(), tri[1].as_usize(), tri[2].as_usize());
        let v0 = glam::Vec3::from(positions[tri0]);
        let v1 = glam::Vec3::from(positions[tri1]);
        let v2 = glam::Vec3::from(positions[tri2]);
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        let normal = e1.cross(e2).normalize();
        normals[tri0] += normal;
        normals[tri1] += normal;
        normals[tri2] += normal;
    }
    for normal in normals.iter_mut() {
        *normal = normal.normalize();
    }
}
