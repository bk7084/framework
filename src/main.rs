use legion::*;

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

fn main() {
    let mut world = World::default();

    let entity = world.push((Position { x: 0.0, y: 0.0 }, Velocity { dx: 1.0, dy: 1.0 }));

    let entities = world.extend(vec![
        (Position { x: 1.0, y: 1.0 }, Velocity { dx: 1.0, dy: 1.0 }),
        (Position { x: 2.0, y: 2.0 }, Velocity { dx: 1.0, dy: 1.0 }),
        (Position { x: 3.0, y: 3.0 }, Velocity { dx: 1.0, dy: 1.0 }),
    ]);

    // entries return `None` if the entity does not exist
    if let Some(mut entry) = world.entry(entity) {
        // access information about the entity's archetype
        println!("{:?} has {:?}", entity, entry.archetype().layout().component_types());

        // add an extra component
        entry.add_component(12f32);

        // access the entity's components, returns `None` if the entity does not have the component
        assert_eq!(entry.get_component::<f32>().unwrap(), &12f32);
    }
}