use std::net::{SocketAddr, TcpListener};

use legion::systems::schedule::Builder;
use shared::{
    message::{ClientCommand, ClientMessage, ServerMessage},
    systems::WindowResource,
};

use legion_sync::{
    tracking::Bincode,
    world::{server::ServerWorldBuilder, WorldBuilder},
};
use net_sync::{
    clock::{FrameLimiter, FrameRateLimitStrategy},
    compression::lz4::Lz4,
};

use crate::systems::{handle_commands_system, handle_messages_system, render_server};
use net_sync::synchronisation::ModifiedComponentsBuffer;

mod systems;

fn main() {
    initialize_terminal();

    let window = initialize_window();

    let tcp_listener = TcpListener::bind("127.0.0.1:1119".parse::<SocketAddr>().unwrap()).unwrap();

    let mut server = ServerWorldBuilder::<ServerMessage, ClientMessage, ClientCommand>::default()
        .with_tcp::<Bincode, Lz4>(tcp_listener)
        .register_systems(initialize_systems)
        .with_resource(FrameLimiter::new(FrameRateLimitStrategy::Yield, 30))
        .with_resource(window)
        .with_resource(ModifiedComponentsBuffer::new())
        .build();

    loop {
        server.tick();

        let resources = server.resources();

        // let mut limiter = resources.get_mut::<FrameLimiter>().unwrap();
        // limiter.wait();
    }
}

fn initialize_window() -> WindowResource {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Server Entity Sync", 800, 800)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let event_pump = sdl_context.event_pump().unwrap();

    WindowResource::new(canvas, event_pump)
}

fn initialize_terminal() {
    simple_logger::init().unwrap();
}

fn initialize_systems(builder: Builder) -> Builder {
    builder
        .add_system(render_server())
        //                .add_system(enemy_system())
        .add_system(handle_messages_system())
        .add_system(handle_commands_system())
        .flush()
}
