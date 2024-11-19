use crate::core::ArrVec;
use std::sync::Arc;
use wgpu::DeviceType;

/// Aggregates all the objects needed to use the GPU.
#[pyo3::pyclass]
#[derive(Clone)]
pub struct GpuContext {
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
    /// Whether the adapter supports constant sized binding arrays.
    pub constant_sized_binding_array: bool,
}

/// Potential adapter to use.
struct PotentialAdapter {
    adapter: wgpu::Adapter,
    info: wgpu::AdapterInfo,
    limits: wgpu::Limits,
    features: wgpu::Features,
}

impl GpuContext {
    /// Creates a new GPU context.
    pub fn new(desired_features: Option<wgpu::Features>) -> Self {
        profiling::scope!("GPUContext::new");
        let backends = wgpu::Backends::VULKAN | wgpu::Backends::METAL | wgpu::Backends::DX12;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            flags: wgpu::InstanceFlags::from_build_config(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: Default::default(),
        });

        let mut adapters = ArrVec::<PotentialAdapter, 16>::new();
        for adapter in instance.enumerate_adapters(backends) {
            let limits = adapter.limits();
            let features = adapter.features();
            let info = adapter.get_info();
            log::info!("{:?} Adapter: {:#?}", backends, info);
            if features.contains(wgpu::Features::PUSH_CONSTANTS) {
                adapters.push(PotentialAdapter {
                    adapter,
                    info,
                    limits,
                    features,
                });
            }
        }
        adapters.sort_by_key(|adapter| match adapter.info.device_type {
            DeviceType::DiscreteGpu => 0,
            DeviceType::IntegratedGpu => 1,
            DeviceType::VirtualGpu => 2,
            DeviceType::Cpu => 3,
            DeviceType::Other => 4,
        });

        if adapters.is_empty() {
            panic!("No adapters found");
        }

        let adapter = adapters.remove(0);

        let features = adapter.features;
        log::info!(
            "Max textures: {}",
            adapter.limits.max_sampled_textures_per_shader_stage
        );
        log::info!(
            "Max samplers: {}",
            adapter.limits.max_samplers_per_shader_stage
        );
        let constant_sized_binding_array = !features.contains(wgpu::Features::BUFFER_BINDING_ARRAY);
        log::info!(
            "Constant sized binding array: {}",
            constant_sized_binding_array
        );

        let mut desired_features = desired_features.unwrap_or_else(wgpu::Features::empty);
        log::debug!("Desired features: {:#?}", desired_features);

        // Only enable mappable primary buffers on macOS with unified memory
        // architecture.
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        if desired_features.contains(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS) {
            desired_features.remove(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS);
        }

        if !features.contains(desired_features) {
            for feat in desired_features.iter() {
                if !features.contains(feat) {
                    log::error!("Feature '{:?}' does not exist", feat);
                }
            }
            panic!(
                "Desired features {:?} not all ->supported by the adapter",
                desired_features
            );
        }

        let limits = adapter.limits;

        // Create the GPU device and queue.
        let (device, queue) = pollster::block_on(async {
            adapter
                .adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("BK7084RS GPU Logical Device"),
                        required_features: features,
                        required_limits: limits.clone(),
                        memory_hints: Default::default(),
                    },
                    Some(std::path::Path::new("./bk7084_trace.log")),
                )
                .await
                .expect("Failed to create device")
        });

        GpuContext {
            instance: Arc::new(instance),
            adapter: Arc::new(adapter.adapter),
            device: Arc::new(device),
            queue: Arc::new(queue),
            features,
            limits,
            constant_sized_binding_array,
        }
    }
}

#[pyo3::pymethods]
impl GpuContext {
    #[new]
    pub fn new_py() -> Self {
        Self::new(None)
    }
}
