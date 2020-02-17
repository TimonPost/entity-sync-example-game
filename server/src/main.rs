use crate::systems::{draw_player_system, server_recv_sync_system};
use crossterm::{cursor::Hide, terminal::enable_raw_mode, ExecutableCommand};
use legion::prelude::{any, Resources, Schedule, Universe};
use legion_sync::resources::ServerUniverseResource;
use net_sync::compression::lz4::Lz4;
use std::{io::stdout, thread, time::Duration};
use track::serialisation::bincode::Bincode;

mod systems;

fn main() {
    initialize_terminal();

    let mut universe = Universe::new();
    let mut world = universe.create_world();

    let mut server_resource =
        ServerUniverseResource::new(Bincode, Lz4, "127.0.0.1:1119".parse().unwrap());

    let mut resources = Resources::default();
    resources.insert(server_resource);

    let mut schedule = initialize_systems();

    loop {
        schedule.execute(&mut world, &mut resources);

        thread::sleep(Duration::from_millis(10));
    }
}

fn initialize_terminal() {
    //    simple_logger::init().unwrap();
    enable_raw_mode();
    stdout().execute(Hide);
}

fn initialize_systems() -> Schedule {
    Schedule::builder()
        .add_system(server_recv_sync_system::<Bincode, Lz4>())
        .add_system(draw_player_system())
        .build()
}
