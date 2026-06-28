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
  };
}
```
