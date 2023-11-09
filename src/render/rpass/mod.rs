mod blinn_phong;
mod clear;
mod wireframe;

use crate::{core::FxHashMap, render::RenderTarget, scene::Scene};
pub use clear::*;
pub use wireframe::*;

/// Key used to identify a pipeline with a specific configuration.
///
/// The first 3 bits are used to identify the primitive topology.
/// The next 2 bits are used to identify the polygon mode.
///
///      [0..3]           [3..5]
/// Primitive topology    Polygon mode
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

    /// Returns the primitive topology.
    pub fn topology(&self) -> wgpu::PrimitiveTopology {
        match (self.0 >> 61) & 0b111 {
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
        match (self.0 >> 59) & 0b11 {
            0 => wgpu::PolygonMode::Fill,
            1 => wgpu::PolygonMode::Line,
            2 => wgpu::PolygonMode::Point,
            _ => unreachable!(),
        }
    }
}

pub struct PipelineIdBuilder {
    topology: wgpu::PrimitiveTopology,
    polygon_mode: wgpu::PolygonMode,
}

impl Default for PipelineIdBuilder {
    fn default() -> Self {
        Self {
            topology: wgpu::PrimitiveTopology::TriangleList,
            polygon_mode: wgpu::PolygonMode::Fill,
        }
    }
}

impl PipelineIdBuilder {
    pub fn build(self) -> PipelineId {
        PipelineId((self.topology as u64) << 61 | (self.polygon_mode as u64) << 59 | 0u64)
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

pub trait RenderingPass {
    /// Records the pass.
    fn record(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &RenderTarget,
        scene: &Scene,
    );
}

mod tests {
    use super::*;

    #[test]
    fn test_pipeline_id() {
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
                let id = PipelineIdBuilder::default()
                    .with_topology(topology)
                    .with_polygon_mode(polygon_mode)
                    .build();
                assert_eq!(id.topology(), topology);
                assert_eq!(id.polygon_mode(), polygon_mode);
            }
        }
    }
}
