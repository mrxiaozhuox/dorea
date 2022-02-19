# Web 网络入口

在 Dorea 系统中，我们一般简称它为：`Service`，它允许我们使用 HTTP\HTTPS 请求进行数据库交互。

```
http://127.0.0.1:3451/@default/execute
body: {
    authorization: "JWT-Token",
    query: "get foo",
}
Result: "bar"
```

我们只需要访问对应的路径，`XXX/@{数据库名}/{目标命令名}` 并传入相应的验证密钥和参数，即可运行命令。

## 鉴权方案

目前我们使用了 `JWT` 方案完成鉴权，这使得接口得到了应有的安全保障。

```
http://127.0.0.1:3451/auth
body: {
    username: "master",
    password: "DOREA@SERVICE"
}
RESULT: { ... token: "JWT_TOKEN" }
```

通过以上方法可以获取 `JWT` 验证密钥，`Service` 支持多账户的权限管理。
我们使用 `master` 账号作为最高权限，它的权限等于 `TCP` 中的账号，即它可以执行任何命令！也不受到任何限制。

账号的组成结构：

```
{
    name: "lab",
    password: "XXXX",
    usa_db: ["lab", "default"],
    cls_command: [
        "service@account@set",
        "service@account@repwd",
        "service@account@lock",
        "service@account@unlock",
        "service@account@killall",
        "db@unload",
        "db@lock",
        "db@unlock",
        "db@preload"
    ],
    checker: "XXXX"
}
```

- `usa_db` 代表当前账号允许操作的数据库列表
- `cls_command` 代表当前账号不允许被使用的命令（通过 `@` 声明子命令不允许被使用）
- `checker` 用于对账号的状态进行检测

!> 建议普通账号直接关闭上述的命令调用，这些命令都是相当危险的！

## 调用原理