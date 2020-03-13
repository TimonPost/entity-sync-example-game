use legion::prelude::*;
use shared::components::Position;

use legion_sync::components::UidComponent;
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
    resources::{
        Packer, PostOfficeResource, RegisteredComponentsResource, RemovedEntities, TrackResource,
    },
    systems::SystemBuilderExt,
    tracking::{Bincode, SerializationStrategy},
};
use net_sync::{
    compression::CompressionStrategy,
    transport::{PostBox, PostOffice, ReceivedPacket},
    uid::{Uid, UidAllocator},
};
use std::{any::Any, sync::Arc};

pub fn entity_remove_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("read_received_system")
        .write_resource::<PostOfficeResource>()
        .write_resource::<RemovedEntities>()
        .read_resource::<UidAllocator<Entity>>()
        .build(|command_buffer, mut world, resource, query| {
            let mut postoffice: &mut PostOffice = &mut resource.0;
            let mut removed_entities: &mut RemovedEntities = &mut resource.1;
            let mut uid_allocator: &UidAllocator<Entity> = &resource.2;

            for (id, mut client) in postoffice.clients_mut().with_inbox() {
                let mut postbox = client.postbox_mut();
                if !postbox.empty_inbox() {
                    let removed: Vec<legion_sync::Event> = postbox.drain_inbox_removed();

                    for packet in removed {
                        match packet {
                            legion_sync::Event::EntityRemoved(entity_id) => {
                                let entity = uid_allocator.get_by_val(&entity_id.id());
                                command_buffer.delete(*entity);
                                removed_entities.add(*entity);
                                debug!("Removed entity {:?}", packet);
                            }
                            _ => panic!("Should only drain removed messages."),
                        }
                    }
                }
            }
        })
}

pub fn update_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("position_update_system")
        .write_registered_components()
        .write_resource::<PostOfficeResource>()
        .read_resource::<UidAllocator<Entity>>()
        .build(|command_buffer, mut world, resource, query| {
            let mut postoffice: &mut PostOffice = &mut resource.0;
            let mut uid_allocator: &UidAllocator<Entity> = &resource.1;

            for (id, mut client) in postoffice.clients_mut().with_inbox() {
                let mut postbox = client.postbox_mut();
                if !postbox.empty_inbox() {
                    let modified: Vec<legion_sync::Event> = postbox.drain_inbox_modified();

                    for packet in modified.iter() {
                        if let legion_sync::Event::ComponentModified(entity_id, record) = packet {
                            let entity = uid_allocator.get_by_val(&entity_id.id());

                            let mut pos = world
                                .get_component_mut::<Position>(*entity)
                                .expect("Component does not exist");
                            Apply::apply_to(&mut *pos, &record.data(), Bincode);

                            debug!("Updating entity");
                        }
                    }
                }
            }
        })
}
