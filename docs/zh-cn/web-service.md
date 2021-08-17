# Web 服务

Dorea-Server 中内置了 `Web Api Service` 服务。

它是一种基于 `HTTP 协议` 的服务，默认为**关闭**状态。

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