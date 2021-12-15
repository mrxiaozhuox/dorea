# 插件设计 API

> Dorea-Plugin 中拥有一些简单的 API 供开发者使用！

## 数据库操作

插件系统中的数据库操作为原生的函数调用（即为整个 **Dorea** 系统中最底层的增删改查函数）

```lua
module = require("interface")

-- 打开一个数据库（name: default）
local db = module.db:open("default")

-- 插入数据
db:setex("key", "value", 0)

-- 读取数据
assert(db:get("key") == "value")

```

## 日志操作

在插件中也支持直接向 `Dorea` 输出运行日志：
```lua
logger:trace("追踪！")
```
```lua
logger:info("信息！")
```
```lua
logger:debug("调试！")
```
```lua
logger:warn("警告！")
```
```lua
logger:error("错误！")
```

它们的输出样式将与 `Dorea` 内部日志一样，当然你可以进行自定义。