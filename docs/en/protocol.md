# Communication Protocol

Currently `Dorea` uses a TCP-based binary length-prefix protocol. Each frame format is as follows:

```text
+----------+----------+----------+----------+
|  MAGIC   | VERSION  |  STATE   |   LEN    |
|  2 bytes | 1 byte   | 1 byte   | 4 bytes  |
+----------+----------+----------+----------+
|              PAYLOAD (LEN bytes)          |
+-------------------------------------------+
```

| Field | Size | Description |
|-------|------|-------------|
| MAGIC | 2 bytes | Fixed `0xD0 0x9A`, used for stream alignment and validation |
| VERSION | 1 byte | Protocol version number, currently `0x01` |
| STATE | 1 byte | Status code: `0x00`=IGNORE, `0x01`=OK, `0x02`=ERR, `0x03`=EMPTY, `0x04`=NOAUTH |
| LEN | 4 bytes | Payload length, big-endian (u32), maximum 16MB |
| PAYLOAD | LEN bytes | Raw binary data (command text is UTF-8, or response byte stream) |

When the client sends a request, STATE is set to `IGNORE (0x00)`. When the server responds, it carries the actual status code.

Command arguments support double-quoted values containing spaces:

```text
set foo "hello world"        # key=foo, value="hello world"
set bar [1,2,3]              # key=bar, value=[1,2,3]
set baz "escaped \"quote\""  # key=baz, value=escaped "quote"
```

## Protocol Parser

We have released the `Protocol Parser` from `Dorea-Core` as a public module. Using Rust, you can directly call existing methods for parsing.

```rust
use dorea::network;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let mut socket = TcpStream::connect(addr).await?;

    let mut frame = network::Frame::new();
    let v: Vec<u8> = frame.parse_frame(&mut socket).await.unwrap();
    let state = frame.latest_state;  // Get response status code
}
```

The parsing function uses `read_exact` to ensure complete reading of frame headers and Payload, naturally handling TCP sticky/partial packet issues.

## Other Protocol Compatibility

Currently we are implementing `HTTP` based `WEB` API interfaces.
