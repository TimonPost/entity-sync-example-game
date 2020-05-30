use std::ops::DerefMut;

use legion::prelude::*;
use shared::{
    calculate_player_movement,
    components::{PlayerInfo, PlayerType, PlayerTypeOp, Position},
    create_texture_from_text, get_rect_from_text,
    message::{ClientCommand, ClientMessage, ServerMessage},
    SCREEN_WIDTH,
};

use legion_sync::{components::UidComponent, resources::EventResource};
use log::debug;
use net_sync::{
    synchronisation::{CommandFrameTicker, ModifiedComponentsBuffer},
    tracker::Trackable,
    transport::{PostOffice, ServerToClientMessage},
    uid::UidAllocator,
};
use sdl2::{pixels::Color, rect::Rect};
use shared::systems::WindowResource;

pub fn enemy_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("enemy_system")
        .read_resource::<EventResource>()
        .write_resource::<UidAllocator<Entity>>()
        .read_resource::<CommandFrameTicker>()
        .with_query(<(
            legion::prelude::Write<Position>,
            Read<UidComponent>,
            Read<PlayerType>,
        )>::query())
        .build(|command, mut world, resource, query| {
            if resource.2.command_frame() % 100 == 0 {
                let entity = command
                    .start_entity()
                    .with_component(Position::new(20, 15))
                    .with_component(PlayerType::new(PlayerTypeOp::Enemy))
                    .build();

                let component = UidComponent::new(resource.1.allocate(entity, None));
                command.add_component(entity, component);
            }

            // if resource.2.command_frame() % 25 == 0 {
            //     for (mut pos, id, player) in query.iter_mut(&mut world) {
            //         if player.player_type() == PlayerTypeOp::Enemy {
            //             let mut pos = pos.track(resource.0.notifier(), id.uid());
            //
            //             if pos.x == 0 {
            //                 pos.x = 40;
            //             } else {
            //                 pos.x -= 1;
            //             }
            //         }
            //     }
            // }

            if resource.2.command_frame() % 300 == 0 {
                for (entity, (_, _, player)) in query.iter_entities_mut(&mut world) {
                    if player.player_type() == PlayerTypeOp::Enemy {
                        command.delete(entity);
                    }
                }
            }
        })
}

pub fn handle_messages_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("hand_messages_system")
        .write_resource::<PostOffice<ServerMessage, ClientMessage, ClientCommand>>()
        .write_resource::<UidAllocator<Entity>>()
        .build(|command, _, resource, _| {
            let postoffice: &mut PostOffice<ServerMessage, ClientMessage, ClientCommand> =
                &mut resource.0;
            let entity_builder: &mut UidAllocator<Entity> = &mut resource.1;

            for (_id, client) in postoffice.clients_with_inbox() {
                let client_id = client.client_id();
                let postbox = client.postbox_mut();

                for message in postbox.drain_inbox(|_| true) {
                    match message {
                        ClientMessage::ConnectionRequest => {
                            debug!("Connection Request");

                            let builder = command
                                .start_entity()
                                .with_component(Position { x: 200, y: 200 })
                                .with_component(PlayerType::new(PlayerTypeOp::Player))
                                .with_component(PlayerInfo::new(client_id));

                            let entity = builder.build();

                            let id = entity_builder.allocate(entity, None);

                            let uid = UidComponent::new(id);
                            command.add_component(entity, uid);

                            postbox.send(ServerToClientMessage::Message(
                                ServerMessage::ConnectionAccepted(client_id),
                            ));
                        }
                        ClientMessage::Disconnect => {
                            debug!("Disconnect Request");
                            // remove player
                        }
                    }
                }
            }
        })
}

pub fn handle_commands_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("player_move_system")
        .with_query(<(
            legion::prelude::Write<Position>,
            Read<PlayerInfo>,
            Read<UidComponent>,
        )>::query())
        .write_resource::<PostOffice<ServerMessage, ClientMessage, ClientCommand>>()
        .read_resource::<CommandFrameTicker>()
        .write_resource::<ModifiedComponentsBuffer>()
        .build(|_, world, resource, query| {
            let postoffice: &mut PostOffice<ServerMessage, ClientMessage, ClientCommand> =
                &mut resource.0;
            let command_frame_ticker: &CommandFrameTicker = &resource.1;
            let modified_components_buffer: &mut ModifiedComponentsBuffer = &mut resource.2;

            for (client_id, client) in postoffice.clients_mut() {
                let postbox = client.command_postbox_mut();
                let messages = postbox.drain_frame(command_frame_ticker.command_frame());

                if let Some(messages) = messages {
                    for message in messages {
                        match message.command {
                            ClientCommand::MoveUp
                            | ClientCommand::MoveRight
                            | ClientCommand::MoveDown
                            | ClientCommand::MoveLeft => {
                                debug!("Handling Move Command: {:?}", message.command);

                                for (mut pos, player_info, uid) in query.iter_mut(world) {
                                    let mut pos = pos.server_track(
                                        modified_components_buffer,
                                        **uid,
                                        command_frame_ticker.command_frame(),
                                    );

                                    if player_info.client_id() == *client_id {
                                        let new_pos = calculate_player_movement(
                                            &message.command,
                                            pos.x,
                                            pos.y,
                                        );
                                        pos.deref_mut().set(new_pos);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        })
}

pub fn render_server() -> Box<dyn Schedulable> {
    SystemBuilder::new("draw_player_system")
        .read_resource::<PostOffice<ServerMessage, ClientMessage, ClientCommand>>()
        .write_resource::<WindowResource>()
        .read_resource::<CommandFrameTicker>()
        .with_query(<(
            Read<UidComponent>,
            legion::prelude::Read<Position>,
            Read<PlayerType>,
        )>::query())
        .build(|_, mut world, resources, query| {
            let postoffice: &PostOffice<ServerMessage, ClientMessage, ClientCommand> = &resources.0;
            let window_resource: &mut WindowResource = &mut resources.1;

            let mut assets = find_folder::Search::ParentsThenKids(3, 3)
                .for_folder("assets")
                .unwrap();
            assets.push("FiraSans-Regular.ttf");

            let mut canvas = window_resource.window_lock().unwrap();
            let tff_context = window_resource.tff().unwrap();
            let texture_creator = canvas.texture_creator();

            // Load a font
            let font = tff_context.load_font(assets, 50).unwrap();

            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();

            static LEFT_MARIGIN: i32 = SCREEN_WIDTH as i32 - 300;

            let mut y_offset = 0;
            for client in postoffice.clients() {
                let client_id_text = format!("PORT: {}", client.1.addr().port());
                let client_offset_text = format!(
                    "offset: {}",
                    client.1.command_postbox().command_frame_offset()
                );
                let client_highest_text =
                    format!("highest: {}", client.1.command_postbox().highest_seen());

                let rendered_text =
                    create_texture_from_text(&texture_creator, &font, &client_id_text, 255, 0, 0);
                canvas
                    .copy(
                        &rendered_text,
                        None,
                        Some(get_rect_from_text(&client_id_text, LEFT_MARIGIN, y_offset)),
                    )
                    .unwrap();

                y_offset += 50;

                let rendered_text = create_texture_from_text(
                    &texture_creator,
                    &font,
                    &client_offset_text,
                    255,
                    0,
                    0,
                );
                canvas
                    .copy(
                        &rendered_text,
                        None,
                        Some(get_rect_from_text(&client_id_text, LEFT_MARIGIN, y_offset)),
                    )
                    .unwrap();

                y_offset += 50;

                let rendered_text = create_texture_from_text(
                    &texture_creator,
                    &font,
                    &client_highest_text,
                    255,
                    0,
                    0,
                );
                canvas
                    .copy(
                        &rendered_text,
                        None,
                        Some(get_rect_from_text(&client_id_text, LEFT_MARIGIN, y_offset)),
                    )
                    .unwrap();

                y_offset += 100;
            }

            let command_frame_text = format!("CF: {}", resources.2.command_frame());
            let rendered_text =
                create_texture_from_text(&texture_creator, &font, &command_frame_text, 255, 0, 0);
            canvas
                .copy(
                    &rendered_text,
                    None,
                    Some(get_rect_from_text(
                        &command_frame_text,
                        LEFT_MARIGIN,
                        y_offset,
                    )),
                )
                .unwrap();

            // === Render Players
            for (_id, pos, player_type) in query.iter(&mut world) {
                if player_type.player_type() == PlayerTypeOp::Enemy {
                    canvas.set_draw_color(Color::RGB(255, 0, 0));
                    canvas.fill_rect(Rect::new(pos.x as i32, pos.y as i32, 50, 50));
                } else {
                    canvas.set_draw_color(Color::RGB(0, 255, 0));
                    canvas.fill_rect(Rect::new(pos.x as i32, pos.y as i32, 50, 50));
                }
            }

            canvas.present()
        })
}
