use crossterm::{
    event,
    event::{Event, KeyCode, KeyEvent},
    style::Color,
};
use legion::prelude::{IntoQuery, *};
use legion_sync::{components::UidComponent, resources::EventResource};
use log::debug;
use net_sync::uid::UidAllocator;
use shared::components::{Coloring, PlayerType, PlayerTypeOp, Position};
use std::time::Duration;
use track::Trackable;

pub fn move_player_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("move_player_system")
        .read_resource::<EventResource>()
        .write_resource::<ClientState>()
        .with_query(<(
            legion::prelude::Write<Position>,
            Read<UidComponent>,
            Read<PlayerType>,
        )>::query())
        .build(|command, mut world, resource, query| {
            let new_pos = |event, x, y| -> (u16, u16) {
                match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Up, ..
                    }) => (x, y - 1),
                    Event::Key(KeyEvent {
                        code: KeyCode::Right,
                        ..
                    }) => (x + 1, y),
                    Event::Key(KeyEvent {
                        code: KeyCode::Down,
                        ..
                    }) => (x, y + 1),
                    Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        ..
                    }) => (x - 1, y),
                    _ => (x, y),
                }
            };

            if event::poll(Duration::from_millis(0)).unwrap() {
                let event = event::read().unwrap();

                for (mut pos, id, player) in query.iter_mut(&mut world) {
                    if player.player_type() == PlayerTypeOp::Player {
                        let mut pos = pos.track(resource.0.notifier(), id.uid());
                        let new_pos = new_pos(event, pos.x, pos.y);
                        pos.set(new_pos);
                    }
                }
            };

            if resource.1.frame % 25 == 0 {
                for (mut pos, id, player) in query.iter_mut(&mut world) {
                    if player.player_type() == PlayerTypeOp::Enemy {
                        debug!("iter enemies.");

                        debug!("track pos.");
                        let mut pos = pos.track(resource.0.notifier(), id.uid());

                        debug!("update pos.");
                        if pos.x == 0 {
                            pos.x = 40;
                        } else {
                            pos.x -= 1;
                        }
                    }
                }
            }

            if resource.1.frame % 200 == 0 {
                for (entity, (_, _, player)) in query.iter_entities_mut(&mut world) {
                    if player.player_type() == PlayerTypeOp::Enemy {
                        command.delete(entity);
                    }
                }
            }

            resource.1.frame += 1;
        })
}

pub fn insert_enemy_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("insert_enemy_system")
        .read_resource::<ClientState>()
        .write_resource::<UidAllocator<Entity>>()
        .build(|command, _, resource, _| {
            if resource.0.frame % 100 == 0 {
                let mut entity = command
                    .start_entity()
                    .with_component(Position::new(20, 15))
                    .with_component(Coloring::new(Color::Red))
                    .with_component(PlayerType::new(PlayerTypeOp::Enemy))
                    .build();

                let component = UidComponent::new(resource.1.allocate(entity, None));
                command.add_component(entity, component);
            }
        })
}

pub struct ClientState {
    pub frame: u32,
}

impl ClientState {
    pub fn new() -> ClientState {
        ClientState { frame: 0 }
    }
}
