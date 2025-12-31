mod config;

use std::{
	fs,
	path::Path,
	time::{Duration, SystemTime},
};

use clap::{Parser, Subcommand};
use color_eyre::eyre::{Result, eyre};
use config::{AppConfig, SettingsFlags};
use reqwest::Client;
use tracing::{error, info};
use v_utils::{utils::InfoSize, xdg_state_file};

#[derive(Parser)]
#[command(author, version = concat!(env!("CARGO_PKG_VERSION"), " (", env!("GIT_HASH"), ")"), about, long_about = None)]
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
	/// Clean files in /tmp that are older than 1 hour
	//TODO!!!!: at least extend to require provision of [Timeframe](v_utils::trades::Timeframe)
	Tempfiles {
		/// Run continuously, cleaning every hour
		#[arg(short, long)]
		daemon: bool,
	},
}

#[tokio::main]
async fn main() -> Result<()> {
	v_utils::clientside!();
	let cli = Cli::parse();
	let config = AppConfig::try_build(cli.settings)?;

	match cli.command {
		Commands::Monitor => monitor(config).await?,
		Commands::Tempfiles { daemon } => tempfiles(daemon).await?,
	}

	Ok(())
}

const DISK_USAGE_THRESHOLDS: &[u8] = &[50, 60, 70, 80, 90, 95];
const DISK_USAGE_RESET_THRESHOLD: u8 = 45;
const MONITOR_INTERVAL: Duration = Duration::from_secs(60 * 60); // 1 hour

async fn monitor(config: AppConfig) -> Result<()> {
	let state_dir = dirs::state_dir().ok_or_else(|| eyre!("Could not determine state directory"))?;

	loop {
		// Check ~/.local/state directory size
		match get_dir_size(&state_dir) {
			Ok(size_bytes) => {
				let size = InfoSize::from_parts(size_bytes, v_utils::utils::InfoSizeUnit::Byte);
				let max_size = config.monitor.max_size;
				info!("~/.local/state size: {size} (threshold: {max_size})");

				if size > max_size {
					let message = format!("⚠️ Server Alert: ~/.local/state is {size}, exceeds threshold of {max_size}");
					if let Err(e) = send_telegram_alert(&config.telegram, &message).await {
						error!("Failed to send state dir alert: {e}");
					} else {
						info!("State dir alert sent to Telegram");
					}
				}
			}
			Err(e) => error!("Failed to get state directory size: {e}"),
		}

		// Check disk usage percentage of /
		if let Err(e) = check_disk_usage(&config).await {
			error!("Failed to check disk usage: {e}");
		}

		tokio::time::sleep(MONITOR_INTERVAL).await;
	}
}

async fn check_disk_usage(config: &AppConfig) -> Result<()> {
	let statvfs = nix::sys::statvfs::statvfs("/")?;
	let total_blocks = statvfs.blocks();
	let available_blocks = statvfs.blocks_available();
	let used_blocks = total_blocks - available_blocks;
	let usage_pct = (used_blocks as f64 / total_blocks as f64 * 100.0) as u8;

	info!("/ disk usage: {usage_pct}%");

	let state_file = xdg_state_file!("last_pct_used");

	// If usage dropped below reset threshold, delete state file
	if usage_pct < DISK_USAGE_RESET_THRESHOLD {
		if state_file.exists() {
			fs::remove_file(&state_file)?;
			info!("Disk usage below {DISK_USAGE_RESET_THRESHOLD}%, cleared alert state");
		}
		return Ok(());
	}

	// Find the highest threshold that current usage exceeds (minimum is 50%)
	let current_threshold = DISK_USAGE_THRESHOLDS.iter().rev().find(|&&t| usage_pct >= t).copied();

	let Some(threshold) = current_threshold else {
		// usage_pct is between DISK_USAGE_RESET_THRESHOLD and 50%, no alert needed
		return Ok(());
	};

	// Check last alerted threshold
	let last_alerted: Option<u8> = if state_file.exists() { fs::read_to_string(&state_file)?.trim().parse().ok() } else { None };

	// Only alert if we crossed a new threshold
	if last_alerted.is_none() || threshold > last_alerted.unwrap() {
		let message = format!("⚠️ Server Alert: / disk usage at {usage_pct}% (crossed {threshold}% threshold)");
		match send_telegram_alert(&config.telegram, &message).await {
			Ok(()) => {
				fs::write(&state_file, threshold.to_string())?;
				info!("Disk usage alert sent for {threshold}% threshold");
			}
			Err(e) => error!("Failed to send disk usage alert: {e}"),
		}
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

async fn tempfiles(daemon: bool) -> Result<()> {
	loop {
		let tmp_dir = Path::new("/tmp");
		let max_age = Duration::from_secs(60 * 60); // 1 hour
		let now = SystemTime::now();

		let mut deleted_count = 0u64;
		let mut deleted_bytes = 0u64;
		let mut error_count = 0u64;

		clean_old_files(tmp_dir, now, max_age, &mut deleted_count, &mut deleted_bytes, &mut error_count);

		println!(
			"Cleaned /tmp: deleted {} files ({:.2} MB), {} errors",
			deleted_count,
			deleted_bytes as f64 / (1024.0 * 1024.0),
			error_count
		);

		if !daemon {
			break;
		}

		tokio::time::sleep(Duration::from_secs(60 * 60)).await; // Sleep for 1 hour
	}

	Ok(())
}

fn clean_old_files(dir: &Path, now: SystemTime, max_age: Duration, deleted_count: &mut u64, deleted_bytes: &mut u64, error_count: &mut u64) {
	let entries = match std::fs::read_dir(dir) {
		Ok(e) => e,
		Err(_) => return,
	};

	for entry in entries.flatten() {
		let path = entry.path();

		if path.is_dir() {
			clean_old_files(&path, now, max_age, deleted_count, deleted_bytes, error_count);
			// Try to remove the directory if it's empty and old enough
			if let Ok(meta) = std::fs::metadata(&path) {
				if let Ok(modified) = meta.modified() {
					if let Ok(age) = now.duration_since(modified) {
						if age > max_age && std::fs::remove_dir(&path).is_ok() {
							*deleted_count += 1;
						}
					}
				}
			}
		} else {
			let meta = match entry.metadata() {
				Ok(m) => m,
				Err(_) => {
					*error_count += 1;
					continue;
				}
			};

			let modified = match meta.modified() {
				Ok(m) => m,
				Err(_) => {
					*error_count += 1;
					continue;
				}
			};

			let age = match now.duration_since(modified) {
				Ok(a) => a,
				Err(_) => continue, // File is from the future, skip
			};

			if age > max_age {
				let size = meta.len();
				if std::fs::remove_file(&path).is_ok() {
					*deleted_count += 1;
					*deleted_bytes += size;
				} else {
					*error_count += 1;
				}
			}
		}
	}
}
