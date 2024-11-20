mod input;
pub use input::*;
pub mod command;

mod window;

pub use window::*;

use crate::render::rpass::BlinnPhongRenderPass;
use crate::render::surface::Surface;
use crate::render::RenderTarget;
use crate::{
    app::command::Command,
    compute::SunlightScore,
    core::{
        camera::{Camera, Projection},
        mesh::{Mesh, MeshBundle},
        Color, ConcatOrder, FxHashMap, Light, SmlString,
    },
    render::{GpuContext, Renderer},
    scene::{Entity, NodeIdx, PyEntity, Scene},
};
use crossbeam_channel::Sender;
use glam::{Mat4, Quat, Vec3};
use legion::IntoQuery;
use numpy as np;
use numpy::array;
use pyo3::{
    prelude::*,
    types::{PyDict, PyTuple},
};
use std::sync::{Arc, RwLock};
use winit::event::{Event, KeyEvent};
use winit::event_loop::{ControlFlow, EventLoopBuilder};
use winit::keyboard::PhysicalKey;
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopProxy},
    window::Window,
};

/// User events that can be sent to the event loop.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum UserEvent<E: 'static> {
    /// A user event with a payload.
    Event(E),
    /// A user event with no payload.
    Empty,
}

unsafe impl<E: 'static> Send for UserEvent<E> {}

#[pyclass(subclass)]
#[derive(Clone)]
pub struct PyAppState {
    pub input: InputState,
    event_loop: Option<EventLoopProxy<UserEvent<()>>>,
    event_listeners: FxHashMap<SmlString, Vec<PyObject>>,
    start_time: std::time::Instant,
    prev_time: std::time::Instant,
    curr_time: std::time::Instant,
    context: Arc<GpuContext>,
    scene: Arc<RwLock<Scene>>,
    renderer: Arc<RwLock<Renderer>>,
    scene_cmd_sender: Sender<Command>,
    renderer_cmd_sender: Sender<Command>,
    sunlight_score: Arc<RwLock<SunlightScore>>,
    main_camera: Option<Entity>,
}

/// Python interface for AppState
#[pymethods]
impl PyAppState {
    #[new]
    pub fn new() -> PyResult<Self> {
        env_logger::init();
        let now = std::time::Instant::now();
        let context = Arc::new(GpuContext::new(Some(
            wgpu::Features::POLYGON_MODE_LINE
                | wgpu::Features::PUSH_CONSTANTS
                | wgpu::Features::TEXTURE_BINDING_ARRAY
                | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY,
        )));
        let (scene_cmd_sender, scene_cmd_receiver) = crossbeam_channel::unbounded::<Command>();
        let scene = Scene::new(scene_cmd_sender.clone(), scene_cmd_receiver);
        let (renderer_cmd_sender, renderer_cmd_receiver) =
            crossbeam_channel::unbounded::<Command>();
        let renderer = Renderer::new(&context, renderer_cmd_receiver);
        let sunlight_score = SunlightScore::new(&context.device);
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
            scene_cmd_sender,
            renderer_cmd_sender,
            main_camera: None,
            sunlight_score: Arc::new(RwLock::new(sunlight_score)),
        })
    }

    /// Register an event type.
    #[pyo3(text_signature = "($self, event_name)")]
    pub fn register_event_type(&mut self, event_type: String) {
        self.event_listeners
            .entry(SmlString::from(event_type))
            .or_default();
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
            .or_default()
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

    /// Get the transform of an entity.
    #[pyo3(name = "get_transform")]
    pub fn get_transform_py(&self, entity: &PyEntity) -> Py<np::PyArray2<f32>> {
        Python::with_gil(|py| {
            let mat = self.scene.read().unwrap().nodes[entity.entity.node]
                .transform()
                .to_mat4()
                .transpose();
            let [x, y, z, w] = mat.to_cols_array_2d();
            np::PyArray2::<f32>::from_array(py, &array![x, y, z, w]).to_owned()
        })
    }

    /// Set the backface culling state.
    pub fn enable_backface_culling(&mut self, enabled: bool) {
        self.renderer_cmd_sender
            .send(Command::EnableBackfaceCulling(enabled))
            .unwrap();
    }

    /// Set the shadows rendering state.
    pub fn enable_shadows(&mut self, enabled: bool) {
        self.renderer_cmd_sender
            .send(Command::EnableShadows(enabled))
            .unwrap();
    }

    /// Set the wireframe rendering state.
    pub fn enable_wireframe(&mut self, enabled: bool) {
        self.renderer_cmd_sender
            .send(Command::EnableWireframe(enabled))
            .unwrap();
    }

    pub fn enable_lighting(&mut self, enabled: bool) {
        self.renderer_cmd_sender
            .send(Command::EnableLighting(enabled))
            .unwrap();
    }

    #[deprecated(note = "Should be automatically updated by the renderer.")]
    pub fn update_shadow_map_ortho_proj(&mut self, max_dist: f32) {
        self.renderer_cmd_sender
            .send(Command::UpdateShadowMapOrthoProj(max_dist))
            .unwrap();
    }

    pub fn compute_sunlight_scores(&mut self) -> Vec<f32> {
        profiling::scope!("compute_sunlight_score");
        self.sunlight_score
            .write()
            .map(|mut score| {
                let scene = self.scene.read().unwrap();
                let mut mesh_bundle_query = <(&MeshBundle, &NodeIdx)>::query();
                let meshes = mesh_bundle_query.iter(&scene.world).filter(|(_, node)| {
                    scene.nodes[**node].is_visible() && scene.nodes[**node].cast_shadows()
                });
                let renderer = self.renderer.read().unwrap();
                score.compute(
                    &self.context.device,
                    &self.context.queue,
                    &scene,
                    &renderer,
                    meshes,
                )
            })
            .unwrap()
    }

    /// Create a camera
    ///
    /// # Arguments
    ///
    /// * `pos` - The position of the camera.
    /// * `target` - The target of the camera.
    /// * `fov` - The field of view of the camera in degrees.
    #[pyo3(name = "create_camera")]
    #[pyo3(signature = (pos, look_at, fov_v, near=0.1, far=200.0, background=Color::DARK_GREY))]
    pub fn create_camera_py(
        &mut self,
        pos: &np::PyArray2<f32>,
        look_at: &np::PyArray2<f32>,
        fov_v: f32,
        near: f32,
        far: f32,
        background: Color,
    ) -> PyEntity {
        log::debug!(
            "[Py] Creating camera at {:?} looking at {:?} with fov: {:?}",
            pos,
            look_at,
            fov_v
        );
        let proj = Projection::perspective(fov_v, near, far);
        Python::with_gil(|_py| {
            let pos = Vec3::from_slice(pos.readonly().as_slice().unwrap());
            let target = Vec3::from_slice(look_at.readonly().as_slice().unwrap());
            let entity = self.create_camera(proj, pos, target, background);
            PyEntity {
                entity,
                cmd_sender: self.scene_cmd_sender.clone(),
            }
        })
    }

    /// Adds a mesh to the scene.
    // TODO: pass transform as an argument.
    #[pyo3(name = "add_mesh")]
    #[pyo3(signature = (mesh, parent=None))]
    pub fn add_mesh_py(&mut self, mesh: &mut Mesh, parent: Option<&PyEntity>) -> PyEntity {
        let parent = parent.map(|p| p.entity.node).unwrap_or(NodeIdx::root());
        let entity = self.spawn_object_with_mesh(parent, mesh);
        PyEntity {
            entity,
            cmd_sender: self.scene_cmd_sender.clone(),
        }
    }

    #[pyo3(name = "spawn_building")]
    pub fn spawn_empty_py(&mut self) -> PyEntity {
        let entity = self.spawn_empty(NodeIdx::root());
        PyEntity {
            entity,
            cmd_sender: self.scene_cmd_sender.clone(),
        }
    }

    #[pyo3(signature = (pos, color=Color::WHITE))]
    pub fn add_point_light_py(&mut self, pos: &np::PyArray2<f32>, color: Color) -> PyEntity {
        let position = Vec3::from_slice(pos.readonly().as_slice().unwrap());
        let entity = self.spawn_light(NodeIdx::root(), Light::Point { color }, Some(position));
        PyEntity {
            entity,
            cmd_sender: self.scene_cmd_sender.clone(),
        }
    }

    #[pyo3(name = "add_directional_light")]
    #[pyo3(signature = (dir, color=Color::WHITE))]
    pub fn add_directional_light_py(&mut self, dir: &np::PyArray2<f32>, color: Color) -> PyEntity {
        let direction = Vec3::from_slice(dir.readonly().as_slice().unwrap());
        let entity = self.spawn_light(
            NodeIdx::root(),
            Light::Directional { direction, color },
            None,
        );
        PyEntity {
            entity,
            cmd_sender: self.scene_cmd_sender.clone(),
        }
    }
}

/// Implementation of the methods only available to Rust.
impl PyAppState {
    pub fn create_window(
        &mut self,
        event_loop: &EventLoop<UserEvent<()>>,
        builder: PyWindowBuilder,
    ) -> Window {
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
            .build(event_loop)
            .unwrap();
        self.event_loop = Some(event_loop.create_proxy());
        window
    }

    /// Spawn an object with the given mesh and parent.
    ///
    /// Returns the entity ID of the spawned object.
    pub fn spawn_object_with_mesh(&mut self, parent: NodeIdx, mesh: &mut Mesh) -> Entity {
        log::debug!("Spawning object with mesh#{}", mesh.name);
        mesh.validate();
        self.renderer
            .write()
            .map(|mut renderer| {
                let mesh_bundle = renderer.upload_mesh(mesh);
                let entity = self
                    .scene
                    .write()
                    .map(|mut scene| scene.spawn(parent, (mesh_bundle,)))
                    .unwrap();
                renderer.add_instancing(mesh_bundle, &[entity.node]);
                entity
            })
            .expect("Failed to spawn object with mesh!")
    }

    /// Spawn an empty object with the given parent.
    pub fn spawn_empty(&mut self, parent: NodeIdx) -> Entity {
        self.scene
            .write()
            .map(|mut scene| scene.spawn(parent, ()))
            .unwrap()
    }

    pub fn spawn_light(&mut self, parent: NodeIdx, light: Light, position: Option<Vec3>) -> Entity {
        self.scene
            .write()
            .map(|mut scene| {
                let entity = scene.spawn(parent, (light,));
                if light.is_point() {
                    // Update light's position only if it's a point light.
                    let translation = position.unwrap_or(Vec3::ZERO);
                    scene.nodes[entity.node].transform_mut().translation = translation;
                }
                entity
            })
            .unwrap()
    }

    /// Prepare the scene and renderer for rendering.
    pub fn prepare(&mut self) {
        let has_light = self.scene.read().unwrap().has_light();
        if !has_light {
            self.spawn_light(
                NodeIdx::root(),
                Light::Directional {
                    direction: Vec3::new(1.0, -1.0, -1.0),
                    color: Color::WHITE,
                },
                None,
            );
        }
        self.scene.write().unwrap().prepare(&mut self.main_camera);
        self.renderer.write().unwrap().prepare();
    }

    /// Creates the main camera.
    pub fn create_camera(
        &mut self,
        proj: Projection,
        pos: Vec3,
        target: Vec3,
        background: Color,
    ) -> Entity {
        let entity = self
            .scene
            .write()
            .map(|mut scene| {
                let camera = Camera::new(proj, background, false);
                let entity = scene.spawn(NodeIdx::root(), (camera,));
                let transform = scene.nodes[entity.node].transform_mut();
                transform.translation = pos;
                // Avoid gimbal lock.
                let forward = (target - pos).normalize();
                let up = if forward.y.abs() > 0.999 {
                    Vec3::new(0.0, 0.0, 1.0)
                } else {
                    Vec3::new(0.0, 1.0, 0.0)
                };
                transform.looking_at(target, up);
                entity
            })
            .expect("Failed to create camera!");
        self.main_camera = Some(entity);
        entity
    }

    /// Returns true if an event has been fully processed.
    pub fn process_input(&mut self, event: &WindowEvent) -> bool {
        profiling::scope!("process_input");
        match event {
            WindowEvent::ModifiersChanged(modifiers) => {
                self.input.update_modifier_states(modifiers);
                true
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        state,
                        ..
                    },
                ..
            } => {
                self.input.update_key_states(*keycode, *state);
                true
            }

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
                let _ = listener.call(py, args, kwargs).map_err(|e| {
                    log::error!("Failed to dispatch event: {}", e);
                });
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

    fn dispatch_update_event(&self, input: Input, dt: f32, t: f32) {
        Python::with_gil(|py| {
            self.dispatch_event(
                py,
                "on_update",
                PyTuple::new(py, &[input.into_py(py), dt.into_py(py), t.into_py(py)]),
                None,
            )
            .unwrap();
        });
    }

    fn update(&mut self, win_size: (u32, u32), dt: f32, t: f32) {
        let input = self.input.take();

        // Rotate the camera with the middle mouse button.
        if input.is_mouse_pressed(MouseButton::Middle)
            || (input.is_mouse_pressed(MouseButton::Left) && input.is_alt_pressed())
        {
            let delta = input.cursor_delta();
            // Make the rotation the same direction as the mouse movement.
            let horiz = -delta[0] / win_size.0 as f32 * std::f32::consts::TAU * 2.0;
            let vert = -delta[1] / win_size.1 as f32 * std::f32::consts::TAU * 2.0;
            // Set a threshold to avoid jitter.
            if horiz.abs() > 0.001 || vert.abs() > 0.001 {
                match (
                    input.is_key_pressed(KeyCode::ShiftLeft),
                    input.is_key_pressed(KeyCode::ControlLeft),
                ) {
                    (true, true) => {
                        // Free rotate the camera around its own axis.
                        let rot = Quat::from_mat4(
                            &(Mat4::from_rotation_y(horiz) * Mat4::from_rotation_x(vert)),
                        );
                        self.scene_cmd_sender
                            .send(Command::Rotate {
                                entity: self.main_camera.unwrap(),
                                rotation: rot,
                                order: ConcatOrder::Post,
                            })
                            .unwrap();
                    }
                    (true, false) => {
                        // Pan the camera.
                        self.scene_cmd_sender
                            .send(Command::CameraPan {
                                entity: self.main_camera.unwrap(),
                                delta_x: horiz,
                                delta_y: vert,
                            })
                            .unwrap();
                    }
                    (false, _) => {
                        // Orbit the camera around the target.
                        self.scene_cmd_sender
                            .send(Command::CameraOrbit {
                                entity: self.main_camera.unwrap(),
                                rotation_x: vert,
                                rotation_y: horiz,
                            })
                            .unwrap();
                    }
                }
            }
        }

        // Zoom in/out with the mouse wheel.
        if input.scroll_delta().is_normal() {
            let scale = if input.is_key_pressed(KeyCode::ControlLeft) {
                10.0
            } else {
                1.0
            };
            self.scene_cmd_sender
                .send(Command::Translate {
                    entity: self.main_camera.unwrap(),
                    translation: Vec3::new(0.0, 0.0, input.scroll_delta() * dt * scale),
                    order: ConcatOrder::Post,
                })
                .unwrap();
        }

        // Dispatch the update event, potentially run the user's update function.
        self.dispatch_update_event(input, dt, t);
    }
}

#[pyfunction]
pub fn run_main_loop(mut app: PyAppState, builder: PyWindowBuilder) {
    let event_loop = EventLoopBuilder::<UserEvent<()>>::with_user_event()
        .build()
        .unwrap();

    // A helper struct to make sure the window and surface are all
    // moved together.
    struct WinSurf<'a> {
        pub window: &'a Window,
        pub surface: Surface<'a>,
    }

    // Create the displaying window.
    let window = app.create_window(&event_loop, builder);
    let win_id = window.id();
    let context = app.context.clone();

    // Create the surface to render to.
    let surface = Surface::new(&context, &window);
    let mut blph_render_pass = BlinnPhongRenderPass::new(&context, surface.format());
    // Ready to present the window.
    window.set_visible(true);

    let mut win_surf = WinSurf {
        window: &window,
        surface,
    };
    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events.
    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop
        .run(move |event, evlp| {
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
                                evlp.exit();
                            }
                            WindowEvent::Resized(sz) => {
                                if win_surf
                                    .surface
                                    .resize(&context.device, sz.width, sz.height)
                                {
                                    // Dispatch the resize event.
                                    app.dispatch_resize_event(sz.width, sz.height);
                                    // TODO: update camera aspect ratio
                                }
                            }
                            WindowEvent::ScaleFactorChanged { .. } => {
                                if win_surf.surface.resize(
                                    &context.device,
                                    win_surf.window.inner_size().width,
                                    win_surf.window.inner_size().height,
                                ) {
                                    // Dispatch the resize event.
                                    app.dispatch_resize_event(
                                        win_surf.window.inner_size().width,
                                        win_surf.window.inner_size().height,
                                    );
                                }
                            }
                            WindowEvent::RedrawRequested => {
                                // Grab a frame from the surface.
                                let frame = win_surf
                                    .surface
                                    .get_current_texture()
                                    .expect("Failed to get a frame from the surface");
                                let target = RenderTarget {
                                    size: frame.texture.size(),
                                    view: frame.texture.create_view(&Default::default()),
                                    format: win_surf.surface.format(),
                                };

                                let scene = app.scene.read().unwrap();
                                match app.renderer.write().unwrap().render(
                                    &scene,
                                    &target,
                                    &mut blph_render_pass,
                                ) {
                                    Ok(_) => {}
                                    Err(
                                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                    ) => {
                                        win_surf.surface.resize(
                                            &context.device,
                                            win_surf.surface.width(),
                                            win_surf.surface.height(),
                                        );
                                    }
                                    Err(wgpu::SurfaceError::OutOfMemory) => {
                                        evlp.exit();
                                    }
                                    Err(e) => eprintln!("{:?}", e),
                                }

                                frame.present();
                            }
                            _ => {}
                        }
                        if app.input.is_key_pressed(KeyCode::Escape) {
                            evlp.exit();
                        }
                    }
                }
                // The main event loop has been cleared and will not be processed
                // again until the next event needs to be handled.
                Event::AboutToWait => {
                    app.curr_time = std::time::Instant::now();
                    let dt = app.delta_time();
                    app.prev_time = app.curr_time;
                    let t = app.start_time.elapsed().as_secs_f32();
                    app.update(win_surf.surface.size(), dt, t);
                    app.prepare();
                    win_surf.window.request_redraw();
                }
                // Otherwise, just let the event pass through.
                _ => {}
            }
        })
        .expect("Failed to run the main loop");
}
