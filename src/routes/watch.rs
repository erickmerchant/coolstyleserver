use crate::args::Commands;
use async_fn_stream::try_fn_stream;
use axum::{
	extract::State,
	response::sse::{Event, KeepAlive, Sse},
};
use camino::Utf8Path;
use futures::{channel::mpsc::channel, executor::block_on, stream::Stream, SinkExt, StreamExt};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use pathdiff::diff_paths;
use serde::Serialize;
use serde_json::json;
use std::{convert::Infallible, fs::canonicalize, path, sync::Arc, time::Duration};

#[derive(Serialize)]
struct Change {
	href: String,
	#[serde(rename = "contentType")]
	content_type: Option<String>,
}

pub async fn watch_handler(
	State(state): State<Arc<crate::State>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
	Sse::new(try_fn_stream(|emitter| async move {
		let (mut sender, mut receiver) = channel(1);
		let mut watcher = RecommendedWatcher::new(
			move |res| {
				block_on(async {
					sender.send(res).await.expect("should send");
				})
			},
			Config::default(),
		)
		.expect("watcher should be created");
		let directory = match &state.args.command {
			Commands::Proxy {
				host: _host,
				directory,
			} => directory,
			Commands::Serve { directory } => directory,
		};

		watcher
			.watch(
				path::Path::new(directory.as_str()),
				RecursiveMode::Recursive,
			)
			.expect("watcher should watch");

		while let Some(res) = receiver.next().await {
			match res {
				Ok(event) => {
					let hrefs: Vec<_> = event
						.paths
						.iter()
						.filter_map(|p| {
							let c = canonicalize(directory.as_str()).expect("path should be valid");
							let href = diff_paths(p, c).map(|p| {
								let p = p.to_str().expect("path should be a string");
								let base = Utf8Path::new(state.args.style_base.as_str());
								let p = base.join(p);

								format!("/{p}")
							});

							match href {
								Some(href) => {
									let content_type =
										mime_guess::from_path(p).first().map(|t| t.to_string());

									Some(Change { href, content_type })
								}
								None => None,
							}
						})
						.collect();

					emitter
						.emit(Event::default().data(json!(hrefs).to_string()))
						.await
				}
				Err(e) => eprintln!("watch error: {e:?}"),
			}
		}

		Ok(())
	}))
	.keep_alive(
		KeepAlive::new()
			.interval(Duration::from_secs(10))
			.text("keep-alive-text"),
	)
}
