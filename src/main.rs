use licorice::client::{Lichess};
use licorice::models::board::{Event, BoardState};

use tokio::stream::StreamExt;

use dotenv::dotenv;
use std::env;

use shakmaty::{Chess, Position, Move};
use shakmaty::uci::{Uci, ParseUciError, IllegalUciError};
use shakmaty::fen;

use rand::prelude::*;

use thiserror::Error;

//////////////////////////////////////////////////////////////////
// MongoDb
use mongodb::{Client, options::ClientOptions};

async fn _connect() -> Result<(), Box<dyn std::error::Error>>{
	// Parse a connection string into an options struct.
	let client_options = ClientOptions::parse(&std::env::var("RUST_MONGODB_URI").unwrap()).await?;

	// Get a handle to the deployment.
	let client = Client::with_options(client_options)?;

	// List the names of the databases in that deployment.
	for db_name in client.list_database_names(None, None).await? {
		println!("{}", db_name);
	}
	
	Ok(())
}
//////////////////////////////////////////////////////////////////

/// Error when trying to play an illegal move.
#[derive(Debug)]
pub struct PlayError<'a, P> {
    m: &'a Move,
    inner: P,
}

impl<P> PlayError<'_, P> {
    pub fn into_inner(self) -> P {
        self.inner
    }
}

#[derive(Error, Debug)]
enum UciError {
	#[error("parse uci error")]
	ParseUciError(#[from] ParseUciError),
	#[error("illegal uci error")]
	IllegalUciError(#[from] IllegalUciError),
}

fn _shakmaty_official() -> Result<(), Box<dyn std::error::Error>> {
	let uci: Uci = "g1f3".parse()?;
	let pos = Chess::default();
	let m = uci.to_move(&pos)?;
	println!("move {}", m);
	Ok(())
}

fn _make_uci_moves(ucis_str: &str) -> Result<String, Box<dyn std::error::Error>> {
	let mut pos = Chess::default();
	for uci_str in ucis_str.split(" ") {
		let uci: Uci = uci_str.parse()?;						
		let m = uci.to_move(&pos.to_owned())?;		
		match pos.to_owned().play(&m) {
			Ok(newpos) => pos = newpos,
			Err(_) => return Err(Box::new(IllegalUciError)),
		}		
	}
	Ok(fen::fen(&pos))
}

fn _shakmaty(){
	let pos = Chess::default();
	
	match "g1f3".parse::<Uci>() {
		Ok(uci) => {
			match uci.to_move(&pos) {
				Ok(m) => match pos.play(&m) {
					Ok(pos) => {											
						println!("legals startpos g1f3 = {:?}", &pos.legals().iter().map(|m| Uci::from_standard(&m).to_string()).collect::<Vec<String>>());
						println!("fen = {}", fen::fen(&pos));
					},
					Err(err) => println!("{:?}", err)
				}, 
				Err(err) => println!("{:?}", err)
			}
		},
		Err(err) => println!("{:?}", err)
	}
	
	let pos = Chess::default();
	
	let legals = pos.legals();
		
	let rand_move = legals.choose(&mut rand::thread_rng()).unwrap();
	
	match pos.play(&rand_move) {
		Ok(pos) => {
			println!("legals startpos {} = {:?}", Uci::from_standard(&rand_move).to_string(), &pos.legals().iter().map(|m| Uci::from_standard(&m).to_string()).collect::<Vec<String>>());
			println!("fen = {}", fen::fen(&pos));
		},
		Err(err) => println!("{:?}", err)
	}	
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv().ok();
	
	//_shakmaty();	
	//let _ = _shakmaty_official();
	//println!("{}", make_uci_moves("e2e4 e7e5 g1f3")?);
	_connect().await?;

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
	
	Ok(())
}
