# DoreaDB Client

> Dorea 系统将会提供各式各样不同类型的客户端工具。

## Dorea CLI

这个工具是在安装 `dorea-server` 时捆绑安装的工具，它可以使用 `cli` 的方式去连接数据库。

```
dorea 0.2.1
ZhuoEr Liu <mrxzx@qq.com>
Does awesome things

USAGE:
    dorea-cli [OPTIONS]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --hostname <HOSTNAME>    Set the server hostname
    -a, --password <PASSWORD>    Connect password
    -p, --port <PORT>            Set the server port
```

本工具的接入方式为 `TCP Connect`，它是直接使用 `Dorea 协议` 直接连接的。

```
@default> set foo "hello world"
[OK]: Successful
@default> get foo
[OK]: "hello world"
@default> binary upload file /Users/liuzhuoer/Documents/hello.img
[OK]: Successful
@default> get file
[OK]: binary!(....)
```

它对于 `Binary(二进制)` 数据类型也有简单的封装，可以快速上传、下载二进制数据。

!> 鉴于 `DoreaDB` 设计模式的原因，它并不适合用于文件的存储，但可以存储一些不庞大的二进制数据集。

```
@default> set bin binary!(/* 这是二进制数据集的直接存储格式，里面是被 Base64 的二进制集 */)
```

### 命令集

- Get 读取数据：`get ${key}`
- Set 设置数据：`set ${key} ${value}`
- Setex 设置定时数据：`setex ${key} ${value} ${expire}`
- Delete 删除数据：`delete ${key}`
- Clean 清空所有数据：`clean`
- Select 切换数据库：`select ${db-name}`
- Info 获取信息：`info ${key} ${operation}`
- Edit 编辑复合数据：`edit ${key} ${operation} ${sub-value}`
- Ping 测试连接：`ping`
- Binary 二进制专用：`binary ${operation} ${key} ${value}`

> 其中部分命令来源于 `Dorea` 本身命令语句，详情可以查阅命令集页面。

## 开发语言 SDK

> 语言 SDK 即为其他语言封装的调用包，你可以使用开发语言直接连接并操作数据库。

语言 SDK 大部分使用 [`WebService`](/zh-cn/web-service) 完成开发，它们依赖于 Web Service.

- **Python-Driver**: [PyDorea](https://pypi.org/project/pydorea/) 「 Contributor: mrxiaozhuox 」
- **Deno-Driver** [Dorea4d](https://deno.land/x/dorea4d/) 「 Contributor: mrxiaozhuox 」