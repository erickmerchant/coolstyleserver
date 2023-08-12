use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
	#[arg(short, long, default_value_t = 4000)]
	pub listen: u16,
	#[arg(short, long, default_value = "http://0.0.0.0:3000")]
	pub proxy: String,
	#[arg(short, long, default_value = "./public")]
	pub watch: String,
	#[arg(short, long, default_value = "")]
	pub style_base: String,
	#[arg(short, long, default_value = "coolstyleserver")]
	pub cool_base: String,
}
