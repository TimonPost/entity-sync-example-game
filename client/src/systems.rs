use legion::prelude::{IntoQuery, *};
use shared::{
    calculate_player_movement,
    components::{PlayerInfo, PlayerType, PlayerTypeOp, Position},
    create_texture_from_text, get_rect_from_text,
    message::{ClientCommand, ClientMessage, ServerMessage},
    ConnectionInformation, SCREEN_WIDTH,
};

use legion_sync::{
    components::UidComponent,
    resources::EventResource,
};
use net_sync::{
    synchronisation::{
        ClientCommandBuffer,  CommandFrameTicker, ResimulationBuffer,
    },
    tracker::Trackable,
    transport,
    transport::{ClientToServerMessage, PostBox, ServerToClientMessage},
    uid::UidAllocator,
};

use sdl2::{
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
};
use shared::systems::WindowResource;
use std::{collections::VecDeque};

pub fn move_player_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("move_player_system")
        .read_resource::<EventResource>()
        .read_resource::<CommandFrameTicker>()
        .write_resource::<ClientCommandBuffer<ClientCommand>>()
        .write_resource::<PostBox<
            transport::ServerToClientMessage<ServerMessage>,
            transport::ClientToServerMessage<ClientMessage, ClientCommand>,
        >>()
        .read_resource::<ConnectionInformation>()
        .write_resource::<PressedInputBuffer>()
        .with_query(<(
            legion::prelude::Write<Position>,
            Read<UidComponent>,
            Read<PlayerType>,
            Read<PlayerInfo>,
        )>::query())
        .build(|command, mut world, resource, query| {
            let event_resource = &resource.0;
            let command_frame_ticker: &CommandFrameTicker = &resource.1;
            let client_command_buffer: &mut ClientCommandBuffer<ClientCommand> = &mut resource.2;
            let connection_info: &ConnectionInformation = &resource.4;
            let input_buffer: &mut PressedInputBuffer = &mut resource.5;

            if let Some(key) = input_buffer.input.pop_front() {
                let command = match key {
                    Keycode::W => Some(ClientCommand::MoveUp),
                    Keycode::D => Some(ClientCommand::MoveRight),
                    Keycode::S => Some(ClientCommand::MoveDown),
                    Keycode::A => Some(ClientCommand::MoveLeft),
                    Keycode::Q => {
                        resource.3.send(ClientToServerMessage::Message(
                            ClientMessage::ConnectionRequest,
                        ));
                        None
                    }
                    _ => None,
                };

                for (mut pos, uid, player, info) in query.iter_mut(&mut world) {
                    if info.client_id() == connection_info.client_id() {
                        if let Some(command) = command.clone() {
                            if let calculated_pos =
                                calculate_player_movement(&command, pos.x, pos.y)
                            {
                                let mut pos = pos.client_track(
                                    client_command_buffer,
                                    command,
                                    **uid,
                                    command_frame_ticker.command_frame(),
                                );
                                pos.set(calculated_pos);
                            }
                        }
                    }
                }
            }
        })
}

pub fn handle_messages_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("hand_messages_system")
        .write_resource::<PostBox<
            ServerToClientMessage<ServerMessage>,
            ClientToServerMessage<ClientMessage, ClientCommand>,
        >>()
        .write_resource::<UidAllocator<Entity>>()
        .write_resource::<ConnectionInformation>()
        .write_resource::<CommandFrameTicker>()
        .build(|command, _, resource, _| {
            let postbox: &mut PostBox<
                ServerToClientMessage<ServerMessage>,
                ClientToServerMessage<ClientMessage, ClientCommand>,
            > = &mut resource.0;
            let connection_info: &mut ConnectionInformation = &mut resource.2;

            let inbox = postbox.drain_inbox(|m| match m {
                transport::ServerToClientMessage::Message(_) => true,
                _ => false,
            });

            for message in inbox {
                match message {
                    ServerToClientMessage::Message(message) => match message {
                        ServerMessage::ConnectionAccepted(clientId) => {
                            connection_info.set_connected(clientId);
                        }
                    },
                    _ => {}
                }
            }
        })
}

pub fn handle_resimulation() -> Box<dyn Schedulable> {
    SystemBuilder::new("handle_resimulation")
        .read_resource::<CommandFrameTicker>()
        .write_resource::<ResimulationBuffer<ClientCommand>>()
        .build(|command, mut world, resource, query| {
            let mut resimulating: &mut ResimulationBuffer<ClientCommand> = &mut resource.1;

            for entry in resimulating.iter() {
                println!(
                    "resimulating: from: {} to: {}, commands: {}",
                    entry.start_command_frame,
                    entry.end_command_frame,
                    entry.to_resimmulate.len()
                )
            }

            resimulating.entries.clear();
        })
}

pub fn client_render_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("client_render_system")
        .read_resource::<ConnectionInformation>()
        .write_resource::<WindowResource>()
        .read_resource::<CommandFrameTicker>()
        .write_resource::<PressedInputBuffer>()
        .with_query(<(
            Read<UidComponent>,
            legion::prelude::Read<Position>,
            Read<PlayerType>,
        )>::query())
        .build(|_, mut world, resources, query| {
            let connection_info: &ConnectionInformation = &resources.0;
            let mut window_resource: &mut WindowResource = &mut resources.1;
            let input_buffer: &mut PressedInputBuffer = &mut resources.3;
            let command_frame = resources.2.command_frame();

            let mut assets = find_folder::Search::ParentsThenKids(3, 3)
                .for_folder("assets")
                .unwrap();
            assets.push("FiraSans-Regular.ttf");

            let mut canvas = window_resource.window_lock().unwrap();
            let mut event_pump = window_resource.event_pump().unwrap();
            let mut tff_context = window_resource.tff().unwrap();
            let texture_creator = canvas.texture_creator();

            // Load a font
            let mut font = tff_context.load_font(assets, 50).unwrap();

            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();

            if connection_info.is_connected() {
                let command_frame_text = format!("CF: {}", command_frame);
                let client_id_text = format!("C Id: {}", connection_info.client_id());

                let rendered_text = create_texture_from_text(
                    &texture_creator,
                    &font,
                    &command_frame_text,
                    255,
                    0,
                    0,
                );
                canvas
                    .copy(
                        &rendered_text,
                        None,
                        Some(get_rect_from_text(
                            &command_frame_text,
                            SCREEN_WIDTH as i32 - 200,
                            100,
                        )),
                    )
                    .unwrap();

                let rendered_text =
                    create_texture_from_text(&texture_creator, &font, &client_id_text, 255, 0, 0);
                canvas
                    .copy(
                        &rendered_text,
                        None,
                        Some(get_rect_from_text(
                            &client_id_text,
                            SCREEN_WIDTH as i32 - 200,
                            120,
                        )),
                    )
                    .unwrap();

                // === Render Players
                for (id, pos, player_type) in query.iter(&mut world) {
                    if player_type.player_type() == PlayerTypeOp::Enemy {
                        canvas.set_draw_color(Color::RGB(255, 0, 0));
                        canvas.fill_rect(Rect::new(pos.x as i32, pos.y as i32, 50, 50));
                    } else {
                        canvas.set_draw_color(Color::RGB(0, 255, 0));
                        canvas.fill_rect(Rect::new(pos.x as i32, pos.y as i32, 50, 50));
                    }
                }
            }

            // === Read input
            for event in event_pump.poll_iter() {
                println!("event: {:?}", event);
                match event {
                    sdl2::event::Event::Quit { .. } => {}
                    sdl2::event::Event::KeyDown { keycode, .. } => match keycode {
                        Some(Keycode::W) | Some(Keycode::D) | Some(Keycode::S)
                        | Some(Keycode::Q) | Some(Keycode::A) => {
                            input_buffer.input.push_back(keycode.unwrap())
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }

            canvas.present()
        })
}

pub struct PressedInputBuffer {
    pub input: VecDeque<Keycode>,
}

impl PressedInputBuffer {
    pub fn new() -> PressedInputBuffer {
        PressedInputBuffer {
            input: VecDeque::<Keycode>::new(),
        }
    }
}
