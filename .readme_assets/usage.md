```sh
server_upkeep monitor
```

Config at `~/.config/server_upkeep.nix`:
```nix
{
  telegram = {
    bot_token = "your_bot_token";
    alerts_chat = "your_chat_id";
  };
  monitor = {
    max_size = "5GB";  # human-readable: 500MB, 1.5TB, etc.
  };
}
```
