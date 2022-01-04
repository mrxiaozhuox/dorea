pub const SUBCOMMAND_DOCS_HELP: &'static str = "

[Dorea Command Docs]: https://dorea.mrxzx.info/#/zh-cn/cmd

you can use this command to get the manual for other command.

- docs:     `docs`
- db:       `docs db`
- info      `docs info`
- edit      `docs edit`

";

pub const SUBCOMMAND_DB_HELP: &'static str = "
## Subcommand
- preload <name> :              preload a database to system.
- unload <name> :               unload a database from system.
- list :                        get loaded database list.
- lock <name> :                 lock a database [locked db cannot be unload].
- unlock <name> :               unlock a database [can be unload].
- status :                      get all database status info.
";

pub const SUBCOMMAND_INFO_HELP: &'static str = "
## Subcommand
- current :                     print current connect database name.
- version :                     print doreadb system version.
- max-connect-number | mcn :    doreadb max connection number.
- server-startup-time | sst :   doreadb server startup time[timestamp].
- total-index-number | tin :    doreadb maximum index number and current index number.
- connect-id | cid :            current connection id number[uuid].
- keys :                        current database key list.
- @{key_name} :
    - expire :                  data expire time[timestamp].
    - timestamp :               data expire time and modify time[timestamp].
    - weight :                  data weight, similar data size.
";

pub const SUBCOMMAND_EDIT_HELP: &'static str = "
## Subcommand
- @{key_name} :
    - incr [number]             add [number] to value: (Number, List) type only.
    - expire [+|-|= number]     use operation [+|-|=] to set the expire time: support all type.
    - insert <key> <value>      insert value to [dict or list], just need one argument for `list`: (List, Dict) type only.
    - remove <key>              remove a value from [dict or list], <key> for list is the index: (List Dict) type only.
    - push <value>              push a value to the list: List type only.
    - pop                       pop the last value from the list: List type only.
    - sort [ASC|DESC]           ASC for positive sequence, and DESC for reverse sequence: List type only.
    - reverse                   reverse the data sequence: (List Tuple) type only.
";