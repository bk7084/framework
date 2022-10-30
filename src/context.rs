use std::sync::Arc;
use wgpu::CompositeAlphaMode;

/// Aggregation of necessary resources for using GPU.
#[derive(Clone)]
pub struct GpuContext {
    pub instance: Arc<wgpu::Instance>,
    pub adapter: Arc<wgpu::Adapter>,
    pub surface: Arc<wgpu::Surface>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub query_set: Option<Arc<wgpu::QuerySet>>,
    pub surface_config: wgpu::SurfaceConfiguration,
}

impl GpuContext {
    pub async fn new(window: &winit::window::Window) -> Self {
        profiling::scope!("GpuContext::new");
        let win_size = window.inner_size();
        // Create instance handle to GPU and automatically select the backend according to the platform.
        let instance = Arc::new(wgpu::Instance::new(wgpu::Backends::all()));
        // An abstract type of surface to present rendered images to.
        let surface = Arc::new(unsafe { instance.create_surface(window) });
        // Physical device: handle to actual graphics card.
        let adapter = Arc::new(instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap_or_else(|| {
                panic!(
                    "Failed to request physical device! {}",
                    concat!(file!(), ":", line!())
                )
            }));
        println!("Using GPU: {}", adapter.get_info().name);
        let features = adapter.features();
        // Logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: features
                        | wgpu::Features::POLYGON_MODE_LINE
                        | wgpu::Features::TIMESTAMP_QUERY,
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to request logical device! {}",
                    concat!(file!(), ":", line!())
                )
            });
        // Surface configuration
        let modes = surface.get_supported_present_modes(&adapter);
        let present_mode = if modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else if modes.contains(&wgpu::PresentMode::Fifo) {
            wgpu::PresentMode::Fifo
        } else {
            wgpu::PresentMode::Immediate
        };

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: win_size.width,
            height: win_size.height,
            present_mode,
            alpha_mode: CompositeAlphaMode::Auto
        };
        surface.configure(&device, &surface_config);

        // Query pool to retrieve information from the GPU
        let query_set = Some(device.create_query_set(&wgpu::QuerySetDescriptor {
            label: None,
            count: 1,
            ty: wgpu::QueryType::Timestamp,
        }));

        Self {
            instance,
            surface,
            adapter,
            device: Arc::new(device),
            queue: Arc::new(queue),
            query_set: query_set.map(Arc::new),
            surface_config,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }
}

