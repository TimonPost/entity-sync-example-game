use crossterm::{
    event,
    event::{Event, KeyCode, KeyEvent},
    QueueableCommand,
};
use legion::prelude::{IntoQuery, *};
use legion_sync::{components::UuidComponent, resources::EventListenerResource};
use log::debug;
use shared::components::Position;
use std::time::Duration;
use track::Trackable;

pub fn move_player_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("move player")
        .read_resource::<EventListenerResource>()
        .with_query(<(legion::prelude::Write<Position>, Read<UuidComponent>)>::query())
        .build(|_, mut world, resource, query| {
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

                debug!("{:?}", event);

                for (mut pos, uuid) in query.iter_mut(&mut world) {
                    let mut pos = pos.track_by(resource.notifier(), uuid.uuid());
                    let new_pos = new_pos(event, pos.x, pos.y);
                    pos.set(new_pos);
                }
            };

            debug!("client tick");
        })
}
