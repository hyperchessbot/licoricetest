use licorice::client::{Lichess};

use tokio::stream::StreamExt;

#[tokio::main]
async fn main() {
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
