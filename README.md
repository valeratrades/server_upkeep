# server_upkeep
![Minimum Supported Rust Version](https://img.shields.io/badge/nightly-1.93+-ab6000.svg)
[<img alt="crates.io" src="https://img.shields.io/crates/v/server_upkeep.svg?color=fc8d62&logo=rust" height="20" style=flat-square>](https://crates.io/crates/server_upkeep)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs&style=flat-square" height="20">](https://docs.rs/server_upkeep)
![Lines Of Code](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/valeratrades/b48e6f02c61942200e7d1e3eeabf9bcb/raw/server_upkeep-loc.json)
![Build time](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/valeratrades/b48e6f02c61942200e7d1e3eeabf9bcb/raw/server_upkeep-build.json)
![Test time](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/valeratrades/b48e6f02c61942200e7d1e3eeabf9bcb/raw/server_upkeep-test.json)
<br>
[![errors](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/valeratrades/b48e6f02c61942200e7d1e3eeabf9bcb/raw/server_upkeep-errors.json)](https://github.com/valeratrades/server_upkeep/actions/workflows/errors.yml?query=branch%3Amain)
[![warnings](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/valeratrades/b48e6f02c61942200e7d1e3eeabf9bcb/raw/server_upkeep-warnings.json)](https://github.com/valeratrades/server_upkeep/actions/workflows/warnings.yml?query=branch%3Amain)

Monitor server directories and alert (via Telegram or an arbitrary command) when size thresholds are exceeded.
<!-- markdownlint-disable -->
<details>
<summary>
<h2>Installation</h2>
</summary>

```sh
cargo install --git https://github.com/valeratrades/server_upkeep
```


</details>
<!-- markdownlint-restore -->

## Usage
```sh
server_upkeep monitor
```

Config at `~/.config/server_upkeep.nix`. `alert` is either a shell command the
alert text is piped into (stdin), or a Telegram table:
```nix
{
  alert = "v_notify -a tg -l error -"; # or a table: { bot_token = "..."; alerts_chat = "..."; }
  monitor = {
    max_size = "5GB";  # human-readable: 500MB, 1.5TB, etc.
    disk_alert_pct = 80;  # lowest `/` usage % that alerts
  };
}
```



<br>

<sup>Links for devs:</sup>
- <sup><a href="./docs/ARCHITECTURE.md">architecture</a></sup>
- <sup><a href="https://github.com/EV-Main/.github/blob/main/code/STYLE.md">style</a></sup>
- <sup><a href="https://github.com/EV-Main/.github/tree/main/code/conventions">conventions</a></sup>
- <sup><a href="https://github.com/EV-Main/.github/blob/main/code/conventions/git">contributing</a></sup>

#### License

<sup>
	Licensed under <a href="LICENSE">Blue Oak 1.0.0</a>
</sup>

<br>

<sub>
	Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be licensed as above, without any additional terms or conditions.
</sub>

