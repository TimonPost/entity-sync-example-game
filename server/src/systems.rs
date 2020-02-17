use crossterm::{
    cursor::MoveTo,
    style::Print,
    terminal::{Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use legion::prelude::*;
use shared::components::Position;
use std::io::{stdout, Write};

use legion_sync::components::UuidComponent;
use legion_sync::resources::ServerUniverseResource;
use legion_sync::NetworkPacket;
use log::debug;
use net_sync::compression::lz4::Lz4;
use net_sync::compression::CompressionStrategy;
use track::serialisation::SerialisationStrategy;
use track::{serialisation::bincode::Bincode, Apply, ModificationChannel, ModificationEvent};

pub fn draw_player_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("track to server")
        .with_query(<(legion::prelude::Write<Position>, Read<UuidComponent>)>::query())
        .build(|_, mut world, mut client_resource, query| {
            for (mut pos, uuid) in query.iter_mut(&mut world) {
                let mut stdout = stdout();
                stdout.queue(Clear(ClearType::All));
                stdout.queue(MoveTo(pos.x, pos.y));
                stdout.queue(Print("X"));
                stdout.flush();
            }
        })
}

pub fn server_recv_sync_system<
    S: SerialisationStrategy + 'static,
    C: CompressionStrategy + 'static,
>() -> Box<dyn Schedulable> {
    SystemBuilder::new("track to server")
        .write_resource::<ServerUniverseResource<S, C>>()
        .with_query(<(legion::prelude::Write<Position>, Read<UuidComponent>)>::query())
        .build(|command_buffer, mut world, server_resource, query| {
            if let Some(packets) = server_resource.try_receive() {
                let packets: Vec<NetworkPacket> = packets;

                for packet in packets {
                    match packet.event_type {
                        legion_sync::Event::Inserted => {
                            let entity = command_buffer.insert(
                                (),
                                vec![(Position { x: 0, y: 0 }, UuidComponent::from(packet.uuid))],
                            );

                            debug!("Inserted entity {:?}", packet.uuid);
                        }
                        legion_sync::Event::Modified => {
                            for (mut pos, uuid) in query.iter_mut(&mut world) {
                                if uuid.uuid() == packet.uuid {
                                    Apply::apply_to(&mut *pos, &packet.data.unwrap(), Bincode);
                                    break;
                                }
                            }

                            debug!("Modified entity {:?}", packet.uuid);
                        }
                        legion_sync::Event::Removed => {
                            debug!("Removed entity {:?}", packet.uuid);
                        }
                    }
                }
            }

            debug!("server tick");
        })
}
