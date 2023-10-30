use crate::typedefs::ArrVec;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use wgpu::DeviceType;

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
}

#[derive(Clone)]
pub struct PotentialAdapter {
    pub adapter: wgpu::Adapter,
    pub info: wgpu::AdapterInfo,
    pub limits: wgpu::Limits,
    pub features: wgpu::Features,
}

/// Configuration for the GPU together with the graphics API.
pub struct GPUConfig {
    /// Physical device requirements.
    pub device_descriptor: wgpu::DeviceDescriptor<'static>,
    /// The power preference of the GPU.
    pub power_preference: wgpu::PowerPreference,
    /// Graphics API backend to use.
    pub backend: wgpu::Backends,
    /// Presentation mode to use.
    pub present_mode: wgpu::PresentMode,
    /// Surface format for the swap chain.
    pub format: wgpu::TextureFormat,
    /// Depth format for the swap chain.
    pub depth_format: wgpu::TextureFormat,
}

impl GPUConfig {
    /// Creates a new configuration trying to use the best available options.
    pub fn new(limits: &wgpu::Limits) -> Self {
        GPUConfig {
            device_descriptor: wgpu::DeviceDescriptor {
                label: Some("BK7084RS Device"),
                features: wgpu::Features::default(),
                limits: limits.clone(),
            },
            power_preference: wgpu::PowerPreference::HighPerformance,
            backend: wgpu::Backends::PRIMARY,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            format: TextureF,
            depth_format: TextureFormat::R8Unorm,
        }
    }

    pub const fn preferred_surface_format(formats: &[wgpu::TextureFormat]) -> wgpu::TextureFormat {
        for format in &formats {
            if format == &wgpu::TextureFormat::Bgra8UnormSrgb
                || format == &wgpu::TextureFormat::Rgba8UnormSrgb
            {
                return *format;
            }
        }
        formats[0]
    }
}

impl GPUContext {
    /// Creates a new GPU context.
    pub async fn new(
        desired_backend: Option<wgpu::Backend>,
        desired_features: Option<wgpu::Features>,
    ) -> Self {
        profiling::scope!("GPUContext::new");
        let backend = wgpu::Backends::VULKAN | wgpu::Backends::METAL | wgpu::Backends::DX12;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            flags: wgpu::InstanceFlags::from_build_config(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: Default::default(),
        });

        let mut adapters = ArrVec::<PotentialAdapter, 4>::new();
        for adapter in instance.enumerate_adapters(backend) {
            let limits = adapter.limits();
            let features = adapter.features();
            let info = adapter.get_info();
            log::info!("{:?} Adapter: {:#?}", backend, info);
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

        let adapter = adapters.get(0).expect("No adapters found");

        // Create the GPU device and queue.
        let (device, queue) = adapter
            .adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("BK7084RS GPU Logical Device"),
                    features: desired_features.unwrap_or(adapter.features),
                    limits: adapter.limits.clone(),
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
        }
    }
}
