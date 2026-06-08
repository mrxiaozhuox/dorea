# Dorea-DB Plugin System

> Dorea uses `Lua` as the plugin development language. You can use `lua` to freely customize your database system.

### Why Choose Lua?

I thought about this question for a long time. For `Rust` itself, we could completely use `Python` as the plugin development language, or directly use `RPC` to support all programming languages. But ultimately I chose `Lua` language.

`Lua` language is much lighter and easier to deploy and install, while also giving developers a pure development environment.

When you don't import other libraries (libraries developed in C, etc.), what you can use is only the purest `UserData` provided by Dorea.

### Design Scheme

This plugin system is just a ~~rough template~~ (planned feature), so it is not enabled by default.

You need to `clone` the compatible `dorea-plugin-loader` and install `lua 5.4` to officially use it.

```
git clone https://github.com/doreadb/dorea-plugin-loader.git plugin
```

### Plugin Events

Currently the plugin supports the following event management:

- plugin_onload - On load
- plugin_unload - On unload
- plugin_interval - Periodic task
- custom_command.xxx - Custom command handler

!> Unload is not yet implemented (because I don't know when it will be unloaded, maybe when custom transactions are made in the future)

### Prerequisites

Before you officially start using the plugin system, you need (or we recommend) to install the following tools:

- lua 5.4 (Newer version of Lua, the system only supports this version) Website: [link](https://www.lua.org)
- luarocks (Lua's package management tool, some plugins may need to install some prerequisite libraries, please pay attention to plugin documentation) Website: [link](https://luarocks.org)
