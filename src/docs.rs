pub const SUBCOMMAND_DOCS_HELP: &str = "

[Dorea Command Docs]: https://dorea.mrxzx.info/#/zh-cn/cmd

you can use this command to get the manual for other command.

- docs:     `docs`
- db:       `docs db`
- info      `docs info`
- edit      `docs edit`
- service   `docs service`

";

pub const SUBCOMMAND_SERVICE_HELP: &str = "
- account :
    - set <name> <password> [usa_db] [cls_cmd] \\
        :                               edit or create a account
        : <name>:                       account name
        : <password>:                   account login password
        : <usa_db>:                     the list for you can use database.
        : <cls_cmd>:                    the list for you cannot use command.
    - list :                        print account list.
    - num :                         print account number.
    - repwd <username> <pwd> :      change user password.
    - lock <name> :                 lock(disable) a account.
    - unlock <name> :               unlock(usable) a account.
    - killall :                     kill all using account.
";

pub const SUBCOMMAND_DB_HELP: &str = "
- preload <name> :                  preload a database to system.
- unload <name> :                   unload a database from system.
- list :                            get loaded database list.
- lock <name> :                     lock a database [locked db cannot be unload].
- unlock <name> :                   unlock a database [can be unload].
- status :                          get all database status info.
";

pub const SUBCOMMAND_INFO_HELP: &str = "
- current :                         print current connect database name.
- version :                         print doreadb system version.
- max-connect-number | mcn :        doreadb max connection number.
- server-startup-time | sst :       doreadb server startup time[timestamp].
- total-index-number | tin :        doreadb maximum index number and current index number.
- connect-id | cid :                current connection id number[uuid].
- keys :                            current database key list.
- @{key_name} :
    - expire :                      data expire time[timestamp].
    - timestamp :                   data expire time and modify time[timestamp].
    - weight :                      data weight, similar data size.
";

pub const SUBCOMMAND_EDIT_HELP: &str = "
- @{key_name} :
    - incr [number]                 add [number] to value: (Number, List) type only.
    - expire [+|-|= number]         use operation [+|-|=] to set the expire time: support all type.
    - insert <key> <value>          insert value to [dict or list], just need one argument for `list`: (List, Dict) type only.
    - remove <key>                  remove a value from [dict or list], <key> for list is the index: (List Dict) type only.
    - push <value>                  push a value to the list: List type only.
    - pop                           pop the last value from the list: List type only.
    - sort [ASC|DESC]               ASC for positive sequence, and DESC for reverse sequence: List type only.
    - reverse                       reverse the data sequence: (List Tuple) type only.
";
