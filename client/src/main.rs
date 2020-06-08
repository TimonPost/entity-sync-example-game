use legion::systems::schedule::Builder;
use shared::{
    message::{ClientCommand, ClientMessage, ServerMessage},
    systems::WindowResource,
    ConnectionInformation,
};

use legion_sync::{
    tracking::Bincode,
    world::{client::ClientWorldBuilder, WorldBuilder},
};
use net_sync::{
    clock::{FrameLimiter, FrameRateLimitStrategy},
    compression::lz4::Lz4,
};

use crate::systems::{
    client_render_system, handle_messages_system, handle_resimulation, move_player_system,
    PressedInputBuffer,
};

mod systems;

fn main() {
    initialize_terminal();

    let window = initialize_window();

    let mut client = ClientWorldBuilder::<ServerMessage, ClientMessage, ClientCommand>::default()
        .with_tcp::<Bincode, Lz4>("127.0.0.1:1119".parse().unwrap())
        .register_systems(initialize_main_systems)
        .with_resource(FrameLimiter::new(FrameRateLimitStrategy::Sleep, 30))
        .with_resource(window)
        .with_resource(ConnectionInformation::new())
        .with_resource(PressedInputBuffer::new())
        .build();

    loop {
        client.tick();

        // let mut limiter = resources.get_mut::<FrameLimiter>().unwrap();
        // limiter.wait();
    }
}

fn initialize_terminal() {
    simple_logger::init().unwrap();
}

fn initialize_window() -> WindowResource {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Client Entity Sync", 800, 800)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let event_pump = sdl_context.event_pump().unwrap();

    WindowResource::new(canvas, event_pump)
}

fn initialize_main_systems(builder: Builder) -> Builder {
    builder
        .add_system(client_render_system())
        .add_system(move_player_system())
        .add_system(handle_resimulation())
        .add_system(handle_messages_system())
        .flush()
}
