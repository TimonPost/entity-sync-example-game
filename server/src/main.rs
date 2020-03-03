use crate::systems::{
    apply_modifications_system, insert_received_entities_system, remove_entities_system,
};
use crossterm::{cursor::Hide, terminal::enable_raw_mode, ExecutableCommand};
use legion::prelude::{Resources, Schedule, Universe};
use legion_sync::network_universe::NetworkUniverse;
use legion_sync::resources::RegisteredComponentsResource;
use legion_sync::tracking::Bincode;
use legion_sync::{
    resources::{
        tcp::TcpListenerResource, BufferResource, Packer, ReceiveBufferResource, TrackResource,
    },
    systems::tcp::{tcp_connection_listener, tcp_receive_system},
};
use net_sync::compression::lz4::Lz4;
use shared::systems::{draw_entities, draw_player_system};
use std::{
    io::stdout,
    net::{SocketAddr, TcpListener},
    thread,
    time::Duration,
};

mod systems;

fn main() {
    initialize_terminal();

    let mut universe = NetworkUniverse::new();
    let mut local_world = universe.create_world();
    let mut remote_world = universe.create_world();

    let listener = TcpListener::bind("127.0.0.1:1119".parse::<SocketAddr>().unwrap()).unwrap();
    listener.set_nonblocking(true);

    let mut resources = Resources::default();
    resources.insert(TrackResource::new());
    resources.insert(ReceiveBufferResource::default());
    resources.insert(TcpListenerResource::new(Some(listener)));
    resources.insert(Packer::<Bincode, Lz4>::default());
    resources.insert(BufferResource::from_capacity(1500));
    resources.insert(RegisteredComponentsResource::new());

    let mut remote_schedule = initialize_remote_sync_systems();
    let mut local_schedule = initialize_local_systems();

    let mut frame = 1;
    loop {
        local_schedule.execute(&mut local_world, &mut resources);
        remote_schedule.execute(&mut remote_world, &mut resources);

        if frame % 10 == 0 {
            universe.merge_into(&mut local_world, &remote_world);
            frame = 1;
        }

        frame += 1;
        thread::sleep(Duration::from_millis(10));
    }
}

fn initialize_terminal() {
    //            simple_logger::init().unwrap();
    enable_raw_mode();
    stdout().execute(Hide);
}

fn initialize_local_systems() -> Schedule {
    Schedule::builder()
        .add_system(draw_player_system())
        //        .add_system(draw_entities())
        .flush()
        .build()
}

fn initialize_remote_sync_systems() -> Schedule {
    Schedule::builder()
        .add_system(tcp_connection_listener())
        .add_system(tcp_receive_system::<Bincode, Lz4>())
        .add_system(insert_received_entities_system())
        .add_system(apply_modifications_system())
        .add_system(remove_entities_system())
        .flush()
        .build()
}
