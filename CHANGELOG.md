# Change Log

## v0.5.0

### 🖥️ TUI 模式

- **新增 TUI (Terminal User Interface) 模式**：通过 `dorea-cli tui` 命令或 REPL 中输入 `tui` 启动
- **Tab 切换界面**：Data / Monitor / Log / Help 四个 Tab，支持 `Tab` / `Shift+Tab` 切换
- **Data Tab**：两列布局（键列表 | 值视图），支持 Vim 风格导航 (`j`/`k`/`h`/`l`) 和方向键
- **Pretty 模式**：组件化值展示，支持 Dict/List/Tuple 展开折叠，语法高亮（String 绿色、Number 黄色、Boolean 紫色）
- **智能展开**：第一层子项 ≤30 时自动展开，超过则折叠
- **Monitor Tab**：显示服务器版本、运行时间、连接数、索引数等信息，切换 Tab 时自动刷新
- **Log Tab**：记录操作日志
- **命令输入**：按 `:` 进入命令模式，执行结果以居中弹窗显示，支持 ESC 关闭、`:` 覆盖
- **快捷键**：`r` 刷新、`F2` 切换 Pretty/Raw、`Enter` 展开/折叠、`q` 退出

### 🚀 性能优化

- **Pipeline 批量写入**：新增 `DoreaClient::pipeline()` 方法，支持批量发送命令，吞吐量提升 **2.28x**
- **文件句柄缓存**：缓存数据文件句柄，减少 `open()` 系统调用，单客户端写入提升 **27%**
- **批量响应协议**：Pipeline 使用单次 `write_all` 发送所有响应，减少系统调用开销

### ✨ 新功能

- **Pipeline 协议**：客户端告知服务端命令数量，服务端精准批量处理
- **NetPacketState::PIPELINE**：新增协议状态码，标识 Pipeline 批量命令
- **并发批量测试示例**：新增 `bulk_concurrent.rs` 示例，展示多客户端并发写入
- **Search 命令增强**：支持 Value 内容搜索和简化匹配语法
  - `search <pattern>` — 默认搜索 key + value（子串匹配）
  - `search key <pattern>` — 仅搜索 key
  - `search value <pattern>` — 仅搜索 value 内容
  - `^` 前缀匹配 / `$` 后缀匹配 / `^...$` 精确匹配
  - 无通配符时自动子串匹配，无需 `*word*`
  - 保留 `*` / `?` 通配符支持
- **移除旧 search 语法**：旧 `search <pattern> [limit]` 不再支持，需显式指定 `key`/`value`

### 🐛 修复

- 修复服务启动时间别名错误（`stt` → `sst`）
- 修复多个示例命令语法错误

### 📚 文档与示例

- 新增完整的英文文档
- 新增更多实用示例（counter、queue 等）
- 优化 `bulk.rs` 示例，添加性能统计

### 🌐 多语言支持

- 添加多语言测试支持（中文、日文、韩文、俄文、阿拉伯文、Emoji）
- 修复中文字符串解析问题
- CLI 支持 Doson 格式（tuple、dict、list）的美化输出

### 🧪 测试

- 添加 `parse_command_args` 全面的单元测试

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