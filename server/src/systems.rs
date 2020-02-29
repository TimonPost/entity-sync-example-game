use legion::prelude::*;
use shared::components::Position;

use legion_sync::{components::UidComponent, resources::ReceiveBufferResource, ReceivedPacket};
use log::debug;
use track::Apply;

use legion::command::WorldWritable;
use legion::entity::{EntityAllocator, EntityBlock};
use legion::filter::{ChunksetFilterData, Filter};
use legion::storage::{ComponentTypeId, TagTypeId};
use legion::world::{IntoComponentSource, TagLayout, TagSet};
use legion_sync::register::ComponentRegister;
use legion_sync::resources::{Packer, RegisteredComponentsResource};
use legion_sync::tracking::{Bincode, SerializationStrategy};
use legion_sync::{
    filters::filter_fns::{modified, removed},
    resources::TrackResource,
};
use net_sync::compression::CompressionStrategy;
use net_sync::uid::Uid;
use std::any::Any;
use std::sync::Arc;

pub fn read_received_system<
    S: SerializationStrategy + 'static,
    C: CompressionStrategy + 'static,
>() -> Box<dyn Schedulable> {
    SystemBuilder::new("read_received_system")
        .write_resource::<ReceiveBufferResource>()
        .write_resource::<TrackResource>()
        .read_resource::<Packer<S, C>>()
        .with_query(<(Read<UidComponent>, legion::prelude::Write<Position>)>::query())
        .build(|command_buffer, mut world, resource, query| {
            // filter takes self, therefore we need to clone
            let filter = query.clone().filter(modified(&resource.1));

            //            let collection: ChunkDataIter<'_, V, ChunkViewIter<'_, '_, V, F::ArchetypeFilter, F::ChunksetFilter, F::ChunkFilter>> = filter.iter_mut(&mut world).collect();

            for (identifier, mut pos) in filter.iter_mut(&mut world) {
                let modified_packets: Vec<ReceivedPacket> = resource.0.drain_modified();

                for packet in modified_packets.iter() {
                    if let legion_sync::Event::Modified(records) = packet.event() {
                        if identifier.uid() == packet.identifier() {
                            Apply::apply_to(&mut *pos, &records, Bincode);
                            break;
                        }

                        debug!("Modified entity {:?}", packet.identifier());
                    }
                }
            }

            let filter = query.clone().filter(removed(&resource.1));
            let removed_packets: Vec<ReceivedPacket> = resource.0.drain_removed();

            for (identifier, pos) in filter.iter_mut(&mut world) {
                for packet in removed_packets.iter() {
                    debug!("Removed entity {:?}", packet.identifier());
                }
            }
        })
}

pub fn insert_received_entities_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("insert_received_entities_system")
        .write_resource::<ReceiveBufferResource>()
        .read_resource::<RegisteredComponentsResource>()
        .build(|command_buffer, mut world, resource, _| {
            let inserted_packets: Vec<ReceivedPacket> = resource.0.drain_inserted();

            for packet in inserted_packets.iter() {
                if let legion_sync::Event::Inserted(records) = packet.event() {
                    let mut entity_builder = command_buffer.start_entity().build();

                    debug!(
                        "Inserted entity {:?} with {:?} components",
                        packet.identifier(),
                        records.len()
                    );

                    for component in records {
                        let registered_components = resource.1.hashmap();
                        let registered_component = registered_components
                            .get(&Uid(component.register_id()))
                            .unwrap();

                        registered_component.deserialize_single(
                            world,
                            command_buffer,
                            entity_builder.clone(),
                            &component.data(),
                        );
                        debug!(
                            "Added component {:?} to entity",
                            registered_component.type_name()
                        );
                    }
                }
            }
        })
}
