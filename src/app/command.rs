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
    /// Sets by force the material to use. This will override the material
    /// set by the submesh. If the material index is out of bounds of all
    /// the materials of the entity, the command will set the material to
    /// the last material of the entity.
    UseMaterial { entity: Entity, material: u32 },
    /// Sets the entity as the main camera.
    SetAsMainCamera { entity: Entity },
    /// Clears the material override.
    ClearMaterialOverride { entity: Entity },
}

/// Receiver of commands.
pub type CommandReceiver = crossbeam_channel::Receiver<Command>;

/// Sender of commands.
pub type CommandSender = crossbeam_channel::Sender<Command>;
