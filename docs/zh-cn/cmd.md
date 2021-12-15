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

# `SET` | 设置

通过 `Set` 命令插入某条数据到数据库中：

```
set <key> <value> [exipre]
```

- value: 结构请严格按照 [DOSON](/zh-cn/data-value) 规范编写。
- expire: 过期时间；留空或 `0` 代表无过期时间。[可空]

```
~> set foo "bar"
[OK]: Successful
```

# `DELETE` | 删除

通过 `Delete` 删除一条数据：

```
delete <key>
```
删除后将无法再访问这条数据：
```
~> delete foo
[OK]: Successful
```