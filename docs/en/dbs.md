# Database Management

In `Dorea`, you can create multiple small databases - similar to `1~12` in Redis or `Collection` in Mongo.

## Enable Preload

In the configuration file `config.toml`, you can define up to four preloaded databases. You can include the most frequently used and largest databases:

```
pre_load_group = ["default", "system"] # This is Dorea's default preload configuration
```

During system startup, preloaded databases will consume program startup time to load indexes into memory.

```
[INFO] 2021-12-27T20:42:27.424419+08:00 - index information loaded from "default" [0].
[INFO] 2021-12-27T20:42:27.426828+08:00 - index information loaded from "system" [0].
```

## Reserved Database Names

!> Please do not arbitrarily use databases from the following list for data storage. You are responsible for any issues that arise!

### system

System data storage database. During system runtime, a large amount of runtime data will be written to it, so please do not write any personal data into it.

### egg

Please forgive my selfishness QWQ. I will use this database to secretly store some easter eggs (other places won't have any easter eggs that affect operation, but this is my final bottom line!)

Contributors are also welcome to write their own easter eggs! (If you mind this issue, please Fork a copy of the code and remove the relevant easter egg code)

During normal use, please do not load this database either, as it contains a lot of meaningless data and features!

### dorea

Databases named `dorea` should also not be used arbitrarily (related features will be released later).

## Switch Database

Use the `select` command to directly switch the currently used database.

If you try to switch to a database that is not loaded into memory, the system will load it on the spot. (Large data volumes will have latency).

The best solution for latency is: first use `db preload {db_name}`, then after some time use `select {db_name}`.

This way you can do other things in between!

## Unload Database

Use `db unload {db_name}` to manually unload a database from memory (if a database is in use, it cannot be unloaded).

Unlike manual unloading above, the system will also perform [automatic unload service](https://mrxzx.info/2021/12/23/dorea-design-doc/#Index-Elimination-Mechanism) based on weight.

## Preload Database

Use `db preload {db_name}` to preload. During the preload period, command usage is not affected (the system will start a separate process for loading).
