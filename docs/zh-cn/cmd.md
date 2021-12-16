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

## `SET` | 设置

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

!> 本功能开发中，暂不可用哦！

用于数据的查找（包括Key模糊查找）

```
search <match> { source: key | value } [options]
```

- match: 匹配语句（需要用!包裹起来）
- source: 来源，可选为 `Key` 和 `Value`
- options: 一些拓展选项

```
~> search !*.user! key
[OK]: ["admin.user", "mrxzx.user", "foo.user"]
```

通过本命令可模糊查找数据库信息。

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