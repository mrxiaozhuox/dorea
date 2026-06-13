# System Commands

> This document introduces the built-in commands of the system.

```rust
// Internal command list definition
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
    DB,
    DOCS,
    SERVICE,
}
```

## `GET` | Read

Use the `Get` command to retrieve the specific content of data:

```
get <key>
```

Enter the `Key` information to directly retrieve the corresponding data:

```
~> get foo
[OK]: "bar"
```

## `SET` | Write

Use the `Set` command to insert data into the database:

```
set <key> <value> [expire]
```

- value: Please write the structure strictly according to the [DOSON](/en/data-value) specification.
- expire: Expiration time; leave empty or `0` for no expiration time. [Optional]

If the value contains spaces, use double quotes to wrap it. Escaping with `\"` is supported inside quotes:

```
~> set foo "bar"
[OK]: Successful

~> set msg "hello world"
[OK]: Successful

~> set note "say \"hi\""
[OK]: Successful
```

## `DELETE` | Delete

Use `Delete` to remove a piece of data:

```
delete <key>
```

After deletion, the data cannot be accessed (unless it is `set` again).

## `CLEAN` | Clear

Use `Clean` to clear a database:

```
clean [group]
```

If no `group` information is provided, it defaults to clearing the data of the `current` database.

## `SELECT` | Switch Database

Each `Dorea` service can create multiple databases, and you can switch between them using `Select`:

```
select <group> [options]
```

- options: Some other optional parameters [This feature is planned and under development]
  - preload: Pre-load (this will not switch to the target database, but will pre-load the index first)

If a database is loaded for the first time (i.e., it hasn't been loaded during this service startup), the index needs to be loaded with time complexity *O(n)*.

A database that has never been created will also be automatically created when this command is called!

## `SEARCH` | Data Search

Used for data search (key and value content search):

```
search <pattern>              # search key + value (default)
search key <pattern> [limit]   # search key only
search value <pattern> [limit] # search value content only
```

### Matching Rules

| Pattern | Meaning | Example |
|---------|---------|---------|
| `word` | substring | `admin` matches `user:admin`, `admin_config` |
| `^word` | prefix | `^user` matches `user:xxx` |
| `word$` | suffix | `.log$` matches `error.log` |
| `*` `?` | wildcards | `user*:?dmin` |

### Examples

```
~> search admin
[OK]: ["user:admin", "admin_config"]

~> search key ^user
[OK]: ["user:admin", "user:mrxzx"]

~> search value hello
[OK]: ["key1", "test_key"]
```

This command can be used to fuzzy search database information.

## `INFO` | Information Retrieval

This command is used to retrieve basic information about the **database** or **data item**.

```
info <option>
```

### `Current` | Current Database

```
info current
```

Get the currently selected database (use `SELECT` to switch):

```
~> info current
[OK]: default
```

### `Version` | Version Number

```
info version
```

Get the current Dorea version number:

```
~> info version
[OK]: V0.4.0
```

### `Keys` | Key List

```
info keys
```

Get the list of `keys` in the current database:

```
~> info keys
[OK]: ["foo", "hello", "example"]
```

The time complexity is *O(n)* (time complexity optimization will be on the agenda).

### `Max-Connect-Number` | Maximum Connections

```
info max-connect-number
info mcn
```

Get the server maximum connection configuration.

### `Total-Index-Number` | Total Index Count

```
info total-index-number
info tin
```

Get statistics of currently loaded index count.

### `Server-Startup-Time` | Server Startup Time

```
info server-startup-time
info stt
```

Get the server startup time.

### `Connect-Id` | Connection ID

```
info connect-id
info cid
```

Get the unique ID of the current connection.

### `@Key` | Data Item Details

Get detailed information (metadata) of a specified key:

```
info @<key> [sub-info]
```

Available `sub-info` parameters:

- `expire` - Expiration time
- `timestamp` - Timestamp
- `weight` - Weight

```
~> info @foo
[OK]: MetaNode { ... }

~> info @foo expire
[OK]: 0

~> info @foo timestamp
[OK]: (1626470590, 0)
```

## `EDIT` | Edit Composite Data

Operate on existing composite data (List, Dict):

```
edit @<key> <operation> [args...]
```

### `incr` | Increment Number

Increment a number or numbers within composite data:

```
edit @<key> incr [amount]
```

- amount: Increment amount, defaults to 1

```
~> set counter 10
~> edit @counter incr
[OK]: Successful
~> get counter
[OK]: 11

~> edit @counter incr 5
[OK]: Successful
~> get counter
[OK]: 16
```

### `expire` | Set Expiration Time

Modify the expiration time of data:

```
edit @<key> expire <time>
```

- `+N` - Add N seconds
- `-N` - Subtract N seconds
- `=N` or `N` - Set to N seconds

```
~> edit @foo expire +60    # Add 60 seconds
~> edit @foo expire -30    # Subtract 30 seconds
~> edit @foo expire =100   # Set to 100 seconds
```

### `insert` | Insert Data

Insert data into Dict or List:

```
edit @<key> insert <value> [index/key]
```

- Dict: `edit @dict insert "value" "key"`
- List: `edit @list insert "value" [index]`

```
~> set mylist [1, 2, 3]
~> edit @mylist insert 4
~> get mylist
[OK]: [1, 2, 3, 4]

~> set mydict {"a": 1}
~> edit @mydict insert 2 "b"
~> get mydict
[OK]: {"a": 1, "b": 2}
```

### `remove` | Remove Data

Remove data from Dict or List:

```
edit @<key> remove <index/key>
```

```
~> edit @mylist remove 0    # Remove first element of List
~> edit @mydict remove "a"  # Remove item with key "a" from Dict
```

### `push` | Append Element

Append element to the end of List:

```
edit @<key> push <value>
```

```
~> set mylist [1, 2]
~> edit @mylist push 3
~> get mylist
[OK]: [1, 2, 3]
```

### `pop` | Pop Element

Pop the last element from List:

```
edit @<key> pop
```

```
~> set mylist [1, 2, 3]
~> edit @mylist pop
~> get mylist
[OK]: [1, 2]
```

### `sort` | Sort

Sort a List:

```
edit @<key> sort [order]
```

- order: `asc` (ascending, default) or `desc` (descending)

```
~> set mylist [3, 1, 2]
~> edit @mylist sort
~> get mylist
[OK]: [1, 2, 3]

~> edit @mylist sort desc
~> get mylist
[OK]: [3, 2, 1]
```

### `reverse` | Reverse

Reverse a List or composite data:

```
edit @<key> reverse
```

```
~> set mylist [1, 2, 3]
~> edit @mylist reverse
~> get mylist
[OK]: [3, 2, 1]
```

## `DB` | Database Management

Database management commands:

### `db unload` | Unload Database

Unload a specified database from memory:

```
db unload <db-name>
```

Cannot unload a database that is currently in use.

### `db preload` | Preload Database

Preload a specified database (asynchronous loading):

```
db preload <db-name>
```

Preloading does not affect the execution of other commands.

### `db list` | List Databases

List all loaded databases:

```
db list
```

```
~> db list
[OK]: ["default", "system", "test"]
```

### `db num` | Database Count

Get the number of loaded databases:

```
db num
```

```
~> db num
[OK]: 3
```

### `db lock` | Lock Database

Lock a specified database:

```
db lock <db-name>
```

Locked databases cannot be modified.

### `db unlock` | Unlock Database

Unlock a specified database:

```
db unlock <db-name>
```

## `VALUE` | Output Format

Switch the data return format:

```
value style [json|doson]
```

Without arguments, returns the current format.

```
~> value style
[OK]: doson

~> value style json
[OK]: Successful

~> get foo
[OK]: {"String": "bar"}
```

## `PING` | Test Connection

Test server connection:

```
ping
```

```
~> ping
[OK]: PONG
```

## `AUTH` | Authentication

Authenticate with password:

```
auth <password>
```

If the server is configured with a password, you must authenticate first before executing other commands.
