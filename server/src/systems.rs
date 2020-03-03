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

pub fn remove_entities_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("read_received_system")
        .write_resource::<ReceiveBufferResource>()
        .write_resource::<TrackResource>()
        .with_query(<(Read<UidComponent>, legion::prelude::Write<Position>)>::query())
        .build(|command_buffer, mut world, resource, query| {
            let filter = query.clone().filter(removed(&resource.1));
            let removed_packets: Vec<ReceivedPacket> = resource.0.drain_removed();

            for (identifier, pos) in filter.iter_mut(&mut world) {
                for packet in removed_packets.iter() {
                    debug!("Removed entity {:?}", packet.event());
                }
            }
        })
}

pub fn apply_modifications_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("apply_modifications_system")
        .write_resource::<ReceiveBufferResource>()
        .write_resource::<TrackResource>()
        .read_resource::<RegisteredComponentsResource>()
        .with_query(<(Read<UidComponent>, legion::prelude::Write<Position>)>::query())
        .build(|command_buffer, mut world, resource, query| {
            let by_uid = resource.2.slice_with_uid();

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
                    }
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
                if let legion_sync::Event::EntityInserted(_entity_id, records) = packet.event() {
                    let mut entity_builder = command_buffer.start_entity().build();

                    debug!(
                        "Inserted entity {:?} with {:?} components",
                        _entity_id,
                        records.len()
                    );

                    for component in records {
                        let registered_components = resource.1.by_uid();
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
