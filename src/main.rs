mod config;

use std::path::Path;

use clap::{Parser, Subcommand};
use color_eyre::eyre::{Result, eyre};
use config::{AppConfig, ByteSize, SettingsFlags};
use reqwest::Client;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
	#[command(flatten)]
	settings: SettingsFlags,
}

#[derive(Subcommand)]
enum Commands {
	/// Monitor ~/.local/state directory size and alert if over threshold
	Monitor,
}

#[tokio::main]
async fn main() -> Result<()> {
	v_utils::clientside!();
	let cli = Cli::parse();
	let config = AppConfig::try_build(cli.settings)?;

	match cli.command {
		Commands::Monitor => monitor(config).await?,
	}

	Ok(())
}

async fn monitor(config: AppConfig) -> Result<()> {
	let state_dir = dirs::state_dir().ok_or_else(|| eyre!("Could not determine state directory"))?;
	let size = ByteSize(get_dir_size(&state_dir)?);
	let max_size = config.monitor.max_size;

	println!("~/.local/state size: {size} (threshold: {max_size})");

	if size > max_size {
		let message = format!("⚠️ Server Alert: ~/.local/state is {size}, exceeds threshold of {max_size}");
		send_telegram_alert(&config.telegram, &message).await?;
		println!("Alert sent to Telegram");
	} else {
		println!("Size is within limits, no alert needed");
	}

	Ok(())
}

fn get_dir_size(path: &Path) -> Result<u64> {
	let mut total_size = 0u64;

	if path.is_dir() {
		for entry in std::fs::read_dir(path)? {
			let entry = entry?;
			let path = entry.path();
			if path.is_dir() {
				total_size += get_dir_size(&path)?;
			} else {
				total_size += entry.metadata()?.len();
			}
		}
	}

	Ok(total_size)
}

async fn send_telegram_alert(config: &config::TelegramConfig, message: &str) -> Result<()> {
	let client = Client::new();
	let url = format!("https://api.telegram.org/bot{}/sendMessage", config.bot_token);

	let params = [("chat_id", config.alerts_chat.as_str()), ("text", message)];

	let response = client.post(&url).form(&params).send().await?;

	if !response.status().is_success() {
		let error_text = response.text().await?;
		return Err(eyre!("Failed to send Telegram message: {error_text}"));
	}

	Ok(())
}
