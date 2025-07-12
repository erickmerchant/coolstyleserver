use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
	#[command(subcommand)]
	pub command: Commands,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
	Proxy {
		#[arg(default_value = ".")]
		directory: String,
		#[arg(default_value = "http://0.0.0.0:3000")]
		host: String,

		#[arg(short, long, default_value_t = 4000)]
		port: u16,
		#[arg(short, long, default_value = "")]
		style_base: String,
		#[arg(short, long, default_value = "coolstyleserver")]
		cool_base: String,
	},
	Serve {
		#[arg(default_value = ".")]
		directory: String,

		#[arg(short, long, default_value_t = 4000)]
		port: u16,
		#[arg(short, long, default_value = "")]
		style_base: String,
		#[arg(short, long, default_value = "coolstyleserver")]
		cool_base: String,
	},
}
