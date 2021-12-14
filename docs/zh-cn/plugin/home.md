# Dorea-DB 插件系统

> Dorea 采用 `Lua` 作为插件开发语言，你可以使用 `lua` 自由的定制您的数据库系统。

### 为什么选择 lua 语言？

这个问题我思考了很久。对于 `Rust` 本身而言，我们完全可以使用 `Python` 作为插件开发语言，
或者是直接上 `RPC` 的话支持所有的编程语言。但是我最终还是选择了 `Lua` 语言。

`Lua` 语言相对来说会轻便很多很多，部署安装方便的同时，也能给开发者一个纯粹的开发环境。

在你不引入其他库（使用 C 等语言开发的库）的情况下，你能使用的仅仅是 Dorea 所提供的最纯粹的 `UserData`.

### 设计方案

本套插件系统只是先做个~~大概的模板~~（画饼），所以说他并没有默认开启。

你需要 `clone` 兼容的 `dorea-plugin-loader` 并安装 `lua 5.4` 才能正式使用它。

```
git clone https://github.com/doreadb/dorea-plugin-loader.git plugin
```

### 插件事件

目前插件支持以下的事件管理：

- plugin_onload 加载时
- plugin_unload 卸载时
- plugin_interval 定期任务
- custom_command.xxx 自定义命令处理程序

!> unload 卸载还未实现（因为我不知道什么时候会卸载，可能在以后自定义事务做出来再说吧）

### 前置程序

在您正式开始使用插件系统前，您需要（或建议）安装以下工具：

- lua 5.4 ( Lua 较新的版本，系统只支持这个版本 ) 官网：[link](https://www.lua.org)
- luarocks ( Lua 的包管理工具，部分插件可能需要安装一些前置library，请留意插件文档 ) 官网：[link](https://luarocks.org)