mod actions;
mod actors;
mod app_states;
mod control;
mod events;
mod game;
mod gym;
mod level;
mod map;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, )]
struct Args {
    #[clap(short, long, default_value = "train")]
    mode: String,
}

fn main() {
    let args = Args::parse();

    let mut bevy_app = game::build_game_app(args.mode);
    bevy_app.run();
}
