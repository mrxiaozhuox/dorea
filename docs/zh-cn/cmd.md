# 系统命令

> 本文档介绍了系统自带的命令

```rust
// 内部命令列表定义
#[derive(Debug)]
pub enum CommandList {
    GET,
    SET,
    DELETE,
    CLEAN,
    SELECT,
    SEARCH,
    INFO,
    EDIT,
    PING,
    EVAL,
    AUTH,
    VALUE,
    DB,
    DOCS,
    SERVICE,
}
```

## `GET` | 读取

通过 `Get` 命令获取数据具体内容：

```
get <key>
```

输入 `Key` 信息直接获取对应的数据：

```
~> get foo
[OK]: "bar"
```

## `SET` | 设置

通过 `Set` 命令插入某条数据到数据库中：

```
set <key> <value> [expire]
```

- value: 结构请严格按照 [DOSON](/zh-cn/data-value) 规范编写。
- expire: 过期时间；留空或 `0` 代表无过期时间。[可空]

如果值中包含空格，需要使用双引号包裹，引号内支持 `\"` 转义：

```
~> set foo "bar"
[OK]: Successful

~> set msg "hello world"
[OK]: Successful

~> set note "say \"hi\""
[OK]: Successful
```

## `DELETE` | 删除

通过 `Delete` 删除一条数据：

```
delete <key>
```

删除后数据将无法访问（除非再次被设置`set`）

## `CLEAN` | 清空

通过 `Clean` 清空一个数据库

```
clean [group]
```

不传入 `group` 信息默认清空当前 `current` 数据库的数据。

## `SELECT` | 切换库

每一个 `Dorea` 服务中都可以创建多个数据库，并可以使用 `Select` 切换它：

```
select <group> [options]
```

- options: 一些其他可选参数【此功能为画饼、开发中】
  - preload: 提前加载（即本次不会切换到目标数据库，而是先进行索引预加载）

如果一个数据库是第一次加载（即本次服务启动内没被加载过），则需要加载索引，时间复杂度 *O(n)*

一个从未被创建的数据库调用本命令也会自动被创建哦！

## `SEARCH` | 数据查找

用于数据的查找（Key 和 Value 内容搜索）

```
search <pattern>              # 搜索 key + value（默认）
search key <pattern> [limit]   # 仅搜索 key
search value <pattern> [limit] # 仅搜索 value 内容
```

### 匹配规则

| 写法 | 含义 | 示例 |
|------|------|------|
| `word` | 子串匹配 | `admin` 匹配 `user:admin`、`admin_config` |
| `^word` | 前缀匹配 | `^user` 匹配 `user:xxx` |
| `word$` | 后缀匹配 | `.log$` 匹配 `error.log` |
| `*` `?` | 通配符 | `user*:?dmin` |

### 示例

```
~> search admin
[OK]: ["user:admin", "admin_config"]

~> search key ^user
[OK]: ["user:admin", "user:mrxzx"]

~> search value hello
[OK]: ["key1", "test_key"]
```

## `INFO` | 信息获取

本命令用于获取 **数据库** | **数据项** 的一些基本信息。

```
info <option>
```

### `Current` | 当前数据库

```
info current
```

获取当前所选的数据库（使用 `SELECT` 切换）

```
~> info current
[OK]: default
```

### `Version` | 版本号

```
info version
```

获取当前 Dorea 版本号：

```
~> info version
[OK]: V0.4.0
```

### `Keys` | 键列表

```
info keys
```

获取当前数据库下的 `key` 列表。

```
~> info keys
[OK]: ["foo", "hello", "example"]
```

时间复杂度为 *O(n)* （时间复杂度优化将提上议程）

### `Max-Connect-Number` | 最大连接数

```
info max-connect-number
info mcn
```

获取服务器最大连接数配置。

### `Total-Index-Number` | 总索引数

```
info total-index-number
info tin
```

获取当前已加载的索引数量统计。

### `Server-Startup-Time` | 服务器启动时间

```
info server-startup-time
info stt
```

获取服务器启动时间。

### `Connect-Id` | 连接ID

```
info connect-id
info cid
```

获取当前连接的唯一ID。

### `@Key` | 数据项详情

获取指定 key 的详细信息（元数据）：

```
info @<key> [sub-info]
```

可选的 `sub-info` 参数：

- `expire` - 过期时间
- `timestamp` - 时间戳
- `weight` - 权重

```
~> info @foo
[OK]: MetaNode { ... }

~> info @foo expire
[OK]: 0

~> info @foo timestamp
[OK]: (1626470590, 0)
```

## `EDIT` | 编辑复合数据

对已有的复合数据（List、Dict）进行操作：

```
edit @<key> <operation> [args...]
```

### `incr` | 数值自增

对数值或复合数据中的数值进行自增：

```
edit @<key> incr [amount]
```

- amount: 自增量，默认为 1

```
~> set counter 10
~> edit @counter incr
[OK]: Successful
~> get counter
[OK]: 11

~> edit @counter incr 5
[OK]: Successful
~> get counter
[OK]: 16
```

### `expire` | 设置过期时间

修改数据的过期时间：

```
edit @<key> expire <time>
```

- `+N` - 增加N秒
- `-N` - 减少N秒
- `=N` 或 `N` - 设置为N秒

```
~> edit @foo expire +60    # 增加60秒
~> edit @foo expire -30    # 减少30秒
~> edit @foo expire =100   # 设置为100秒
```

### `insert` | 插入数据

向 Dict 或 List 插入数据：

```
edit @<key> insert <value> [index/key]
```

- Dict: `edit @dict insert "value" "key"`
- List: `edit @list insert "value" [index]`

```
~> set mylist [1, 2, 3]
~> edit @mylist insert 4
~> get mylist
[OK]: [1, 2, 3, 4]

~> set mydict {"a": 1}
~> edit @mydict insert 2 "b"
~> get mydict
[OK]: {"a": 1, "b": 2}
```

### `remove` | 删除数据

从 Dict 或 List 中删除数据：

```
edit @<key> remove <index/key>
```

```
~> edit @mylist remove 0    # 删除List第一个元素
~> edit @mydict remove "a"  # 删除Dict中key为"a"的项
```

### `push` | 追加元素

向 List 末尾追加元素：

```
edit @<key> push <value>
```

```
~> set mylist [1, 2]
~> edit @mylist push 3
~> get mylist
[OK]: [1, 2, 3]
```

### `pop` | 弹出元素

弹出 List 末尾元素：

```
edit @<key> pop
```

```
~> set mylist [1, 2, 3]
~> edit @mylist pop
~> get mylist
[OK]: [1, 2]
```

### `sort` | 排序

对 List 进行排序：

```
edit @<key> sort [order]
```

- order: `asc`（升序，默认）或 `desc`（降序）

```
~> set mylist [3, 1, 2]
~> edit @mylist sort
~> get mylist
[OK]: [1, 2, 3]

~> edit @mylist sort desc
~> get mylist
[OK]: [3, 2, 1]
```

### `reverse` | 反转

反转 List 或复合数据：

```
edit @<key> reverse
```

```
~> set mylist [1, 2, 3]
~> edit @mylist reverse
~> get mylist
[OK]: [3, 2, 1]
```

## `DB` | 数据库管理

数据库管理命令：

### `db unload` | 卸载数据库

从内存中卸载指定数据库：

```
db unload <db-name>
```

如果数据库正在使用中则无法卸载。

### `db preload` | 预载数据库

预加载指定数据库（异步加载）：

```
db preload <db-name>
```

预加载期间不影响其他命令的执行。

### `db list` | 列出数据库

列出所有已加载数据库：

```
db list
```

```
~> db list
[OK]: ["default", "system", "test"]
```

### `db num` | 数据库数量

获取已加载数据库数量：

```
db num
```

```
~> db num
[OK]: 3
```

### `db lock` | 锁定数据库

锁定指定数据库：

```
db lock <db-name>
```

锁定后的数据库无法被修改。

### `db unlock` | 解锁数据库

解锁指定数据库：

```
db unlock <db-name>
```

## `VALUE` | 输出格式

切换数据返回格式：

```
value style [json|doson]
```

不传参数则返回当前格式。

```
~> value style
[OK]: doson

~> value style json
[OK]: Successful

~> get foo
[OK]: {"String": "bar"}
```

## `PING` | 测试连接

测试服务器连接：

```
ping
```

```
~> ping
[OK]: PONG
```

## `AUTH` | 认证登录

使用密码进行认证：

```
auth <password>
```

如果服务器配置了密码，连接后需要先认证才能执行其他命令。
