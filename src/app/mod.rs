mod input;
pub use input::*;
pub mod command;

mod window;

pub use window::*;

use crate::{
    app::command::Command,
    core::{
        camera::{Camera, Projection},
        mesh::Mesh,
        Alignment, Color, ConcatOrder, FxHashMap, SmlString,
    },
    render::{rpass::Wireframe, surface::Surface, GpuContext, RenderTarget, Renderer},
    scene::{Entity, NodeIdx, PyEntity, Scene, Transform},
};
use crossbeam_channel::Sender;
use glam::{Mat4, Quat, Vec3};
use numpy as np;
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
#[derive(Clone)]
pub struct PyAppState {
    pub input: InputState,
    event_loop: Option<EventLoopProxy<UserEvent>>,
    event_listeners: FxHashMap<SmlString, Vec<PyObject>>,
    start_time: std::time::Instant,
    prev_time: std::time::Instant,
    curr_time: std::time::Instant,
    context: Arc<GpuContext>,
    scene: Arc<RwLock<Scene>>,
    renderer: Arc<RwLock<Renderer>>,
    sender: Sender<Command>,
    main_camera: Option<Entity>,
}

// unsafe impl Send for PyAppState {}

/// Python interface for AppState
#[pymethods]
impl PyAppState {
    #[new]
    pub fn new() -> PyResult<Self> {
        let now = std::time::Instant::now();
        let context = Arc::new(GpuContext::new(None));
        let scene = Scene::new();
        let sender = scene.cmd_sender().clone();
        let renderer = Renderer::new(&context, scene.cmd_receiver().clone());
        Ok(Self {
            context,
            input: InputState::default(),
            event_loop: None,
            event_listeners: Default::default(),
            start_time: now,
            prev_time: now,
            curr_time: now,
            scene: Arc::new(RwLock::new(scene)),
            renderer: Arc::new(RwLock::new(renderer)),
            sender,
            main_camera: None,
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
    #[pyo3(name = "create_camera")]
    pub fn create_camera_py(&mut self, proj: Projection, pos: &np::PyArray2<f32>) -> PyEntity {
        Python::with_gil(|_py| {
            let entity =
                self.create_main_camera(proj, Vec3::from_slice(pos.readonly().as_slice().unwrap()));
            PyEntity {
                entity,
                cmd_sender: self.sender.clone(),
            }
        })
    }

    /// Adds a mesh to the scene.
    pub fn add_mesh(&mut self, mesh: &Mesh) -> PyEntity {
        let entity = self.spawn_object_with_mesh(NodeIdx::root(), mesh);
        PyEntity {
            entity,
            cmd_sender: self.sender.clone(),
        }
    }
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

    /// Spawn an object with the given mesh and parent.
    ///
    /// Returns the entity ID of the spawned object.
    pub fn spawn_object_with_mesh(&mut self, parent: NodeIdx, mesh: &Mesh) -> Entity {
        let mesh_handle = self
            .renderer
            .write()
            .map(|mut renderer| renderer.add_mesh(mesh))
            .unwrap();
        self.scene
            .write()
            .map(|mut scene| scene.spawn(parent, (mesh_handle,)))
            .unwrap()
    }

    /// Prepare the scene for rendering.
    pub fn prepare_scene(&mut self) {
        self.scene.write().unwrap().process_commands();
    }

    /// Creates the main camera.
    pub fn create_main_camera(&mut self, proj: Projection, pos: Vec3) -> Entity {
        let entity = self
            .scene
            .write()
            .map(|mut scene| {
                let camera = Camera::new(proj, 0.0..f32::INFINITY, Color::LIGHT_PERIWINKLE, true);
                let entity = scene.spawn(NodeIdx::root(), (camera,));
                let transform = scene.nodes[entity.node].transform_mut();
                transform.translation = pos;
                transform.looking_at(Vec3::ZERO, Vec3::Y);
                entity
            })
            .expect("Failed to create camera!");
        self.main_camera = Some(entity);
        entity
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

    fn update(&mut self, win_size: (u32, u32), dt: f32, _t: f32) {
        let input = self.input.take();

        // Rotate the camera with the middle mouse button.
        if input.is_mouse_pressed(MouseButton::Middle) {
            let delta = input.cursor_delta();
            // Make the rotation the same direction as the mouse movement.
            let horiz = -delta[0] / win_size.0 as f32 * std::f32::consts::TAU * 2.0;
            let vert = -delta[1] / win_size.1 as f32 * std::f32::consts::TAU * 2.0;
            // Set a threshold to avoid jitter.
            if horiz.abs() > 0.0001 || vert.abs() > 0.0001 {
                if input.is_key_pressed(KeyCode::LShift) {
                    let rot = Quat::from_mat4(
                        &(Mat4::from_rotation_y(horiz) * Mat4::from_rotation_x(vert)),
                    );
                    self.sender
                        .send(Command::Rotate {
                            entity: self.main_camera.unwrap(),
                            rotation: rot,
                            order: ConcatOrder::Post,
                        })
                        .unwrap();
                } else {
                    self.sender
                        .send(Command::CameraOrbit {
                            entity: self.main_camera.unwrap(),
                            rotation_x: vert,
                            rotation_y: horiz,
                        })
                        .unwrap();
                };
            }
        }

        // Zoom in/out with the mouse wheel.
        if input.scroll_delta().is_normal() {
            self.sender
                .send(Command::Translate {
                    entity: self.main_camera.unwrap(),
                    translation: Vec3::new(0.0, 0.0, input.scroll_delta() * dt),
                    order: ConcatOrder::Post,
                })
                .unwrap();
        }

        // Dispatch the update event, potentially run the user's update function.
        self.dispatch_update_event(input, dt);
    }
}

#[pyfunction]
pub fn run_main_loop(mut app: PyAppState, builder: PyWindowBuilder) {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    // Create the displaying window.
    let window = app.create_window(&event_loop, builder);
    let win_id = window.id();
    let context = app.context.clone();

    // Create the surface to render to.
    let mut surface = Surface::new(&context, &window);

    let mut wireframe_rpass = Wireframe::new(&context.device, surface.format());
    // let mut clear_rpass = ClearPass::new(Renderer::CLEAR_COLOR);

    app.create_main_camera(Projection::perspective(45.0), Vec3::new(5.0, 5.0, 5.0));

    let mut cube = Mesh::cube(1.0);
    cube.compute_per_vertex_normals();
    let rect = Mesh::plane(0.5, Alignment::XY);
    let sphere = Mesh::sphere(1.0, 32, 16);
    let obj_cube = Mesh::load_from_obj("./data/cube/cube.obj");
    let obj_sibenik = Mesh::load_from_obj("./data/sibenik/sibenik.obj");
    let obj_sponza = Mesh::load_from_obj("./data/sponza/sponza.obj");

    let (rect0_id, rect1_id, sphere_id) = {
        let cube_entity = app.spawn_object_with_mesh(NodeIdx::root(), &cube);
        let rect0_entity = app.spawn_object_with_mesh(NodeIdx::root(), &rect);
        let rect1_entity = app.spawn_object_with_mesh(rect0_entity.node, &rect);
        let sphere_entity = app.spawn_object_with_mesh(NodeIdx::root(), &sphere);
        let obj_cube_entity = app.spawn_object_with_mesh(NodeIdx::root(), &obj_cube);
        let obj_sibenik_entity = app.spawn_object_with_mesh(NodeIdx::root(), &obj_sibenik);
        let obj_sponza_entity = app.spawn_object_with_mesh(NodeIdx::root(), &obj_sponza);

        let mut scene = app.scene.write().unwrap();
        let cube_node = &mut scene.nodes[cube_entity.node];
        let cube_transform = cube_node.transform_mut();
        cube_transform.rotation = Quat::from_rotation_y(45.0f32.to_radians());
        cube_transform.scale = Vec3::splat(1.5);

        let rect_node = &mut scene.nodes[rect0_entity.node];
        let rect_transform = rect_node.transform_mut();
        // rect_node.set_position([2.0, 0.0, 0.0].into());
        // rect_transform.rotation = Quat::from_rotation_z(45.0f32.to_radians());
        let tra = Transform::from_translation(Vec3::new(2.0, 0.0, 0.0));
        let rot = Transform::from_rotation(Quat::from_rotation_z(45.0f32.to_radians()));
        *rect_transform = tra * *rect_transform * rot;

        let sphere_node = &mut scene.nodes[sphere_entity.node];
        let sphere_transform = sphere_node.transform_mut();
        sphere_transform.translation = Vec3::new(-4.0, 0.0, 0.0);

        let obj_cube_node = &mut scene.nodes[obj_cube_entity.node];
        let obj_cube_transform = obj_cube_node.transform_mut();
        obj_cube_transform.translation = Vec3::new(0.0, 0.0, -2.0);

        // let obj_sibenik_node = &mut scene.nodes[obj_sibenik_entity.node];
        // let obj_sibenik_transform = obj_sibenik_node.transform_mut();
        // obj_sibenik_transform.translation = Vec3::new(0.0, 0.0, 0.0);

        (rect0_entity.node, rect1_entity.node, cube_entity.node)
    };

    // Ready to present the window.
    window.set_visible(true);

    event_loop.run(move |event, _, control_flow| {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events.
        control_flow.set_poll();

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

                let scene = app.scene.read().unwrap();
                match app
                    .renderer
                    .write()
                    .unwrap()
                    .render(&scene, &mut wireframe_rpass, &target)
                {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        surface.resize(&context.device, surface.width(), surface.height());
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        control_flow.set_exit();
                    }
                    Err(e) => eprintln!("{:?}", e),
                }

                frame.present();
            }
            // The main event loop has been cleared and will not be processed
            // again until the next event needs to be handled.
            Event::MainEventsCleared => {
                app.curr_time = std::time::Instant::now();
                let dt = app.delta_time();
                app.prev_time = app.curr_time;
                let t = app.start_time.elapsed().as_secs_f32();
                {
                    let mut scene = app.scene.write().unwrap();
                    let rect0 = &mut scene.nodes[rect0_id];
                    let rect0_transform = rect0.transform_mut();

                    let tra = Transform::from_translation(Vec3::new(2.0, 0.0, 0.0));
                    let rot =
                        Transform::from_rotation(Quat::from_rotation_z(45.0f32.to_radians() * t));

                    let rot0 =
                        Transform::from_rotation(Quat::from_rotation_z(60.0f32.to_radians() * t));

                    // *rect0_transform = rot * tra * rot0;
                    *rect0_transform = tra * rot0;

                    let rect1 = &mut scene.nodes[rect1_id];
                    let rect1_transform = rect1.transform_mut();
                    *rect1_transform = rot * tra;
                }
                app.update(surface.size(), dt, t);
                app.prepare_scene();
                window.request_redraw();
            }
            // Otherwise, just let the event pass through.
            _ => {}
        }
    });
}
