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

!> This feature is under development and currently unavailable!

Used for data search (including key fuzzy search):

```
search <match> { source: key | value } [options]
```

- match: Match statement (needs to be wrapped with !)
- source: Source, can be `Key` or `Value`
- options: Some extended options

```
~> search !*.user! key
[OK]: ["admin.user", "mrxzx.user", "foo.user"]
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
