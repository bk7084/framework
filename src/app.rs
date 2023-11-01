use crate::{
    input::{InputState, KeyCode},
    renderer::{context::GPUContext, surface::Surface, Renderer},
    scene::Scene,
};
use dolly::rig::CameraRig;
use legion::{Resources, Schedule, World};
use pyo3::{
    prelude::*,
    types::{PyDict, PyTuple},
};
use std::collections::HashMap;
use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder, EventLoopProxy},
    window::{Fullscreen, Window},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum UserEvent {
    Todo,
}
unsafe impl Send for UserEvent {}

#[pyclass]
#[pyo3(name = "Window")]
#[derive(Debug, Clone)]
pub struct WindowBuilder {
    pub size: Option<[u32; 2]>,
    pub position: Option<[u32; 2]>,
    pub resizable: bool,
    pub title: String,
    pub fullscreen: Option<Fullscreen>,
    pub maximized: bool,
    pub transparent: bool,
    pub decorations: bool,
}

impl Default for WindowBuilder {
    #[inline]
    fn default() -> Self {
        Self {
            size: None,
            position: None,
            resizable: true,
            title: "BK7084".to_owned(),
            maximized: false,
            fullscreen: None,
            transparent: false,
            decorations: true,
        }
    }
}

#[pymethods]
impl WindowBuilder {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the size of the window.
    #[pyo3(signature = (width=800, height=600))]
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.size = Some([width, height]);
    }

    /// Set the position of the window.
    #[pyo3(signature = (x=200, y=200))]
    pub fn set_position(&mut self, x: u32, y: u32) {
        self.position = Some([x, y]);
    }

    /// Set whether the window is resizable.
    pub fn set_resizable(&mut self, resizable: bool) {
        self.resizable = resizable;
    }

    /// Set the title of the window.
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    /// Set whether the window is fullscreen.
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = if fullscreen {
            Some(Fullscreen::Borderless(None))
        } else {
            None
        };
    }

    /// Set whether the window is maximized.
    pub fn set_maximized(&mut self, maximized: bool) {
        self.maximized = maximized;
    }

    /// Set whether the window is transparent.
    pub fn set_transparent(&mut self, transparent: bool) {
        self.transparent = transparent;
    }

    /// Set whether the window has decorations.
    pub fn set_decorations(&mut self, decorations: bool) {
        self.decorations = decorations;
    }
}

#[pyclass(subclass)]
#[derive(Clone)]
pub struct AppState {
    pub input: InputState,
    event_loop: Option<EventLoopProxy<UserEvent>>,
    event_listeners: HashMap<String, Vec<PyObject>>,
    start_time: std::time::Instant,
    prev_time: std::time::Instant,
    curr_time: std::time::Instant,
    scene: Scene,
}

unsafe impl Send for AppState {}

/// Python interface for AppState
#[pymethods]
impl AppState {
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
            scene: Scene::new(),
        })
    }

    /// Register an event type.
    #[pyo3(text_signature = "($self, event_name)")]
    pub fn register_event_type(&mut self, event_type: String) {
        self.event_listeners
            .entry(event_type)
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
            .entry(event_type)
            .or_insert(Vec::new())
            .push(listener);
    }

    /// Detach a handler from an event type.
    pub fn detach_event_handler(&mut self, event_type: String, listener: PyObject) {
        if let Some(listeners) = self.event_listeners.get_mut(&event_type) {
            listeners.retain(|l| !l.is(&listener));
        }
    }

    /// Dispatch an event to all attached listeners.
    #[pyo3(text_signature = "($self, event_name, *args, **kwargs)")]
    pub fn dispatch_event(
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

    pub fn delta_time(&self) -> f32 {
        self.curr_time.duration_since(self.prev_time).as_secs_f32()
    }
}

/// Implementation of the methods only available to Rust.
impl AppState {
    pub fn create_window(
        &mut self,
        event_loop: &EventLoop<UserEvent>,
        builder: WindowBuilder,
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

    pub fn update(&mut self, dt: f32) {
        Python::with_gil(|py| {
            let input = self.input.take();
            self.dispatch_event(
                py,
                "on_update",
                PyTuple::new(py, &[dt.into_py(py), input.into_py(py)]),
                None,
            )
            .unwrap();
        });
    }
}

#[pyfunction]
pub fn run_main_loop(mut app: AppState, builder: WindowBuilder) {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    let mut window = app.create_window(&event_loop, builder);
    let win_id = window.id();

    // Create the GPU context and surface.
    let context = pollster::block_on(GPUContext::new(None));
    let mut surface = Surface::new(&context, &window);
    let mut renderer = Renderer::new(&context, surface.aspect_ratio());

    // Ready to present the window.
    window.set_visible(true);

    event_loop.run(move |event, _, control_flow| {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events.
        control_flow.set_poll();

        app.curr_time = std::time::Instant::now();
        let dt = app.delta_time();
        app.prev_time = app.curr_time;
        print!("frame time: {} secs\r", dt);

        match event {
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
                            surface.resize(&context.device, sz.width, sz.height);
                            // TODO: update camera aspect ratio
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            surface.resize(
                                &context.device,
                                new_inner_size.width,
                                new_inner_size.height,
                            );
                            // TODO: update camera aspect ratio
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

                match renderer.render(&frame) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        surface.resize(&context.device, surface.width(), surface.height())
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        *control_flow = winit::event_loop::ControlFlow::Exit
                    }
                    Err(e) => eprintln!("{:?}", e),
                }

                frame.present();
            }
            /// The main event loop has been cleared and will not be processed
            /// again until the next event needs to be handled.
            Event::MainEventsCleared => {
                // TODO: update states
                window.request_redraw();
            }
            // Otherwise, just let the event pass through.
            _ => {}
        }
    });
}
