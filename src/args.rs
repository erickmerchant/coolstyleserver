use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
	#[arg(short, long, default_value_t = 4000)]
	pub listen: u16,
	#[arg(short, long)]
	pub proxy: Option<String>,
	#[arg(short, long, default_value = "./public")]
	pub watch: String,
	#[arg(short, long, default_value = "coolstyleserver")]
	pub base: String,
}
