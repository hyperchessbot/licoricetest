use licorice::client::{Lichess};
use licorice::models::board::{Event, BoardState};

use tokio::stream::StreamExt;

use dotenv::dotenv;
use std::env;

use shakmaty::{Chess, Position};
use shakmaty::uci::Uci;
use shakmaty::fen;

fn _shakmaty(){
	let pos = Chess::default();
	
	let legals = &pos.legals();
	
	println!("legals startpos = {:?}", legals.iter().map(|m| Uci::from_standard(&m).to_string()).collect::<Vec<String>>());
	
	match pos.play(&legals[0]) {
		Ok(pos) => {
			println!("legals startpos e4 = {:?}", &pos.legals().iter().map(|m| Uci::from_standard(&m).to_string()).collect::<Vec<String>>());
			println!("fen = {:?}", fen::fen(&pos));
		},
		Err(err) => println!("{:?}", err)
	}	
}

#[tokio::main]
async fn main() {
	dotenv().ok();
	
	//_shakmaty();

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
					.stream_bot_game_state(&game.id)
					.await
					.unwrap();
				
				while let Some(game_event) = game_stream.next().await {
					println!("{:?}", game_event);
					let game_event = game_event.unwrap();
					
					match game_event {
						BoardState::GameFull ( game_full ) => {
							println!("game full {:?}", game_full);
						},
						_ => println!("unkown game event {:?}", game_event),
					};
				}
			}
			_ => println!("{:?}", event),
		};
    	
	}
}
