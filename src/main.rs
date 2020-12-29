use licorice::client::{Lichess};
use licorice::models::board::{Event};

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
	
	 let lichess = Lichess::new(std::env::var("RUST_BOT_TOKEN").unwrap());

	let _query_params = vec![("max", "1")];

	let mut event_stream = lichess
		.stream_incoming_events()
		.await
		.unwrap();

	while let Some(event) = event_stream.next().await {
    	let event = event.unwrap();
		
		match event {
			Event::Challenge { challenge } => {
				println!("incoming challenge {:?}", challenge.id);
				println!("accept response {:?}", lichess.challenge_accept(&challenge.id).await.unwrap());
			},
			Event::GameStart { game } => {
				println!("game started {:?}", game.id);
				
				let mut game_stream = lichess
					.stream_bot_game_state()
					.await
					.unwrap();
				
				while let Some(game_event) = game_stream.next().await {
					let game_event = game_event.unwrap();
					
					match game_event {
						GameFull { game } => {
							
						},
						_ => println!("unkown game event {:?}"),
					}
			}
			_ => println!("{:?}", event),
		};
    	
	}
}
