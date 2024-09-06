use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
	#[command(subcommand)]
	pub command: Commands,

	#[arg(short, long, default_value_t = 4000)]
	pub port: u16,
	#[arg(short, long, default_value = "")]
	pub style_base: String,
	#[arg(short, long, default_value = "coolstyleserver")]
	pub cool_base: String,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
	Proxy {
		#[arg(default_value = ".")]
		directory: String,
		#[arg(default_value = "http://0.0.0.0:3000")]
		host: String,
	},
	Serve {
		#[arg(default_value = ".")]
		directory: String,
	},
}
