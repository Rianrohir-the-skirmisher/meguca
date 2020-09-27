use clap::Clap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

lazy_static::lazy_static! {
	/// Configurations for this specific application server
	pub static ref SERVER: Server = Server::parse();
}

/// Configurations for this specific application server
#[derive(Clap)]
pub struct Server {
	/// Database address to connect to
	#[clap(short, long)]
	pub database: String,

	/// Address for the server to listen on
	#[clap(short, long, default_value = "127.0.0.1:8000")]
	pub address: String,

	/// Indicates this server is behind a reverse proxy and can honour
	/// X-Forwarded-For and similar headers
	#[clap(short, long)]
	pub reverse_proxied: bool,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SpamScores {
	/// Score per unicode character for any post body modification
	pub character: usize,

	/// Score for inserting an image into the post
	pub image: usize,

	/// Score for creating a post
	pub post_creation: usize,
}

/// Global server configurations
#[derive(Default, Serialize, Deserialize)]
pub struct Config {
	/// Enable captchas and antispam
	pub captcha: bool,

	/// Configured labeled links to resources
	pub links: HashMap<String, String>,

	/// Amounts to increase spam score by for a user action
	pub spam_scores: SpamScores,
}

protocol::gen_global!(
	// Server-wide configurations
	Config {
		pub fn read();
		pub fn write();
	}
);
