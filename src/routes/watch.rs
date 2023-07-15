use axum::{
	extract::State,
	response::sse::{Event, Sse},
};
use futures::{stream::Stream, SinkExt, StreamExt};
use notify::Watcher;
use pathdiff::diff_paths;
use serde_json::json;
use std::{convert::Infallible, fs::canonicalize, path, time};

pub async fn watch(
	State(state): State<crate::State>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
	Sse::new(async_stream::try_stream! {
		let (mut tx, mut rx) = futures::channel::mpsc::channel(1);
		let mut watcher = notify::RecommendedWatcher::new(
			move |res| {
				futures::executor::block_on(async {
					tx.send(res).await.expect("should send");
				})
			},
			notify::Config::default(),
		).expect("watcher should be created");

		watcher.watch(path::Path::new(state.args.watch.as_str()), notify::RecursiveMode::Recursive).expect("watcher should watch");

		while let Some(res) = rx.next().await {
			match res {
				Ok(event) => {
					let hrefs = event.paths.iter().map(|p| {
						let c = canonicalize(state.args.watch.as_str()).expect("path should be valid");

						diff_paths(p, c).map(|p| {
							let p = p.to_str().expect("path should be a string");

							format!("/{}", p)
						})
					}).collect::<Vec<_>>();

					yield Event::default().data(json!({
						"hrefs": hrefs,
					}).to_string());
				},
				Err(e) => println!("watch error: {:?}", e),
			}
		}
	})
	.keep_alive(
		axum::response::sse::KeepAlive::new()
			.interval(time::Duration::from_secs(10))
			.text("keep-alive-text"),
	)
}
