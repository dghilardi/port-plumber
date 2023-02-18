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