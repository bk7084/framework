use legion::*;
use legion::systems::CommandBuffer;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Velocity {
    dx: f32,
    dy: f32,
}

struct ValObj {
    val: u32,
}

#[system]
fn create_entities(cmds: &mut CommandBuffer, #[resource] values: &Vec<u32>) {
    for val in values.iter() {
        // entities are inserted as tuples of components
        cmds.push((Position { x: 0.0, y: 0.0 }, Velocity { dx: 1.0, dy: 1.0 }, ValObj { val: *val }));
    }
}

// #[system(for_each)]
// fn update_positions(pos: &mut Position, vel: &Velocity, #[resource] time: &Time) {
//     pos.x += vel.dx * time.;
//     pos.y += vel.dy;
// }

fn main() {
    // creates a container of entities
    // let mut world = World::default();
    // // resources can be shared between systems
    // let mut resources = Resources::default();
    // resources.insert(vec![1, 2, 3u32]);
    // // resources.insert(Time::default());
    //
    // // schedule will automatically execute systems in parallel
    // let mut schedule = Schedule::builder()
    //     .add_system(create_entities())
    //     .build();
    //
    // schedule.execute(&mut world, &mut resources);

    // let entity = world.push((Position { x: 0.0, y: 0.0 }, Velocity { dx: 1.0, dy: 1.0 }));
    //
    // let entities = world.extend(vec![
    //     (Position { x: 1.0, y: 1.0 }, Velocity { dx: 1.0, dy: 1.0 }),
    //     (Position { x: 2.0, y: 2.0 }, Velocity { dx: 1.0, dy: 1.0 }),
    //     (Position { x: 3.0, y: 3.0 }, Velocity { dx: 1.0, dy: 1.0 }),
    // ]);
    //
    // // entries return `None` if the entity does not exist
    // if let Some(mut entry) = world.entry(entity) {
    //     // access information about the entity's archetype
    //     println!("{:?} has {:?}", entity, entry.archetype().layout().component_types());
    //
    //     // add an extra component
    //     entry.add_component(12f32);
    //
    //     // access the entity's components, returns `None` if the entity does not have the component
    //     assert_eq!(entry.get_component::<f32>().unwrap(), &12f32);
    // }
}