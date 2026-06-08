# 通讯协议

目前 `Dorea` 采用基于 TCP 的二进制长度前缀协议，每帧格式如下：

```text
+----------+----------+----------+----------+
|  MAGIC   | VERSION  |  STATE   |   LEN    |
|  2 bytes | 1 byte   | 1 byte   | 4 bytes  |
+----------+----------+----------+----------+
|              PAYLOAD (LEN bytes)          |
+-------------------------------------------+
```

| 字段 | 大小 | 说明 |
|------|------|------|
| MAGIC | 2 字节 | 固定 `0xD0 0x9A`，用于流对齐和校验 |
| VERSION | 1 字节 | 协议版本号，当前为 `0x01` |
| STATE | 1 字节 | 状态码：`0x00`=IGNORE, `0x01`=OK, `0x02`=ERR, `0x03`=EMPTY, `0x04`=NOAUTH |
| LEN | 4 字节 | Payload 长度，大端序 (u32)，最大 16MB |
| PAYLOAD | LEN 字节 | 原始二进制数据（命令文本为 UTF-8，或响应字节流） |

客户端发送请求时 STATE 设为 `IGNORE (0x00)`，服务端响应时携带实际状态码。

命令参数支持双引号包裹含空格的值：

```text
set foo "hello world"        # key=foo, value="hello world"
set bar [1,2,3]              # key=bar, value=[1,2,3]
set baz "escaped \"quote\""  # key=baz, value=escaped "quote"
```

## 协议解析器

我们将 `Dorea-Core` 中的 `协议解析器` 作为公开模块发布了，使用 Rust 可以直接调用现有方法进行解析。

```rust
use dorea::network;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let mut socket = TcpStream::connect(addr).await?;

    let mut frame = network::Frame::new();
    let v: Vec<u8> = frame.parse_frame(&mut socket).await.unwrap();
    let state = frame.latest_state;  // 获取响应状态码
}
```

解析函数使用 `read_exact` 保证完整读取帧头和 Payload，天然处理 TCP 粘包/半包问题。

## 其他协议兼容

目前我们正在实现 `HTTP` 的 `WEB` API 接口。