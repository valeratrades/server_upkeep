use std::str::FromStr;

use v_utils::macros::{MyConfigPrimitives, Settings};

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
	pub max_size: ByteSize,
}

impl Default for MonitorConfig {
	fn default() -> Self {
		Self { max_size: __default_max_size() }
	}
}

fn __default_max_size() -> ByteSize {
	ByteSize(1024 * 1024 * 1024) // 1GB
}

/// Human-readable byte size that can be parsed from strings like "20GB", "500MB", "1.5TB"
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ByteSize(pub u64);

impl FromStr for ByteSize {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let s = s.trim();
		if s.is_empty() {
			return Err("empty string".to_string());
		}

		// Find where the number ends and unit begins
		let (num_str, unit) = match s.find(|c: char| c.is_alphabetic()) {
			Some(idx) => (&s[..idx], s[idx..].trim()),
			None => (s, ""),
		};

		let num: f64 = num_str.trim().parse().map_err(|e| format!("invalid number: {e}"))?;

		let multiplier: u64 = match unit.to_uppercase().as_str() {
			"" | "B" => 1,
			"K" | "KB" | "KIB" => 1024,
			"M" | "MB" | "MIB" => 1024 * 1024,
			"G" | "GB" | "GIB" => 1024 * 1024 * 1024,
			"T" | "TB" | "TIB" => 1024 * 1024 * 1024 * 1024,
			other => return Err(format!("unknown unit: {other}")),
		};

		Ok(ByteSize((num * multiplier as f64) as u64))
	}
}

impl<'de> serde::Deserialize<'de> for ByteSize {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>, {
		struct ByteSizeVisitor;

		impl serde::de::Visitor<'_> for ByteSizeVisitor {
			type Value = ByteSize;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a byte size like \"20GB\", \"500MB\", or a number")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error, {
				ByteSize::from_str(v).map_err(E::custom)
			}

			fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
			where
				E: serde::de::Error, {
				Ok(ByteSize(v as u64))
			}

			fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
			where
				E: serde::de::Error, {
				Ok(ByteSize(v))
			}

			fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
			where
				E: serde::de::Error, {
				Ok(ByteSize(v as u64))
			}
		}

		deserializer.deserialize_any(ByteSizeVisitor)
	}
}

impl std::fmt::Display for ByteSize {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let bytes = self.0;
		if bytes >= 1024 * 1024 * 1024 * 1024 {
			write!(f, "{:.2}TB", bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0))
		} else if bytes >= 1024 * 1024 * 1024 {
			write!(f, "{:.2}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
		} else if bytes >= 1024 * 1024 {
			write!(f, "{:.2}MB", bytes as f64 / (1024.0 * 1024.0))
		} else if bytes >= 1024 {
			write!(f, "{:.2}KB", bytes as f64 / 1024.0)
		} else {
			write!(f, "{}B", bytes)
		}
	}
}
