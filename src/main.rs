use licorice::client::{Lichess};
use licorice::models::board::{Event, BoardState, GameState};

use tokio::stream::StreamExt;

use dotenv::dotenv;
use std::env;

use shakmaty::{Chess, Position, Move, Color};
use shakmaty::uci::{Uci, ParseUciError, IllegalUciError};
use shakmaty::san::{San};
use shakmaty::fen;
use shakmaty::fen::Fen;

use pgn_reader::{Visitor, Skip, RawHeader, BufferedReader, SanPlus};

use rand::prelude::*;

use thiserror::Error;

use ring::{digest};

//////////////////////////////////////////////////////////////////
// MongoDb
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::{doc, Document, Bson};

async fn _connect() -> Result<Client, Box<dyn std::error::Error>>{
	// Parse a connection string into an options struct.
	let client_options = ClientOptions::parse(&std::env::var("RUST_MONGODB_URI").unwrap()).await?;

	// Get a handle to the deployment.
	let client = Client::with_options(client_options)?;

	// List the names of the databases in that deployment.
	for db_name in client.list_database_names(None, None).await? {
		println!("db {}", db_name);
	}
	
	println!("mongodb connected ok");
	
	Ok(client)
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
	if ucis_str.len() > 0 {
		for uci_str in ucis_str.split(" ") {
			let uci: Uci = uci_str.parse()?;						
			let m = uci.to_move(&pos.to_owned())?;		
			match pos.to_owned().play(&m) {
				Ok(newpos) => pos = newpos,
				Err(_) => return Err(Box::new(IllegalUciError)),
			}		
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

async fn _stream_events() -> Result<(), Box<dyn std::error::Error>> {
	let lichess = Lichess::new(std::env::var("RUST_BOT_TOKEN").unwrap());
	
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
				
				let mut game_id = String::new();
				
				let mut bot_white = true;					
				
				while let Some(game_event) = game_stream.next().await {
					println!("{:?}", game_event);
					let game_event = game_event.unwrap();
					
					let white:String;
					let black:String;
					let bot = std::env::var("RUST_BOT_NAME").unwrap();					
					
					let mut state:Vec<GameState> = vec!();
					
					match game_event {
						BoardState::GameFull ( game_full ) => {
							game_id = game_full.id;
							
							println!("game id {}", game_id);
							
							white = game_full.white.username;
							black = game_full.black.username;
							
							if bot == black {
								bot_white = false;
							}
							
							state.push(game_full.state);
							
							println!("{} - {} bot white {}", white, black, bot_white);
						},
						BoardState::GameState ( game_state ) => {
							state.push(game_state);
						},
						_ => println!("unkown game event {:?}", game_event),
					};
					
					let state = state.pop().unwrap();
					
					let fen = _make_uci_moves(state.moves.as_str())?;
						
					println!("fen {}", fen);
					
					let setup: Fen = fen.parse()?;
					let pos: Chess = setup.position(shakmaty::CastlingMode::Standard)?;
					
					let legals = pos.legals();
		
					let rand_move = legals.choose(&mut rand::thread_rng()).unwrap();
					
					let rand_uci = Uci::from_standard(&rand_move).to_string();
					
					println!("rand uci {}", rand_uci);
					
					let turn = setup.turn;
					
					println!("turn {:?}", turn);
					
					let bot_turn = ( ( turn == Color::White ) && bot_white ) || ( ( turn == Color::Black ) && !bot_white );
					
					println!("bot turn {}", bot_turn);
					
					if bot_turn {
						let move_uci = rand_uci;
						
						println!("making move {}", move_uci);
						
						let id = game_id.to_owned();
						
						let result = lichess.make_a_bot_move(id.as_str(), move_uci.as_str(), false).await;
						
						println!("make move result {:?}", result);
					}
				}
			}
			_ => println!("{:?}", event),
		};    	
	}
	
	Ok(())
}

fn _print_env_vars() {
	for (key, value) in env::vars() {
		match &key[..std::cmp::min(5, key.len())] {
			"RUST_" => println!("{}: {}", key, value),
			_ => {},
		};
    }
}

#[derive(Debug)]
struct PgnWithDigest {	
	pgn_str: String,
	sha256_base64: String,
}

impl From<PgnWithDigest> for Document {
	fn from(pgn_with_digest: PgnWithDigest) -> Self {
        doc!{"_id": pgn_with_digest.sha256_base64, "pgn": pgn_with_digest.pgn_str}
    }
}

impl From<Document> for PgnWithDigest {
	fn from(document: Document) -> Self {
        PgnWithDigest{
			pgn_str: document.get("pgn").and_then(Bson::as_str).unwrap_or("").to_string(),
			sha256_base64: document.get("_id").and_then(Bson::as_str).unwrap_or("").to_string(),
		}
    }
}

impl std::fmt::Display for PgnWithDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("pgn = {}\nsha256(base64) = {}", self.pgn_str, self.sha256_base64))
    }
}


impl From<&str> for PgnWithDigest {
	fn from(pgn_str: &str) -> Self {
		PgnWithDigest {
			pgn_str: pgn_str.to_string(),
			sha256_base64: base64::encode(digest::digest(&digest::SHA256, pgn_str.as_bytes()).as_ref()),
		}
	}
}

struct LastPosition {
    pos: Chess,
}

impl LastPosition {
    fn new() -> LastPosition {
        LastPosition { pos: Chess::default() }
    }
}

impl Visitor for LastPosition {
    type Result = Chess;

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        // Support games from a non-standard starting position.
        if key == b"FEN" {
            let pos = Fen::from_ascii(value.as_bytes()).ok()
                .and_then(|f| f.position(shakmaty::CastlingMode::Standard).ok());

            if let Some(pos) = pos {
                self.pos = pos;
            }
        }
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true) // stay in the mainline
    }

    fn san(&mut self, san_plus: SanPlus) {
		let san_orig = san_plus.san;
		let san_str = format!("{}", san_orig);        
		let san_result:std::result::Result<San, _> = san_str.parse();
		match san_result {
			Ok(san) => {
				let move_result = san.to_move(&self.pos);
				
				match move_result {
					Ok(m) => {
						self.pos.play_unchecked(&m);
					},
					_ => println!("{:?}", move_result)
				}				
			},
			_ => println!("{:?}", san_result)
		}		
    }

    fn end_game(&mut self) -> Self::Result {
        ::std::mem::replace(&mut self.pos, Chess::default())
    }
}

async fn _get_games_pgn() -> Result<(), Box<dyn std::error::Error>> {
	let lichess = Lichess::new(std::env::var("RUST_BOT_TOKEN").unwrap());

	let _query_params = vec![("max", "2")];
	
	let mut stream = lichess
         .export_all_games_pgn("chesshyperbot", Some(&_query_params))
         .await
         .unwrap();
	
	let client = _connect().await?;
	
	let db = client.database("rustbook");
	let pgns = db.collection("pgns");
	
	//println!("{:?}", pgns.drop(None).await);
	
	let mut all_pgn = String::new();

	while let Some(pgn_bytes_result) = stream.next().await {				
		let pgn_bytes_chunk = pgn_bytes_result?;
		println!("received item {}", pgn_bytes_chunk.len());		
		all_pgn = all_pgn + std::str::from_utf8(&pgn_bytes_chunk)?;
	}
	
	let mut items:Vec<&str> = all_pgn.split("\n\n\n").collect();	
	let _ = items.pop();
	
	for pgn_str in items {
		let old_pgn_str = pgn_str.to_owned();
		
		let pgn_with_digest:PgnWithDigest = pgn_str.into();
		
		println!("processing pgn with sha {}", pgn_with_digest.sha256_base64);
		
		let result = pgns.find_one(doc!{"_id": pgn_with_digest.sha256_base64.to_owned()}, None).await;
		
		match result {
			Ok(Some(doc)) => {
				let pgn_with_digest_stored:PgnWithDigest = doc.into();
				println!("pgn already in db {}", pgn_with_digest_stored.sha256_base64)
			},
			_ => {
				println!("pgn not in db, inserting");
				
				let result = pgns.insert_one(pgn_with_digest.into(), None).await;
		
				match result {
					Ok(_) => println!("pgn inserted ok"),
					Err(err) => println!("inserting pgn failed {:?}", err)
				}
			}
		}
		
		let pgn_bytes = old_pgn_str.as_bytes();
		
		let mut reader = BufferedReader::new_cursor(&pgn_bytes);

		let mut visitor = LastPosition::new();
		let pos = reader.read_game(&mut visitor)?;
		
		println!("final pos {:?}", pos);
	}
	
	Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv().ok();
	
	//_shakmaty();	
	//let _ = _shakmaty_official();
	//println!("{}", make_uci_moves("e2e4 e7e5 g1f3")?);
	//_connect().await?;
	//_print_env_vars();
	let _ = _get_games_pgn().await;
	//let _ = _stream_events().await;
	
	Ok(())
}
