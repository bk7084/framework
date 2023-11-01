use crate::typedefs::ArrVec;
use std::sync::Arc;
use wgpu::{DeviceType, Limits};

/// Aggregates all the objects needed to use the GPU.
#[derive(Clone)]
pub struct GPUContext {
    /// Instance of the graphics API.
    pub instance: Arc<wgpu::Instance>,
    /// Physical device used to render.
    pub adapter: Arc<wgpu::Adapter>,
    /// Logical device used to render.
    pub device: Arc<wgpu::Device>,
    /// Command queue used to send commands to the GPU.
    pub queue: Arc<wgpu::Queue>,

    /// Features supported by the device.
    pub features: wgpu::Features,
    /// Limits of the device.
    pub limits: wgpu::Limits,
}

/// Potential adapter to use.
struct PotentialAdapter {
    adapter: wgpu::Adapter,
    info: wgpu::AdapterInfo,
    limits: wgpu::Limits,
    features: wgpu::Features,
}

impl GPUContext {
    /// Creates a new GPU context.
    pub async fn new(desired_features: Option<wgpu::Features>) -> Self {
        profiling::scope!("GPUContext::new");
        let backends = wgpu::Backends::VULKAN | wgpu::Backends::METAL | wgpu::Backends::DX12;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            flags: wgpu::InstanceFlags::from_build_config(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: Default::default(),
        });

        let mut adapters = ArrVec::<PotentialAdapter, 4>::new();
        for adapter in instance.enumerate_adapters(backends) {
            let limits = adapter.limits();
            let features = adapter.features();
            let info = adapter.get_info();
            log::info!("{:?} Adapter: {:#?}", backends, info);
            adapters.push(PotentialAdapter {
                adapter,
                info,
                limits,
                features,
            });
        }
        adapters.sort_by_key(|adapter| match adapter.info.device_type {
            DeviceType::DiscreteGpu => 0,
            DeviceType::IntegratedGpu => 1,
            DeviceType::VirtualGpu => 2,
            DeviceType::Cpu => 3,
            DeviceType::Other => 4,
        });

        let adapter = adapters.remove(0);

        let features = adapter
            .features
            .union(desired_features.unwrap_or_else(wgpu::Features::empty));
        let limits = adapter.limits;

        // Create the GPU device and queue.
        let (device, queue) = adapter
            .adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("BK7084RS GPU Logical Device"),
                    features,
                    limits: limits.clone(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        GPUContext {
            instance: Arc::new(instance),
            adapter: Arc::new(adapter.adapter),
            device: Arc::new(device),
            queue: Arc::new(queue),
            features,
            limits,
        }
    }
}
