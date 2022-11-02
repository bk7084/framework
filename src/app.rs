use std::collections::HashMap;
use std::sync::Arc;
use pyo3::ffi::PyWeakReference;
use winit::event_loop::{EventLoop, EventLoopBuilder, EventLoopProxy};
use winit::window::{Fullscreen, Window, WindowBuilder};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString, PyTuple};
use winit::dpi::PhysicalSize;
use crate::context::GpuContext;
use winit::event::{Event, KeyboardInput, ModifiersState, WindowEvent};
use crate::input::{InputState, KeyCode, MouseButton};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum UserEvent {
    ToggleFullscreen,
}

unsafe impl Send for UserEvent {}

#[pyclass(subclass)]
#[derive(Clone)]
pub struct AppState {
    pub title: String,
    #[pyo3(get)]
    pub width: u32,
    #[pyo3(get)]
    pub height: u32,
    pub resizable: bool,
    pub fullscreen: bool,

    pub gpu_ctx: Option<GpuContext>,

    input_state: Py<InputState>,
    event_loop: Option<Arc<EventLoopProxy<UserEvent>>>,
    event_listeners: HashMap<String, Vec<PyObject>>,
    start_time: std::time::Instant,
    prev_time: std::time::Instant,
    curr_time: std::time::Instant,
}

unsafe impl Send for AppState {}

/// Python interface for AppState
#[pymethods]
impl AppState {
    #[new]
    pub fn new(title: String, width: u32, height: u32, resizable: bool, fullscreen: bool) -> PyResult<Self> {
        let now = std::time::Instant::now();
        Python::with_gil(|py| {
            Ok(
                Self {
                    title,
                    width,
                    height,
                    resizable,
                    fullscreen,
                    gpu_ctx: None,
                    input_state: Py::new(py, InputState::default())?,
                    event_loop: None,
                    event_listeners: Default::default(),
                    start_time: now,
                    prev_time: now,
                    curr_time: now,
                }
            )
        })
    }

    /// Register an event type.
    #[pyo3(text_signature = "($self, event_name)")]
    pub fn register_event_type(&mut self, event_type: String) {
        self.event_listeners.entry(event_type).or_insert_with(Vec::new);
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
        self.event_listeners.entry(event_type).or_insert(Vec::new()).push(listener);
    }

    /// Detach a handler from an event type.
    pub fn detach_event_handler(&mut self, event_type: String, listener: PyObject) {
        if let Some(listeners) = self.event_listeners.get_mut(&event_type) {
            listeners.retain(|l| !l.is(&listener));
        }
    }

    /// Dispatch an event to all attached listeners.
    #[pyo3(text_signature = "($self, event_name, *args, **kwargs)")]
    pub fn dispatch_event(&self, py: Python<'_>, event_name: &str, args: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<()> {
        if let Some(listeners) = self.event_listeners.get(event_name) {
            for listener in listeners {
                // println!("[rs] cursor pos: {:?}", self.input_state);
                listener.call(py, args, kwargs).unwrap();
            }
        }
        Ok(())
    }

    pub fn delta_time(&self) -> f32 {
        self.curr_time.duration_since(self.prev_time).as_secs_f32()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            (self.width, self.height) = (width, height);
            match &mut self.gpu_ctx {
                Some(ctx) => {
                    ctx.resize(width, height);
                },
                None => {}
            }
        }
    }

    pub fn toggle_fullscreen(&mut self) {
        self.fullscreen = !self.fullscreen;
        self.event_loop.as_ref().unwrap().send_event(UserEvent::ToggleFullscreen).unwrap();
    }

    pub fn is_key_pressed(&self, key_code: KeyCode) -> bool {
        let key_code = winit::event::VirtualKeyCode::from(key_code);
        Python::with_gil(|py| {
            let input: InputState = self.input_state.extract(py).unwrap();
            *input.keys.get(&key_code).unwrap_or(&false)
        })
    }

    pub fn is_key_released(&self, key_code: KeyCode) -> bool {
        !self.is_key_pressed(key_code)
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        // *self.input_state.mouse_buttons.get(&button.into()).unwrap_or(&false)
        false
    }

    pub fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        !self.is_mouse_button_pressed(button)
    }

    pub fn is_shift_pressed(&self) -> bool {
        // self.input_state.modifiers.shift()
        false
    }

    pub fn is_ctrl_pressed(&self) -> bool {
        // self.input_state.modifiers.ctrl()
        false
    }

    pub fn is_alt_pressed(&self) -> bool {
        // self.input_state.modifiers.alt()
        false
    }

    pub fn is_super_pressed(&self) -> bool {
        // self.input_state.modifiers.logo()
        false
    }

    pub fn cursor_position(&self) -> [f32; 2] {
        Python::with_gil(|py| {
            let input: InputState = self.input_state.extract(py).unwrap();
            input.cursor_pos
        })
    }

    pub fn cursor_delta(&self) -> [f32; 2] {
        // self.input_state.cursor_delta
        [0.0, 0.0]
    }

    pub fn scroll_delta(&self) -> f32 {
        // self.input_state.scroll_delta
        0.0
    }

    pub fn input_state(&mut self, py: Python) -> Py<InputState> {
        self.input_state.clone_ref(py)
    }
}

/// Implementation of the methods only available to Rust.
impl AppState {
    pub fn init(&mut self, event_loop: &EventLoop<UserEvent>) -> Window {
        env_logger::init();
        let window = WindowBuilder::new()
            .with_title(self.title.clone())
            .with_inner_size(PhysicalSize::new(self.width, self.height))
            .with_resizable(self.resizable)
            .build(&event_loop)
            .unwrap();
        self.gpu_ctx = Some(pollster::block_on(GpuContext::new(&window)));
        self.event_loop = Some(Arc::new(event_loop.create_proxy()));
        window
    }

    /// Returns true if an event has been fully processed.
    pub fn process_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::ModifiersChanged(state) => {
                println!("ModifiersChanged: {:?}", state);
                Python::with_gil(|py| {
                    let mut input: InputState = self.input_state.extract(py).unwrap();
                    input.update_modifier_states(*state);
                });
                true
            }
            WindowEvent::KeyboardInput { input, .. } => {
                println!("KeyboardInput: {:?}", input);
                match input {
                    KeyboardInput {
                        virtual_keycode: Some(keycode),
                        state,
                        ..
                    } => {
                        Python::with_gil(|py| {
                            let mut input: InputState = self.input_state.extract(py).unwrap();
                            input.update_key_states(*keycode, *state);
                        });
                        true
                    },
                    _ => {
                        false
                    }
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                println!("CursorMoved: {:?}", position);
                Python::with_gil(|py| {
                    let mut input: InputState = self.input_state.extract(py).unwrap();
                    input.update_cursor_delta(*position);
                    println!("cursor pos: {:?}", input.cursor_pos);
                });
                true
            },
            WindowEvent::MouseInput {
                 state, button, ..
            } => {
                Python::with_gil(|py| {
                    let mut input: InputState = self.input_state.extract(py).unwrap();
                    input.update_mouse_button_states(*button, *state);
                });
                true
            },
            WindowEvent::MouseWheel { delta, ..} => {
                Python::with_gil(|py| {
                    let mut input: InputState = self.input_state.extract(py).unwrap();
                    input.update_scroll_delta(*delta);
                });
                true
            }
            _ => { false }
        }
    }

    pub fn update(&mut self, dt: f32) {
        // println!("cursor pos: {:?}", self.input_state.cursor_pos);
        Python::with_gil(|py| {
            self.dispatch_event(py,
                                "on_update",
                                PyTuple::new(py, &[dt.into_py(py)]), None).unwrap();
        });
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        profiling::scope!("AppState::render");
        match &mut self.gpu_ctx {
            Some(ctx) => {
                let output = ctx.surface.get_current_texture()?;
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render")
                });

                {
                    let _render_pass = encoder.begin_render_pass(
                        &wgpu::RenderPassDescriptor {
                            label: Some("Main Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.1,
                                        g: 0.2,
                                        b: 0.3,
                                        a: 1.0
                                    }),
                                    store: true
                                }
                            })],
                            depth_stencil_attachment: None
                        }
                    );
                }

                ctx.queue.submit(std::iter::once(encoder.finish()));

                output.present();

                Ok(())
            },
            None => {
                Ok(())
            }
        }
    }
}

#[pyfunction]
pub fn run_main_loop(mut app: AppState) {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let window = app.init(&event_loop);
    let win_id = window.id();

    event_loop.run(move |event, _, control_flow| {
        app.curr_time = std::time::Instant::now();
        let dt = app.delta_time();
        app.prev_time = app.curr_time;

        match event {
            Event::WindowEvent {
                ref event, window_id
            } if window_id == win_id => if !app.process_input(event) {
                println!("Handling the rest of the events");
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit
                    },
                    WindowEvent::Resized(sz) => {
                        app.resize(sz.width, sz.height);
                    },
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        app.resize(new_inner_size.width, new_inner_size.height);
                    },
                    _ => {}
                }
                if app.is_key_pressed(KeyCode::F11) {
                    app.toggle_fullscreen();
                }
                if app.is_key_pressed(KeyCode::Escape) {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
            },
            Event::UserEvent(event) => match event {
                UserEvent::ToggleFullscreen => {
                    window.set_fullscreen(if app.fullscreen {
                        Some(Fullscreen::Borderless(None))
                    } else {
                        None
                    });
                }
            },
            Event::RedrawRequested(window_id) if window_id == win_id => {
                match app.render() {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost) => app.resize(app.width, app.height),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = winit::event_loop::ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e)
                }
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            },
            _ => {}
        }

        app.update(dt);
    });
}