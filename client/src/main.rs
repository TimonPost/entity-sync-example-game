use crate::systems::move_player_system;
use crossterm::{cursor::Hide, terminal::enable_raw_mode, ExecutableCommand};
use legion::{filter::filter_fns::any, prelude::*};
use legion_sync::resources::{ReceiveBufferResource, RegisteredComponentsResource};
use legion_sync::tracking::Bincode;
use legion_sync::{
    components::UidComponent,
    resources::{tcp::TcpClientResource, EventResource, Packer, SentBufferResource},
    systems::{tcp::tcp_sent_system, track_modifications_system},
};
use net_sync::{compression::lz4::Lz4, uid::UidAllocator};
use shared::{components::Position, systems::draw_player_system};
use std::{io::stdout, thread, time::Duration};

mod systems;

fn main() {
    initialize_terminal();

    let universe = Universe::new();
    let mut world = universe.create_world();

    let tcp_client = TcpClientResource::new("127.0.0.1:1119".parse().unwrap()).unwrap();
    let mut event_resource = EventResource::new();

    world.subscribe(event_resource.legion_subscriber().clone(), any());

    let mut resources = Resources::default();
    resources.insert(tcp_client);
    resources.insert(event_resource);
    resources.insert(SentBufferResource::new());
    resources.insert(Packer::<Bincode, Lz4>::default());
    resources.insert(RegisteredComponentsResource::new());

    initial_data(&mut world);

    let mut schedule = initialize_systems();

    loop {
        schedule.execute(&mut world, &mut resources);

        thread::sleep(Duration::from_millis(20));
    }
}

fn initialize_terminal() {
    //        simple_logger::init().unwrap();
    enable_raw_mode();
    stdout().execute(Hide);
}

fn initialize_systems() -> Schedule {
    Schedule::builder()
        .add_system(track_modifications_system())
        .add_system(tcp_sent_system::<Bincode, Lz4>())
        .add_system(move_player_system())
        .add_system(draw_player_system())
        .build()
}

fn initial_data(world: &mut World) -> &[Entity] {
    let mut uid_allocator = UidAllocator::new();

    world.insert(
        (),
        (0..1).map(|_| {
            (
                Position { x: 10, y: 10 },
                UidComponent::new(uid_allocator.allocate(Some(1))),
            )
        }),
    )
}
