<p align="center">
	<h2 align="center">Dorea DB 🛰</h2>
	<p align="center">
    <a href="https://github.com/mrxiaozhuox/Dorea/actions">
    	<img alt="Build" src="https://img.shields.io/github/workflow/status/mrxiaozhuox/Dorea/Rust?style=for-the-badge" />
    </a>
    <a href="https://github.com/mrxiaozhuox/Dorea/blob/master/LICENSE">
      <img alt="GitHub" src="https://img.shields.io/github/license/mrxiaozhuox/Dorea?style=for-the-badge">
    </a>
    <a href="https://github.com/mrxiaozhuox/Dorea/blob/master/LICENSE">
			<img alt="Code" src="https://img.shields.io/github/languages/code-size/mrxiaozhuox/Dorea?style=for-the-badge">
    </a>
	</p>
	<p align="center">
    <strong>Dorea 是一款 Key - Value 数据存储系统。它基于 Bitcask 存储模型实现！</strong>
	</p>
	<p align="center">
    <a href="http://dorea.mrxzx.info/">文档</a> | 
    <a href="https://crates.io/crates/dorea">Crates.io</a> | 
    <a href="https://docs.rs/dorea/">Core 文档</a>
	</p>
	<p align="center">
    <a href="https://github.com/mrxiaozhuox/dorea/blob/master/README.CN.md">简体中文</a> | 
    <a href="https://github.com/mrxiaozhuox/dorea/blob/master/README.md">English</a>
	</p>
</p>



## 功能

- Dorea 插件管理器: [Github Repo](https://github.com/doreadb/dorea-plugin-loader)

### 数据类型

`Dorea` 将会实现基本的数据类型与复合类型：

- String 基础字符串
- Number 数字类型 - 使用 `float 64` 存储
- Boolean 布尔值
- Binary 二进制数据
- List \<DataValue> 列表（复合），可将任何数据类型作为元素插入
- Dict \<String, DataValue> 字典（复合），可将任何数据类型作为元素插入
- Tuple \<DataValue, DataValue> 元组（复合），可将任何数据类型作为元素插入



## 存储模型

`Dorea` 基于 Bitcask 存储模型实现，`bitcask` 是一种 *日志型* 的存储模型。

所有 **添加、更新、删除** 操作都是以追加的方式实现的。

```
key: foo | value: "bar" | timestamp: 1626470590043 # 插入了一条新的数据
key: foo | value: "new" | timestamp: 1626470590043 # 更新了数据（不会删除上面的插入）
key: foo | value:  none | timestamp: 1626470590043 # 删除了数据（也不会影响到上面的插入与更新）
```

当一个存储文件到达一个最大容量时，则将其归档，并新建一个写入文件。

### 索引加载

在 `Bitcask` 中，索引会被一次性全部加载到内存中。

但是在 `Dorea` 中，你需要配置默认自动加载的 `Group` ，当有程序切换 `Group` 时，才去加载相应的索引。

**PS: ** 在数据量不大（< 100w）时，这种加载的时间成本都是可以忽略不计的。 

### Merge 操作

程序会每隔一段时间对已归档的文件进行整理（将多余的数据删除），让其只保存最后一次的操作信息。

## 传输协议

Dorea 使用基于 TCP 的二进制长度前缀协议，每帧格式如下：

```
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

命令参数支持双引号包裹含空格的值：

```
set foo "hello world"        # key=foo, value="hello world"
set bar [1,2,3]              # key=bar, value=[1,2,3]
set baz "escaped \"quote\""  # key=baz, value=escaped "quote"
```



## Core API

`Dorea-core` 开放了部分内部功能供开发者调用。

- Server - 服务端（没啥可自定义的，就是单纯一个启动函数）
- Client - 客户端 （对于基本的操作都封装成了函数）
- Processor - 处理程序（包含数据解析器、协议解析器等）

## 部分截图

![](https://upc.cloud.wwsg18.com/uploads%2F2021%2F08%2F26%2F1_5PJTELnd_%E6%B7%B1%E5%BA%A6%E6%88%AA%E5%9B%BE_20210826191636.png)