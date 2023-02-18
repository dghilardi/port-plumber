# Port Plumber

Utility bind ports with initialization commands

## Configuration

```toml
# Bind 127.0.0.1:12345 to 127.0.0.1:80
[plumbing."127.0.0.1:12345"]
target = "127.0.0.1:80"


# Bind 127.0.0.1:23456 to 127.0.0.1:2048
# At first connection the setup command will be spawn and 500ms will be awaited before redirecting the connection to the target
[plumbing."127.0.0.1:23456"]
target = "127.0.0.1:2048"
resource.setup = { command = "http-server", args = ["-h", "127.0.0.1", "-p", "2048", "-v"] }
resource.warmup_millis = 500
```

## Autostart

### Systemd

On linux systems systemd can be used to automatically start this command on startup.

To achieve this, the following file needs to be created:

`$HOME/.config/systemd/user/port-plumber.service`

```toml
[Unit]
Description=Launch port-plumber

[Service]
Type=simple
ExecStart=%h/.cargo/bin/port-plumber
Environment=RUST_LOG=info
[Install]
WantedBy=default.target
```

And the following commands needs to be executed:

 * `systemctl --user daemon-reload` reload systemd user daemons
 * `systemctl --user enable port-plumber` enable `port-plumber` daemon