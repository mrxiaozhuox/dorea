# Web 服务

Dorea-Server 中内置了 `Web Api Service` 服务。

它是一种基于 `HTTP 协议` 的服务，默认为**开启**状态。

!> 我们建议您开启它，因为后续官方所开发并支持的SDK将使用 Web Service

[Api文档](https://docs.apipost.cn/preview/81ea2366835dd14f/f7650f4ead7214fa)

配置文件位于：

```text
[DoreaConfigPath]/service.toml
```

大致内容为（文档编辑版本：#0.3.0-alpha）：

```toml
[foundation]
switch = true
port = 3451
token = "YouNeedChangeThisString"

[account]
master = "YouNeedChangeThisString"
```

 - `swtich` 为 `web service` 的开关，你可以将其设定为 `true` 并重启服务器便可打开 Web 服务。
 - `port` 为 Web 服务所运行的目标端口，我们默认使用 `DoreaPort + 1` 也就是 `3451` 端口。
 - `token` 用于 `JWT` 生成，但是你需要保存好它。如果泄露，任何人都可以自行生成 `JWT` 密钥。

## 账户管理

`Web Service` 拥有一套极其简单的账户管理制度，它允许你为不同数据库配置专用账户。

默认的 `master` 相当于 `Administrator` 最高权限，所有的数据库你都可以直接访问。

在后续你也可以定义一些单独使用的账户：

```
default = "default-group@password"
setting = "setting-group@password"
```

你可以为每一个数据组定义一个密码，使它只能在一个组下被使用。

## 接口访问

接下来将简单的介绍一下接口的访问格式。

`Web Service` 的所有请求都需要使用 `POST` 完成。

访问某个数据库：

```
http://127.0.0.1:3451/@default/[option]
```

例如我们尝试获取 `default` 的基本信息：

```
# 请求路径
http://127.0.0.1:3451/@default/list

# 请求结果
{
    "alpha": "OK",
    "data": {
        "group_name": "default",
        "key_list": [],
        "key_number": 0
    },
    "message": "",
    "resptime": 1629204449
}
```

- `alpha` 字段用于第一时间判断本次请求是否成功（它与 Dorea 协议中的状态相似，有三种值：「 OK, ERR, NOAUTH 」）
- `data` 数据段，会根据请求类型的不同，返回不同数据（操作类型的请求一般只会有 alpha 字段作检查，data 则为空 ）
- `messsage` 字段用于返回错误信息（它只会在 `ERR` 的情况下才拥有内容 ）
- `resptime` 为服务器响应时间，它是一个**时间戳**数据。