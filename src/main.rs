use legion::{systems::CommandBuffer, *};

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

#[derive(Debug, Clone, Copy, PartialEq)]
struct ValObj {
    val: u32,
}

#[system]
fn create_entities(cmds: &mut CommandBuffer, #[resource] values: &Vec<u32>) {
    for val in values.iter() {
        // entities are inserted as tuples of components
        cmds.push((
            Position { x: 0.0, y: 0.0 },
            Velocity { dx: 1.0, dy: 1.0 },
            ValObj { val: *val },
        ));
    }
}

// #[system(for_each)]
// fn update_positions(pos: &mut Position, vel: &Velocity, #[resource] time:
// &Time) {     pos.x += vel.dx * time.;
//     pos.y += vel.dy;
// }

fn main() {
    ecs_101();
    // ecs_intro();
}

fn ecs_101() {
    // creates a container of entities
    let mut world = World::default();
    // resources can be shared between systems
    let mut resources = Resources::default();
    resources.insert(vec![1, 2, 3u32]);
    // resources.insert(Time::default());

    // schedule will automatically execute systems in parallel
    let mut schedule = Schedule::builder()
        .add_system(create_entities_system())
        .build();

    schedule.execute(&mut world, &mut resources);

    // let entity = world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { dx: 1.0,
    // dy: 1.0 }));
    let entity = world.spawn((
        (Position { x: 0.0, y: 0.0 }, Velocity { dx: 1.0, dy: 1.0 }),
        ValObj { val: 1 },
    ));

    let entities = world.extend(vec![
        (Position { x: 1.0, y: 1.0 }, Velocity { dx: 1.0, dy: 1.0 }),
        (Position { x: 2.0, y: 2.0 }, Velocity { dx: 1.0, dy: 1.0 }),
        (Position { x: 3.0, y: 3.0 }, Velocity { dx: 1.0, dy: 1.0 }),
    ]);

    let empty_entity = world.spawn(());

    world
        .entry(empty_entity)
        .unwrap()
        .add_component(Position { x: 10.0, y: 10.0 });

    // entries return `None` if the entity does not exist
    if let Some(mut entry) = world.entry(entity) {
        // access information about the entity's archetype
        println!(
            "{:?} has {:?}",
            entity,
            entry.archetype().layout().component_types()
        );

        if let Ok(pos) = entry.get_component::<Position>() {
            println!("Position: {:?}", pos);
        }

        // add an extra component
        entry.add_component(12f32);

        // access the entity's components, returns `None` if the entity does not have
        // the component
        assert_eq!(entry.get_component::<f32>().unwrap(), &12f32);
    }

    let mut pos_val_query = <(&Position, &ValObj)>::query();
    for (pos, val) in pos_val_query.iter(&world) {
        println!("Position: {:?}, ValObj: {:?}", pos, val);
    }
}

fn ecs_intro() {
    struct Person {
        name: &'static str,
    }

    struct Age(u8);

    #[system(for_each)]
    fn say_hello(person: &Person) {
        println!("Hello, {}!", person.name);
    }

    #[system]
    fn introduce_people(cmds: &mut CommandBuffer, #[resource] names: &Vec<&'static str>) {
        for name in names.iter() {
            cmds.push((Person { name: *name }, Age(30)));
        }
    }

    let mut world = World::default();

    // resources can be shared between systems
    let mut resources = Resources::default();
    resources.insert(vec!["Alice", "Bob"]);
    let mut schedule = Schedule::builder()
        .add_system(introduce_people_system())
        .add_system(say_hello_system())
        .build();

    schedule.execute(&mut world, &mut resources);
}
