use crate::core::{FxHashMap, SmlString};

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
/// - [0]: the pipeline kind (0 = render, 1 = compute).
/// - [1..4]: primitive topology.
/// - [4..6]: polygon mode (0 = fill, 1 = line, 2 = point).
/// - [6..8]: cull mode (0 = front, 1 = back, 2 = none).
///
///
/// [0..1]       [1..4]                [4..6]         [6..8]
/// PipelineType Primitive topology    Polygon mode   Cull mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PipelineId(u64);

impl Default for PipelineId {
    fn default() -> Self {
        Self::new()
    }
}

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

    /// Returns the cull mode.
    pub fn cull_mode(&self) -> Option<wgpu::Face> {
        match (self.0 >> 56) & 0b11 {
            0 => Some(wgpu::Face::Front),
            1 => Some(wgpu::Face::Back),
            2 => None,
            _ => unreachable!(),
        }
    }

    pub fn from_states(
        kind: PipelineKind,
        topology: wgpu::PrimitiveTopology,
        polygon_mode: wgpu::PolygonMode,
        cull_mode: Option<wgpu::Face>,
    ) -> Self {
        PipelineIdBuilder::default()
            .with_kind(kind)
            .with_topology(topology)
            .with_polygon_mode(polygon_mode)
            .with_cull_mode(cull_mode)
            .build()
    }
}

pub struct PipelineIdBuilder {
    kind: PipelineKind,
    topology: wgpu::PrimitiveTopology,
    polygon_mode: wgpu::PolygonMode,
    cull_mode: Option<wgpu::Face>,
}

impl Default for PipelineIdBuilder {
    fn default() -> Self {
        Self {
            kind: PipelineKind::Render,
            topology: wgpu::PrimitiveTopology::TriangleList,
            polygon_mode: wgpu::PolygonMode::Fill,
            cull_mode: None,
        }
    }
}

impl PipelineIdBuilder {
    pub fn build(self) -> PipelineId {
        PipelineId(
            (self.kind as u64) << 63
                | (self.topology as u64) << 60
                | (self.polygon_mode as u64) << 58
                | (self.cull_mode.map_or(2, |c| match c {
                    wgpu::Face::Front => 0,
                    wgpu::Face::Back => 1,
                }) as u64)
                    << 56
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

    pub fn with_cull_mode(mut self, cull_mode: Option<wgpu::Face>) -> Self {
        self.cull_mode = cull_mode;
        self
    }
}

/// A collection of pipelines.
pub struct Pipelines(pub(crate) FxHashMap<SmlString, Vec<(PipelineId, wgpu::RenderPipeline)>>);

impl Default for Pipelines {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipelines {
    /// Creates a new collection of pipelines.
    pub fn new() -> Self {
        Self(FxHashMap::default())
    }

    /// Returns the pipeline with the given label and key.
    pub fn get(&self, label: &str, key: PipelineId) -> Option<&wgpu::RenderPipeline> {
        let pipelines = self.0.get(label)?;
        let index = pipelines.binary_search_by_key(&key, |(k, _)| *k).ok()?;
        Some(&pipelines[index].1)
    }

    pub fn get_by_label(&self, label: &str) -> Option<&Vec<(PipelineId, wgpu::RenderPipeline)>> {
        self.0.get(label)
    }

    /// Returns all pipelines with the given label.
    pub fn get_all(&self, label: &str) -> Option<&Vec<(PipelineId, wgpu::RenderPipeline)>> {
        self.0.get(label)
    }

    /// Returns pipelines with the given label and satisfying the given
    /// predicate.
    pub fn get_all_filtered(
        &self,
        label: &str,
        predicate: impl Fn(&PipelineId) -> bool,
    ) -> Option<Vec<&wgpu::RenderPipeline>> {
        let pipelines = self.0.get(label)?;
        Some(
            pipelines
                .iter()
                .filter(|(k, _)| predicate(k))
                .map(|(_, p)| p)
                .collect(),
        )
    }

    pub fn insert(&mut self, label: &str, key: PipelineId, pipeline: wgpu::RenderPipeline) {
        let pipelines = self.0.entry(label.into()).or_default();
        let index = pipelines.binary_search_by_key(&key, |(k, _)| *k);
        match index {
            Ok(index) => pipelines[index] = (key, pipeline),
            Err(index) => pipelines.insert(index, (key, pipeline)),
        }
        pipelines.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
    }
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
                    for cull_mode in [Some(wgpu::Face::Front), Some(wgpu::Face::Back), None] {
                        let id = PipelineId::builder()
                            .with_topology(topology)
                            .with_polygon_mode(polygon_mode)
                            .with_kind(kind)
                            .with_cull_mode(cull_mode)
                            .build();
                        assert_eq!(id.topology(), topology);
                        assert_eq!(id.polygon_mode(), polygon_mode);
                        assert_eq!(id.kind(), kind);
                        assert_eq!(id.cull_mode(), cull_mode);
                    }
                }
            }
        }
    }
}
