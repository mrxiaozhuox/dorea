# Installation

Dorea can works on macOS, Linux, and Windows.

## Cargo install 🎯

You can install it from cargo online.

```shell
cargo install dorea
```

The cargo will download Dorea from [crates.io](https://crates.io/crates/dorea).

```shell
# start the Dorea server
dorea-server
# connect Dorea server
dorea-cli -h 127.0.0.1 -p 3450
```



## Rust Include 🛸

If you are using Rust to develop new programs, then you can import Dorea in `cargo.toml`

```toml
[dependencies]
dorea = "0.1"
```

```rust
use dorea::server::{Listener,ServerOption};

#[tokio::main]
pub async fn main() {
  let mut listener = Listener::new("127.0.0.1",3450, ServerOption {
      quiet: false // quiet mode: logs will not print to the console.
  }).await;
  listener.start().await;
}
```

Then you can also create a Dorea server.

