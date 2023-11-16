use crate::core::FxHashMap;

/// Pipeline kind.
#[pyo3::pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineKind {
    /// Render pipeline.
    Render = 0x00,
    /// Compute pipeline.
    Compute = 0x01,
}

/// Key used to identify a pipeline with a specific configuration.
///
/// The first 1 bit is used to identify the pipeline kind (0 = render, 1 =
/// compute). The next 3 bits are used to identify the primitive topology.
///
///
/// [0..1]       [1..4]                [4..6]
/// PipelineType Primitive topology    Polygon mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineId(u64);

impl PipelineId {
    /// Creates a new pipeline key with invalid configuration.
    pub fn new() -> Self {
        Self(u64::MAX)
    }

    /// Returns whether the pipeline key is invalid.
    pub fn is_invalid(&self) -> bool {
        self.0 == u64::MAX
    }

    pub fn builder() -> PipelineIdBuilder {
        PipelineIdBuilder::default()
    }

    /// Returns the pipeline kind.
    pub fn kind(&self) -> PipelineKind {
        match (self.0 >> 63) & 0b1 {
            0 => PipelineKind::Render,
            1 => PipelineKind::Compute,
            _ => unreachable!(),
        }
    }

    /// Returns the primitive topology.
    pub fn topology(&self) -> wgpu::PrimitiveTopology {
        match (self.0 >> 60) & 0b111 {
            0 => wgpu::PrimitiveTopology::PointList,
            1 => wgpu::PrimitiveTopology::LineList,
            2 => wgpu::PrimitiveTopology::LineStrip,
            3 => wgpu::PrimitiveTopology::TriangleList,
            4 => wgpu::PrimitiveTopology::TriangleStrip,
            _ => unreachable!(),
        }
    }

    /// Returns the polygon mode.
    pub fn polygon_mode(&self) -> wgpu::PolygonMode {
        match (self.0 >> 58) & 0b11 {
            0 => wgpu::PolygonMode::Fill,
            1 => wgpu::PolygonMode::Line,
            2 => wgpu::PolygonMode::Point,
            _ => unreachable!(),
        }
    }
}

pub struct PipelineIdBuilder {
    kind: PipelineKind,
    topology: wgpu::PrimitiveTopology,
    polygon_mode: wgpu::PolygonMode,
}

impl Default for PipelineIdBuilder {
    fn default() -> Self {
        Self {
            kind: PipelineKind::Render,
            topology: wgpu::PrimitiveTopology::TriangleList,
            polygon_mode: wgpu::PolygonMode::Fill,
        }
    }
}

impl PipelineIdBuilder {
    pub fn build(self) -> PipelineId {
        PipelineId(
            (self.kind as u64) << 63
                | (self.topology as u64) << 60
                | (self.polygon_mode as u64) << 58
                | 0u64,
        )
    }

    pub fn with_kind(mut self, kind: PipelineKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn with_topology(mut self, topology: wgpu::PrimitiveTopology) -> Self {
        self.topology = topology;
        self
    }

    pub fn with_polygon_mode(mut self, polygon_mode: wgpu::PolygonMode) -> Self {
        self.polygon_mode = polygon_mode;
        self
    }
}

/// A collection of pipelines.
pub struct Pipelines(pub(crate) FxHashMap<PipelineId, wgpu::RenderPipeline>);

impl Default for Pipelines {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipelines {
    fn create_render_pipeline(&mut self) -> wgpu::RenderPipeline {
        todo!()
    }
}

impl Pipelines {
    /// Creates a new collection of pipelines.
    pub fn new() -> Self {
        Self(FxHashMap::default())
    }

    /// Returns the pipeline for the given key.
    pub fn get(&self, key: PipelineId) -> Option<&wgpu::RenderPipeline> {
        self.0.get(&key)
    }

    /// Creates a new pipeline for the given key.
    pub fn create(&mut self, _key: PipelineId) {}
}

mod tests {
    #[test]
    fn test_pipeline_id() {
        use crate::render::{PipelineId, PipelineKind};

        for topology in [
            wgpu::PrimitiveTopology::PointList,
            wgpu::PrimitiveTopology::LineList,
            wgpu::PrimitiveTopology::LineStrip,
            wgpu::PrimitiveTopology::TriangleList,
            wgpu::PrimitiveTopology::TriangleStrip,
        ] {
            for polygon_mode in [
                wgpu::PolygonMode::Fill,
                wgpu::PolygonMode::Line,
                wgpu::PolygonMode::Point,
            ] {
                for kind in [PipelineKind::Render, PipelineKind::Compute] {
                    let id = PipelineId::builder()
                        .with_topology(topology)
                        .with_polygon_mode(polygon_mode)
                        .with_kind(kind)
                        .build();
                    assert_eq!(id.topology(), topology);
                    assert_eq!(id.polygon_mode(), polygon_mode);
                    assert_eq!(id.kind(), kind);
                }
            }
        }
    }
}
