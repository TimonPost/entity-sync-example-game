use crate::systems::{entity_remove_system, position_update_system};
use bincode::Config;
use crossterm::{
    cursor::Hide,
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
use legion::{
    prelude::{CommandBuffer, Entity, Resources, Schedule, Universe, World},
    systems::{resource::Resource, schedule::Builder},
};
use legion_sync::{
    resources::{
        tcp::TcpListenerResource, BufferResource, Packer, ReceiveBufferResource,
        RegisteredComponentsResource, RemovedEntities, ResourcesExt, TickResource, TrackResource,
    },
    systems::{
        tcp::{tcp_connection_listener, tcp_receive_system},
        SchedulerExt,
    },
    tracking::Bincode,
    universe::{server::ServerUniverseBuilder, UniverseBuilder},
};
use net_sync::compression::{lz4::Lz4, CompressionStrategy};
use shared::systems::{draw_entities, draw_player_system};
use std::{
    collections::VecDeque,
    io::stdout,
    net::{SocketAddr, TcpListener},
    thread,
    time::Duration,
};
use track::serialization::SerializationStrategy;

mod systems;

fn main() {
    initialize_terminal();

    let listener = TcpListener::bind("127.0.0.1:1119".parse::<SocketAddr>().unwrap()).unwrap();
    listener.set_nonblocking(true);

    let mut server = ServerUniverseBuilder::default()
        .main_builder(initialize_local_systems)
        .remote_builder(initialize_remote_systems)
        .with_tcp::<Bincode, Lz4>(listener)
        .with_resource(TickResource::new())
        .with_resource(RemovedEntities::new())
        .default_resources::<Bincode, Lz4>()
        .default_systems()
        .build();

    loop {
        server.tick();
        thread::sleep(Duration::from_millis(10));
    }
}

fn initialize_terminal() {
    //                    simple_logger::init().unwrap();
    enable_raw_mode();
    stdout().execute(EnterAlternateScreen);
    stdout().execute(Hide);
}

fn initialize_local_systems(builder: Builder) -> Builder {
    builder
        .add_system(draw_player_system())
        //                .add_system(draw_entities())
        .flush()
}

fn initialize_remote_systems(builder: Builder) -> Builder {
    builder
        .add_system(position_update_system())
        .add_system(entity_remove_system())
        .flush()
}
