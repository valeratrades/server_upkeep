use v_utils::{
	macros::{MyConfigPrimitives, Settings},
	utils::InfoSize,
};

#[derive(Clone, Debug, Default, MyConfigPrimitives, Settings)]
pub struct AppConfig {
	pub telegram: TelegramConfig,
	pub monitor: MonitorConfig,
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
	InfoSize::from_parts(10, v_utils::utils::InfoSizeUnit::Gigabyte)
}
