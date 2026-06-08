# Plugin Design API

> Dorea-Plugin has some simple APIs for developers to use!

## Database Operations

Database operations in the plugin system are native function calls (i.e., the lowest-level CRUD functions in the entire **Dorea** system):

```lua
module = require("interface")

-- Open a database (name: default)
local db = module.db:open("default")

-- Insert data
db:setex("key", "value", 0)

-- Read data
assert(db:get("key") == "value")

```

## Logging Operations

Plugins also support directly outputting runtime logs to `Dorea`:

```lua
logger:trace("Trace!")
```
```lua
logger:info("Info!")
```
```lua
logger:debug("Debug!")
```
```lua
logger:warn("Warning!")
```
```lua
logger:error("Error!")
```

Their output style will be the same as `Dorea` internal logs, and of course you can customize them.
