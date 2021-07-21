# 通讯协议

目前 `Dorea` 采用自定义的 `TCP` 通讯协议，它的结构为：

```text
$: {DATA_SIZE} | %: {SYS_STATE} | #: B64'{DATA_BODY}';
```

- **DATA_SIZE** 数据包长度
- **SYS_STATE** 数据状态（ERR、OK、NOAUTH）
- **DATA_BODY** 数据内容（包含在 `B64` 中则说明内容经过 Base64 处理）

`SYS_STATE` 数据状态一般在服务器向客户端回复时才会携带，客户端发送信息会省略这个参数。

## 协议解析器

我们将 `Dorea-Core` 中的 `协议解析器` 作为公开模块发布了，使用 Rust 可以直接调用现有方法进行解析。

```rust
use dorea::network;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let mut socket = TcpStream::connect(addr).await?;
    
    let frame = network::Frame();
    let v: Vec<u8> = frame.parse_frame(&mut socket).await.unwarp();
}
```

解析函数需要将 `TCP` 连接的可变引用传递进去，因为它有一套自己的读取方案（数据读少了，则自动补充）

## 其他协议兼容

目前我们打算实现对于 `Redis` 协议的兼容。