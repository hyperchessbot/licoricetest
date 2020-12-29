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

	let mut stream = lichess
		.stream_incoming_events()
		.await
		.unwrap();

	while let Some(event) = stream.next().await {
    	let event = event.unwrap();
		match event {
			Event::Challenge{challenge} => {
				println!("incoming challenge {:?}", challenge.id);
				println!("accept response {:?}", lichess.challenge_accept(&challenge.id).await.unwrap());
			},
			_ => println!("{:?}", event),
		};
    	
	}
}
