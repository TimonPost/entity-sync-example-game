use crate::systems::{insert_enemy_system, move_player_system, ClientState};
use crossterm::{cursor::Hide, style::Color, terminal::enable_raw_mode, ExecutableCommand};
use legion::systems::schedule::Builder;
use legion_sync::{
    components::UidComponent,
    filters::filter_fns::registered,
    resources::{RemovedEntities, TickResource},
    tracking::Bincode,
    universe::{
        client::{ClientUniverse, ClientUniverseBuilder},
        UniverseBuilder,
    },
};
use net_sync::{compression::lz4::Lz4, uid::UidAllocator};
use shared::{
    components::{Coloring, PlayerType, PlayerTypeOp, Position},
    systems::draw_player_system,
};
use std::{io::stdout, thread, time::Duration};

mod systems;

fn main() {
    initialize_terminal();

    let mut client = ClientUniverseBuilder::default()
        .with_tcp::<Bincode, Lz4>("127.0.0.1:1119".parse().unwrap())
        .main_builder(initialize_main_systems)
        .with_resource(ClientState::new())
        .with_resource(TickResource::new())
        .with_resource(RemovedEntities::new())
        .default_systems()
        .default_resources::<Bincode, Lz4>()
        .build();

    initial_data(&mut client);

    loop {
        client.tick();

        thread::sleep(Duration::from_millis(20));
    }
}

fn initialize_terminal() {
    //            simple_logger::init().unwrap();
    enable_raw_mode();
    stdout().execute(Hide);
}

fn initialize_main_systems(builder: Builder) -> Builder {
    builder
        .add_system(draw_player_system())
        .add_system(insert_enemy_system())
        .add_system(move_player_system())
        .flush()
}

fn initial_data(client: &mut ClientUniverse) {
    let entities = client.universe().main_world_mut().insert(
        (),
        (0..1).map(|_| {
            (
                Position { x: 5, y: 5 },
                PlayerType::new(PlayerTypeOp::Player),
                Coloring::new(Color::Blue),
            )
        }),
    );

    for entity in entities.to_vec() {
        let uid = UidComponent::new(client.new_entity_id(entity));
        client
            .universe()
            .main_world_mut()
            .add_component(entity, uid);
    }
}
