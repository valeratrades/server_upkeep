use serde::{Deserialize, Serialize};
use v_utils::{
	InfoSize,
	macros::{MyConfigPrimitives, Settings},
};

#[derive(Clone, Debug, Default, MyConfigPrimitives, Settings)]
pub struct AppConfig {
	pub alert: Alert,
	pub monitor: MonitorConfig,
}

/// Alert sink. A bare string is a shell command the alert text is piped into via stdin
/// (e.g. `"v_notify -a tg -l error -"`); a table selects built-in Telegram delivery.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Alert {
	Command(String),
	Telegram(TelegramConfig),
}

impl Default for Alert {
	fn default() -> Self {
		Self::Telegram(TelegramConfig::default())
	}
}

#[derive(Clone, Debug, Default, MyConfigPrimitives)]
pub struct TelegramConfig {
	pub bot_token: String,
	pub alerts_chat: String,
}

#[derive(Clone, Debug, MyConfigPrimitives)]
pub struct MonitorConfig {
	/// Maximum allowed size for ~/.local/state (e.g., "20GB", "500MB")
	#[serde(default = "__default_max_size")]
	pub max_size: InfoSize,
}

impl Default for MonitorConfig {
	fn default() -> Self {
		Self { max_size: __default_max_size() }
	}
}

fn __default_max_size() -> InfoSize {
	InfoSize::from_parts(10, v_utils::InfoSizeUnit::Gigabyte)
}
