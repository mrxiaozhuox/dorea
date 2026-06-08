# Change Log

## v0.4.0

- 增加 `Docs` 命令，可在系统内部查询文档
- 增加 `Db` 命令，开放更多库的直接管控命令
- 增加 `Service` 命令，可在运行期间控制 `Web-Service`
- 对 `Web-Service` 普通账号进行权限控制，限制其可使用的命令
- 重写 `Web-Service` 账号系统，账号信息存储于数据库中，支持自定义权限
- 增加 `WebSocket` 通道支持，目前可访问 `/_ws` 以连接 `WS` 服务器
- **重写通讯协议**：从文本协议改为二进制长度前缀协议（MAGIC + VERSION + STATE + LEN + PAYLOAD），解决 TCP 粘包/半包、无帧大小上限、Base64 双重编码等问题
- **并发模型重构**：从单 `Mutex<DataBaseManager>` 改为 `DashMap<String, Arc<RwLock<DataBase>>>`，实现数据库级别的读写锁并发访问
- **命令参数解析重写**：支持双引号包裹含空格的值及 `\"` 转义（如 `set foo "hello world"`）
- **索引计数器原子化**：`TOTAL_INDEX_NUMBER` / `MAX_INDEX_NUMBER` 从 `Mutex<TotalInfo>` 改为 `AtomicU32`
- **异步 I/O**：数据库热路径文件操作从 `std::fs` 迁移至 `tokio::fs`
- 修复 `check_db()` 判断错误导致数据丢失的问题（`is_dir()` → `is_file()`）
- 修复 `repwd` 命令查询了字面量 `"username"` 而非变量的问题
- 修复 `delete` 命令在写入失败时索引计数器仍递减的问题
- 修复 `@key` 前缀判断使用字符串切片越界的风险（改为 `starts_with('@')`）
- 删除遗留的调试 `println!` 输出

## v0.3.1

> 0.3.X 的首次修正版

- 对 `Get` 操作进行优化，使效率更高
- ~~插件系统增强；接口设计优化~~
- 优化索引数限制相关；单个数据库限制
- 优化传输协议设计
- 增加动态索引卸载功能

## V0.3.0

> 0.3 首个正式版，添加部分 Features

- 增加 `Binary` 类型
- 默认开启 `Web-Service`
- 增加 Docker 支持

## V0.3.0-alpha 

> 首个重构版本，基本功能完成