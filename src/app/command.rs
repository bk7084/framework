use crate::{core::ConcatOrder, scene::Entity};
use glam::{Quat, Vec3};

/// Possible commands that can be executed.
#[derive(Debug, Clone)]
pub enum Command {
    /// Translates the entity in the local space.
    Translate {
        entity: Entity,
        translation: Vec3,
        order: ConcatOrder,
    },
    /// Rotates the entity in the local space.
    Rotate {
        entity: Entity,
        rotation: Quat,
        order: ConcatOrder,
    },
    /// Rotates the camera entity around the center of the scene (more precisely
    /// rotates the object around the camera).
    CameraOrbit {
        entity: Entity,
        rotation_x: f32,
        rotation_y: f32,
    },
    /// Pans the camera entity.
    CameraPan {
        entity: Entity,
        delta_x: f32,
        delta_y: f32,
    },
    /// Scales the entity in the local space.
    Scale {
        entity: Entity,
        scale: Vec3,
        order: ConcatOrder,
    },
    /// Sets the transform of the entity.
    SetTransform {
        entity: Entity,
        translation: Vec3,
        rotation: Quat,
        scale: Vec3,
    },
    /// Sets if the entity is active or not.
    SetActive { entity: Entity, active: bool },
    /// Sets if the entity is visible or not.
    SetVisible { entity: Entity, visible: bool },
    /// Sets if the entity casts shadows or not.
    SetCastShadows { entity: Entity, cast_shadows: bool },
    /// Sets by force the material to use. This will override the material
    /// set by the submesh. If the material index is out of bounds of all
    /// the materials of the entity, the command will set the material to
    /// the last material of the entity.
    UseMaterial { entity: Entity, material: u32 },
    /// Sets the entity as the main camera.
    SetAsMainCamera { entity: Entity },
    /// Sets the direction of the directional light.
    SetDirectionalLight { entity: Entity, direction: Vec3 },
    /// Clears the material override.
    ClearMaterialOverride { entity: Entity },
    /// Enables or disables backface culling.
    EnableBackfaceCulling(bool),
    /// Enables or disables wireframe rendering.
    EnableWireframe(bool),
    /// Enables or disables shadwos.
    EnableShadows(bool),
    /// Updates manually the shadow map orthographic projection.
    UpdateShadowMapOrthoProj(f32),
    /// Enables or disables the lighting.
    EnableLighting(bool),
}

/// Receiver of commands.
pub type CommandReceiver = crossbeam_channel::Receiver<Command>;

/// Sender of commands.
pub type CommandSender = crossbeam_channel::Sender<Command>;
