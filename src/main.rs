use licorice::client::{Lichess};

use tokio::stream::StreamExt;

use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
	dotenv().ok();

    for (key, value) in env::vars() {
		match &key[..std::cmp::min(5, key.len())] {
			"RUST_" => println!("{}: {}", key, value),
			_ => {},
		};
    }
	
	let lichess = Lichess::default();

	let query_params = vec![("max", "1")];

	let mut stream = lichess
		.export_all_games_json("chesshyperbot", Some(&query_params))
		.await
		.unwrap();

	while let Some(game) = stream.next().await {
    	let game = game.unwrap();
    	println!("{:?}", game.id);
	}
}
