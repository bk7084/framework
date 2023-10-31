use crate::{
    input::{InputState, KeyCode},
    renderer::{context::GPUContext, surface::Surface},
};
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
    window::{Fullscreen, Window, WindowBuilder},
};

pub trait App {
    /// Create a new window together with the event loop.
    fn create_window(&mut self, builder: WindowBuilder) -> Window {
        todo!()
    }
}

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
    pub input: InputState,
    event_loop: Option<EventLoopProxy<UserEvent>>,
    event_listeners: HashMap<String, Vec<PyObject>>,
    start_time: std::time::Instant,
    prev_time: std::time::Instant,
    curr_time: std::time::Instant,
    // ecs: World,
    // resources: Resources,
    // systems: Schedule,
    // pub gpu_ctx: Option<GpuContext>,
}

unsafe impl Send for AppState {}

/// Python interface for AppState
#[pymethods]
impl AppState {
    #[new]
    pub fn new(
        title: String,
        width: u32,
        height: u32,
        resizable: bool,
        fullscreen: bool,
    ) -> PyResult<Self> {
        let now = std::time::Instant::now();
        Ok(Self {
            title,
            width,
            height,
            resizable,
            fullscreen,
            input: InputState::default(),
            event_loop: None,
            event_listeners: Default::default(),
            start_time: now,
            prev_time: now,
            curr_time: now,
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

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            (self.width, self.height) = (width, height);
            // match &mut self.gpu_ctx {
            //     Some(ctx) => {
            //         ctx.resize(width, height);
            //     },
            //     None => {}
            // }
        }
    }

    pub fn toggle_fullscreen(&mut self) {
        self.fullscreen = !self.fullscreen;
        self.event_loop
            .as_ref()
            .unwrap()
            .send_event(UserEvent::ToggleFullscreen)
            .unwrap();
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

    // pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    //     profiling::scope!("AppState::render");
    //     let output = self.context.surface.get_current_texture()?;
    //             let view = output
    //                 .texture
    //                 .create_view(&wgpu::TextureViewDescriptor::default());
    //
    //             let mut encoder =
    //                 ctx.device
    //                     .create_command_encoder(&wgpu::CommandEncoderDescriptor {
    //                         label: Some("Render"),
    //                     });
    //
    //             {
    //                 let _render_pass =
    // encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    // label: Some("Main Render Pass"),                     color_attachments:
    // &[Some(wgpu::RenderPassColorAttachment {                         view:
    // &view,                         resolve_target: None,
    //                         ops: wgpu::Operations {
    //                             load: wgpu::LoadOp::Clear(wgpu::Color {
    //                                 r: 0.1,
    //                                 g: 0.2,
    //                                 b: 0.3,
    //                                 a: 1.0,
    //                             }),
    //                             store: true,
    //                         },
    //                     })],
    //                     depth_stencil_attachment: None,
    //                 });
    //             }
    //
    //             ctx.queue.submit(std::iter::once(encoder.finish()));
    //
    //             output.present();
    //
    //             Ok(())
    //     }
    // }
}

#[pyfunction]
pub fn run_main_loop(mut app: AppState) {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let mut window = app.init(&event_loop);
    let win_id = window.id();

    // Create the GPU context and surface.
    let context = pollster::block_on(GPUContext::new(None));
    let mut surface = Surface::new(&context, &window);

    // Ready to present the window.
    window.set_visible(true);

    event_loop.run(move |event, _, control_flow| {
        app.curr_time = std::time::Instant::now();
        let dt = app.delta_time();
        app.prev_time = app.curr_time;

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == win_id => {
                if !app.process_input(event) {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = winit::event_loop::ControlFlow::Exit
                        }
                        WindowEvent::Resized(sz) => {
                            app.resize(sz.width, sz.height);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            app.resize(new_inner_size.width, new_inner_size.height);
                        }
                        _ => {}
                    }
                    if app.input.is_key_pressed(KeyCode::F11) {
                        app.toggle_fullscreen();
                    }
                    if app.input.is_key_pressed(KeyCode::Escape) {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }
                }
            }
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
                // match app.render() {
                //     Ok(_) => {},
                //     Err(wgpu::SurfaceError::Lost) => app.resize(app.width,
                // app.height),
                //     Err(wgpu::SurfaceError::OutOfMemory) => *control_flow =
                // winit::event_loop::ControlFlow::Exit,
                //     Err(e) => eprintln!("{:?}", e)
                // }
            }
            /// The main event loop has been cleared and will not be processed
            /// again until the next event needs to be handled.
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            // Otherwise, just let the event pass through.
            _ => {}
        }
    });
}
