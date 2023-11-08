mod input;
pub use input::*;
mod window;
pub use window::*;

use crate::{
    core::{
        camera::{Camera, Projection},
        mesh::Mesh,
        Color, FxHashMap, SmlString,
    },
    render::{
        rpass::{ClearPass, Wireframe},
        surface::Surface,
        GpuContext, RenderTarget, Renderer,
    },
    scene::{Entity, NodeIdx, Scene},
};
use dolly::rig::CameraRig;
use pyo3::{
    prelude::*,
    types::{PyDict, PyTuple},
};
use std::sync::{Arc, RwLock};
use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder, EventLoopProxy},
    window::Window,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum UserEvent {
    Todo,
}
unsafe impl Send for UserEvent {}

#[pyclass(subclass)]
#[derive(Debug, Clone)]
pub struct PyAppState {
    pub input: InputState,
    event_loop: Option<EventLoopProxy<UserEvent>>,
    event_listeners: FxHashMap<SmlString, Vec<PyObject>>,
    start_time: std::time::Instant,
    prev_time: std::time::Instant,
    curr_time: std::time::Instant,
    scene: Option<Arc<RwLock<Scene>>>,
}

unsafe impl Send for PyAppState {}

/// Python interface for AppState
#[pymethods]
impl PyAppState {
    #[new]
    pub fn new() -> PyResult<Self> {
        let now = std::time::Instant::now();
        Ok(Self {
            input: InputState::default(),
            event_loop: None,
            event_listeners: Default::default(),
            start_time: now,
            prev_time: now,
            curr_time: now,
            scene: None,
        })
    }

    /// Register an event type.
    #[pyo3(text_signature = "($self, event_name)")]
    pub fn register_event_type(&mut self, event_type: String) {
        self.event_listeners
            .entry(SmlString::from(event_type))
            .or_insert_with(Vec::new);
    }

    /// Register multiple event types.
    #[pyo3(text_signature = "($self, event_names)")]
    pub fn register_event_types(&mut self, event_types: Vec<String>) {
        for event_type in event_types {
            self.register_event_type(event_type);
        }
    }

    /// Attach a handler to an event type.
    pub fn attach_event_handler(&mut self, event_type: String, listener: PyObject) {
        self.event_listeners
            .entry(SmlString::from(event_type))
            .or_insert(Vec::new())
            .push(listener);
    }

    /// Detach a handler from an event type.
    pub fn detach_event_handler(&mut self, event_type: String, listener: PyObject) {
        if let Some(listeners) = self.event_listeners.get_mut(event_type.as_str()) {
            listeners.retain(|l| !l.is(&listener));
        }
    }

    /// Get the frame time in seconds.
    pub fn delta_time(&self) -> f32 {
        self.curr_time.duration_since(self.prev_time).as_secs_f32()
    }

    /// Create a camera
    pub fn create_camera(&mut self, projection: Projection) -> Entity {
        let camera = Camera::new(projection, 0.0..f32::INFINITY, Color::LIGHT_PERIWINKLE);
        self.scene
            .as_mut()
            .expect("Scene not initialized!")
            .write()
            .map(|mut scene| scene.spawn(NodeIdx::root(), (camera,)))
            .expect("Failed to create camera!")
    }

    // pub fn add_mesh(&mut self, mesh: Mesh) -> Entity {
    //     self.scene
    //         .write()
    //         .map(|mut scene| {
    //             let gpu_mesh = scene.
    //             scene.spawn(NodeIdx::root(), (mesh,))
    //         })
    //         .expect("Failed to add mesh!")
    // }
}

/// Implementation of the methods only available to Rust.
impl PyAppState {
    pub fn create_window(
        &mut self,
        event_loop: &EventLoop<UserEvent>,
        builder: PyWindowBuilder,
    ) -> Window {
        env_logger::init();
        let inner_size = builder.size.unwrap_or([800, 600]);
        let position = builder.position.unwrap_or([200, 200]);
        let window = winit::window::WindowBuilder::new()
            .with_title(builder.title)
            .with_inner_size(PhysicalSize::new(inner_size[0], inner_size[1]))
            .with_resizable(builder.resizable)
            .with_position(winit::dpi::PhysicalPosition::new(
                position[0] as i32,
                position[1] as i32,
            ))
            .with_maximized(builder.maximized)
            .with_fullscreen(builder.fullscreen)
            .with_transparent(builder.transparent)
            .with_decorations(builder.decorations)
            .with_visible(false)
            .build(&event_loop)
            .unwrap();
        self.event_loop = Some(event_loop.create_proxy());
        window
    }

    /// Returns true if an event has been fully processed.
    pub fn process_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::ModifiersChanged(state) => {
                self.input.update_modifier_states(*state);
                true
            }
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput {
                    virtual_keycode: Some(keycode),
                    state,
                    ..
                } => {
                    self.input.update_key_states(*keycode, *state);
                    true
                }
                _ => false,
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.input.update_cursor_delta(*position);
                true
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.input.update_mouse_button_states(*button, *state);
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.input.update_scroll_delta(*delta);
                true
            }
            _ => false,
        }
    }

    /// Dispatch an event to all attached listeners.
    fn dispatch_event(
        &self,
        py: Python<'_>,
        event_name: &str,
        args: &PyTuple,
        kwargs: Option<&PyDict>,
    ) -> PyResult<()> {
        if let Some(listeners) = self.event_listeners.get(event_name) {
            for listener in listeners {
                listener.call(py, args, kwargs).unwrap();
            }
        }
        Ok(())
    }

    fn dispatch_resize_event(&self, width: u32, height: u32) {
        Python::with_gil(|py| {
            self.dispatch_event(
                py,
                "on_resize",
                PyTuple::new(py, &[width.into_py(py), height.into_py(py)]),
                None,
            )
        })
        .unwrap();
    }

    fn dispatch_update_event(&self, input: Input, dt: f32) {
        Python::with_gil(|py| {
            self.dispatch_event(
                py,
                "on_update",
                PyTuple::new(py, &[dt.into_py(py), input.into_py(py)]),
                None,
            )
            .unwrap();
        });
    }

    fn update(&mut self, dt: f32) {
        let input = self.input.take();
        self.dispatch_update_event(input, dt);
    }

    /// Initialize the scene.
    fn init_scene(&mut self, device: &wgpu::Device) {
        match self.scene {
            None => self.scene = Some(Arc::new(RwLock::new(Scene::new(device)))),
            Some(_) => {}
        }
    }
}

#[pyfunction]
pub fn run_main_loop(mut app: PyAppState, builder: PyWindowBuilder) {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    // Create the displaying window.
    let mut window = app.create_window(&event_loop, builder);
    let win_id = window.id();

    // Create the GPU context and surface.
    let context = GpuContext::new(None);
    // Initialize the scene.
    app.init_scene(&context.device);
    // Create the surface to render to.
    let mut surface = Surface::new(&context, &window);
    // Create the renderer.
    let mut renderer = Renderer::new(&context, surface.aspect_ratio());

    let mut wireframe_rpass = Wireframe::new(&context.device, surface.format());
    // let mut clear_rpass = ClearPass::new(Renderer::CLEAR_COLOR);

    let mesh = Mesh::cube();

    {
        let mut scene = app.scene.as_ref().unwrap().write().unwrap();
        scene.spawn_mesh(NodeIdx::root(), &mesh, &context.device, &context.queue);
        let projection = Projection::perspective(60.0);
        let camera = Camera::new(projection, 0.0..f32::INFINITY, Color::LIGHT_PERIWINKLE);
        let _ = scene.spawn(NodeIdx::root(), (camera,));
    }

    // Ready to present the window.
    window.set_visible(true);

    event_loop.run(move |event, _, control_flow| {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events.
        control_flow.set_poll();

        app.curr_time = std::time::Instant::now();
        let dt = app.delta_time();
        app.prev_time = app.curr_time;
        // print!("frame time: {} secs\r", dt);

        match event {
            Event::UserEvent(_) => {
                // todo
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == win_id => {
                if !app.process_input(event) {
                    match event {
                        WindowEvent::CloseRequested => {
                            control_flow.set_exit();
                        }
                        WindowEvent::Resized(sz) => {
                            if surface.resize(&context.device, sz.width, sz.height) {
                                // Dispatch the resize event.
                                app.dispatch_resize_event(sz.width, sz.height);
                                // TODO: update camera aspect ratio
                            }
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            if surface.resize(
                                &context.device,
                                new_inner_size.width,
                                new_inner_size.height,
                            ) {
                                // TODO: update camera aspect ratio
                            }
                        }
                        _ => {}
                    }
                    if app.input.is_key_pressed(KeyCode::Escape) {
                        control_flow.set_exit();
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == win_id => {
                // Grab a frame from the surface.
                let frame = surface
                    .get_current_texture()
                    .expect("Failed to get a frame from the surface");
                let target = RenderTarget {
                    size: frame.texture.size(),
                    view: frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    format: surface.format(),
                };

                if let Some(scene) = app.scene.as_mut() {
                    let scene = scene.read().unwrap();
                    match renderer.render(&scene, &mut wireframe_rpass, &target) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            surface.resize(&context.device, surface.width(), surface.height());
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            control_flow.set_exit();
                        }
                        Err(e) => eprintln!("{:?}", e),
                    }
                }

                frame.present();
            }
            /// The main event loop has been cleared and will not be processed
            /// again until the next event needs to be handled.
            Event::MainEventsCleared => {
                app.update(dt);
                window.request_redraw();
            }
            // Otherwise, just let the event pass through.
            _ => {}
        }
    });
}
