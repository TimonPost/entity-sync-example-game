use legion::prelude::*;
use shared::components::Position;

use legion_sync::{components::UidComponent, resources::ReceiveBufferResource, ReceivedPacket};
use log::debug;
use track::Apply;

use legion::{
    command::WorldWritable,
    entity::{EntityAllocator, EntityBlock},
    filter::{ChunksetFilterData, Filter},
    storage::{ComponentTypeId, TagTypeId},
    world::{IntoComponentSource, TagLayout, TagSet},
};
use legion_sync::{
    filters::filter_fns::{modified, removed},
    register::ComponentRegister,
    resources::{Packer, RegisteredComponentsResource, RemovedEntities, TrackResource},
    tracking::{Bincode, SerializationStrategy},
};
use net_sync::{compression::CompressionStrategy, uid::Uid};
use std::{any::Any, sync::Arc};

pub fn entity_remove_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("read_received_system")
        .write_resource::<ReceiveBufferResource>()
        .write_resource::<TrackResource>()
        .write_resource::<RemovedEntities>()
        .with_query(<(Read<UidComponent>)>::query())
        .build(|command_buffer, mut world, resource, query| {
            let filter = query.clone().filter(removed(&resource.1));
            let removed_packets: Vec<ReceivedPacket> = resource.0.drain_removed();

            for (entity, identifier) in filter.iter_entities(&mut world) {
                command_buffer.delete(entity);
                resource.2.add(entity);
                debug!("Removed entity {:?}", removed_packets);
            }
        })
}

pub fn position_update_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("position_update_system")
        .write_resource::<ReceiveBufferResource>()
        .write_resource::<TrackResource>()
        .read_resource::<RegisteredComponentsResource>()
        .with_query(<(Read<UidComponent>, legion::prelude::Write<Position>)>::query())
        .build(|command_buffer, mut world, resource, query| {
            let filter = query.clone().filter(modified(&resource.1));

            for (entity, (identifier, mut pos)) in filter.iter_entities_mut(&mut world) {
                let uid = resource
                    .2
                    .get_uid(&pos.id().0)
                    .expect("Type should be registered, make sure to implement `sync` attribute.");

                let modified_packets: Vec<ReceivedPacket> =
                    resource.0.drain_modified(identifier.uid(), *uid);

                for packet in modified_packets.iter() {
                    if let legion_sync::Event::ComponentModified(_entity_id, record) =
                        packet.event()
                    {
                        Apply::apply_to(&mut *pos, &record.data(), Bincode);

                        debug!("Updating entity");
                    }
                }
            }
        })
}
