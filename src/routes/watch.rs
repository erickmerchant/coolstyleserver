use crate::args::Commands;
use async_stream::try_stream;
use axum::{
	extract::State,
	response::sse::{Event, KeepAlive, Sse},
};
use camino::Utf8Path;
use futures::{channel::mpsc::channel, executor::block_on, stream::Stream, SinkExt, StreamExt};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use pathdiff::diff_paths;
use serde_json::json;
use std::{convert::Infallible, fs::canonicalize, path, sync::Arc, time::Duration};

pub async fn watch_handler(
	State(state): State<Arc<crate::State>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
	Sse::new(try_stream! {
		let (mut sender, mut receiver) = channel(1);
		let mut watcher = RecommendedWatcher::new(
			move |res| {
				block_on(async {
					sender.send(res).await.expect("should send");
				})
			},
			Config::default(),
		).expect("watcher should be created");

		let directory = match &state.args.command {
			Commands::Proxy { host: _host, directory } => directory,
			Commands::Serve { directory } => directory
		};

		watcher.watch(path::Path::new(directory.as_str()), RecursiveMode::Recursive).expect("watcher should watch");

		while let Some(res) = receiver.next().await {
			match res {
				Ok(event) => {
					let hrefs : Vec<_> = event.paths.iter().map(|p| {
						let c = canonicalize(directory.as_str()).expect("path should be valid");

						diff_paths(p, c).map(|p| {
							let p = p.to_str().expect("path should be a string");
							let base = Utf8Path::new(state.args.style_base.as_str());
							let p = base.join(p);

							format!("/{p}")
						})
					}).collect();

					yield Event::default().data(json!(hrefs).to_string());
				},
				Err(e) => eprintln!("watch error: {:?}", e),
			}
		}
	})
	.keep_alive(
		KeepAlive::new()
			.interval(Duration::from_secs(10))
			.text("keep-alive-text"),
	)
}
