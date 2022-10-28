use std::collections::HashMap;
use std::sync::Arc;
use winit::event_loop::{EventLoop, EventLoopBuilder, EventLoopProxy};
use winit::window::{Fullscreen, Window, WindowBuilder};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString, PyTuple};
use winit::dpi::PhysicalSize;
use crate::context::GpuContext;
use winit::event::{Event, KeyboardInput, WindowEvent};

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
    pub event_loop: Option<Arc<EventLoopProxy<UserEvent>>>,
    on_update: Option<PyObject>,
    event_listeners: HashMap<String, Vec<PyObject>>,

    start_time: std::time::Instant,
    prev_time: std::time::Instant,
    curr_time: std::time::Instant,
}

unsafe impl Send for AppState {}

#[pymethods]
impl AppState {
    #[new]
    pub fn new(title: String, width: u32, height: u32, resizable: bool, fullscreen: bool) -> Self {
        let now = std::time::Instant::now();
        Self {
            title,
            width,
            height,
            resizable,
            fullscreen,
            gpu_ctx: None,
            event_loop: None,
            on_update: None,
            event_listeners: Default::default(),
            start_time: now,
            prev_time: now,
            curr_time: now,
        }
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

    /// Attach an event listener to an event type.
    pub fn attach_listener(&mut self, event_type: String, listener: PyObject) {
        self.event_listeners.entry(event_type).or_insert(Vec::new()).push(listener);
    }

    /// Detach an event listener from an event type.
    pub fn detach_listener(&mut self, event_type: String, listener: PyObject) {
        if let Some(listeners) = self.event_listeners.get_mut(&event_type) {
            listeners.retain(|l| !l.is(&listener));
        }
    }

    /// Dispatch an event to all attached listeners.
    #[pyo3(text_signature = "($self, event_name, *args, **kwargs)")]
    pub fn dispatch(&self, event_name: &str, args: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<()> {
        if let Some(listeners) = self.event_listeners.get(event_name) {
            for listener in listeners {
                Python::with_gil(|py| {
                    listener.call(py, args, kwargs).unwrap();
                })
            }
        }
        Ok(())
    }

    pub fn delta_time(&self) -> f32 {
        self.curr_time.duration_since(self.prev_time).as_secs_f32()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.resize_inner(PhysicalSize::new(width, height));
    }

    pub fn toggle_fullscreen(&mut self) {
        self.fullscreen = !self.fullscreen;
        self.event_loop.as_ref().unwrap().send_event(UserEvent::ToggleFullscreen).unwrap();
    }

    pub fn register_on_update(&mut self, on_update: PyObject) {
        println!("register_on_update");
        self.on_update = Some(on_update);
    }
}

impl AppState {
    pub fn resize_inner(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            (self.width, self.height) = (new_size.width, new_size.height);
            match &mut self.gpu_ctx {
                Some(ctx) => {
                    ctx.resize(new_size);
                },
                None => {}
            }
        }
    }

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
        false
    }

    pub fn update(&mut self, dt: f32) {
        Python::with_gil(|py| {
            self.dispatch("on_update",
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
        app.update(app.delta_time());
        app.prev_time = app.curr_time;

        match event {
            Event::WindowEvent {
                ref event, window_id
            } if window_id == win_id => if !app.process_input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            state: winit::event::ElementState::Pressed,
                            virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => {
                        *control_flow = winit::event_loop::ControlFlow::Exit
                    },
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            state: winit::event::ElementState::Pressed,
                            virtual_keycode: Some(winit::event::VirtualKeyCode::F11),
                            ..
                        },
                        ..
                    } => {
                        app.toggle_fullscreen();
                    },
                    WindowEvent::Resized(physical_size) => {
                        app.resize_inner(*physical_size);
                    },
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        app.resize_inner(**new_inner_size);
                    },
                    _ => {}
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
    });
}