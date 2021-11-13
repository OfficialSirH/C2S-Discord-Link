use std::{env};

fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
}