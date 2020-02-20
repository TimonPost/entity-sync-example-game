use legion::prelude::*;
use shared::components::Position;

use crate::WorldExt;
use legion_sync::{components::UidComponent, resources::ReceiveBufferResource, ReceivedPacket};
use log::debug;
use track::{serialisation::bincode::Bincode, Apply};

use crate::change_filter::filter_fns::{modified, removed, inserted};
use legion::systems::SystemQuery;
use legion_sync::resources::TrackResource;

pub fn read_received_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("read_received_system")
        .write_resource::<ReceiveBufferResource>()
        .with_query(<(legion::prelude::Write<Position>, Read<UidComponent>)>::query())
        .build(|command_buffer, mut world, resource, mut query| {
            // filter takes self, therefore we need to clone
            let filter = query.clone().filter(modified(resource.tracking_cash()));

            for (mut pos, identifier) in filter.iter_mut(&mut world) {
                let packets: Vec<ReceivedPacket> = resource.drain(|event, id| match event { legion_sync::Event::Modified(_) => **identifier == id });

                for packet in packets {
                    if identifier.uid() == packet.identifier() {
                        Apply::apply_to(&mut *pos, &packet.data(), Bincode);
                        break;
                    }

                    debug!("Modified entity {:?}", packet.identifier());
                }
            }

            let filter = query.clone().filter(removed(resource.tracking_cash()));

            for (mut pos, identifier) in filter.iter_mut(&mut world) {
                let packets: Vec<ReceivedPacket> = resource.drain(|event, id| *event == legion_sync::Event::Removed && **identifier == id);

                for packet in packets {
                    debug!("Removed entity {:?}", packet.identifier());
                }
            }

            let filter = query.clone().filter(inserted(resource.tracking_cash()));

            for (mut pos, identifier) in filter.iter_mut(&mut world) {
                let packets: Vec<ReceivedPacket> = resource.drain(|event, id| match event { legion_sync::Event::Inserted(_) => **identifier == id });

                for packet in packets {
                    command_buffer.insert(
                        (),
                        vec![(
                            Position { x: 0, y: 0 },
                            UidComponent::new(packet.identifier().clone()),
                        )],
                    );

                    debug!("Inserted entity {:?}", packet.identifier());
                }
            }
        })
}
