use legion::prelude::{IntoQuery, Schedulable, SystemBuilder, Read};
use std::io::{stdout, Write};
use crossterm::QueueableCommand;
use crossterm::terminal::{ClearType, Clear};
use crossterm::cursor::MoveTo;
use crossterm::style::{Print, PrintStyledContent, style};
use legion_sync::components::UidComponent;
use crate::components::{Position, Coloring, PlayerType, PlayerTypeOp};

pub fn draw_player_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("draw_player_system")
        .with_query(<(Read<UidComponent>, legion::prelude::Write<Position>, Read<Coloring>, Read<PlayerType>)>::query())
        .build(|_, mut world, _, query| {
            let mut stdout = stdout();
            stdout.queue(Clear(ClearType::All));

            for (_, pos, color, player_type) in query.iter_mut(&mut world) {
                stdout.queue(MoveTo(pos.x, pos.y));

                if player_type.player_type() == PlayerTypeOp::Enemy {
                    stdout.queue(PrintStyledContent(style("W").with(color.color_type())));
                }else {
                    stdout.queue(PrintStyledContent(style("O").with(color.color_type())));
                }
                stdout.flush();
            }
        })
}

pub fn draw_entities() -> Box<dyn Schedulable> {
    SystemBuilder::new("draw_player_system")
        .with_query(<(Read<UidComponent>, legion::prelude::Read<Position>)>::query())
        .build(|_, mut world, _, query| {
            let mut stdout  = stdout();
//            stdout.queue(Clear(ClearType::All));
//            stdout.queue(MoveTo(0, 0));
            let mut count = 0;
            for (uid, pos) in query.iter_mut(&mut world) {
                count += 1;
                stdout.queue(Print(format!("{:?} {:?}\r\n", *uid, *pos)));
                stdout.flush();
            }
//            println!("count: {}\n\r", count);
        })
}