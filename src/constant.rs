pub(crate) const SUBCOMMAND_INFO_HELP: &'static str = "
- current :                    print current connect database name.
- version :                    print doreadb system version.
- max-connect-number | mcn :   doreadb max connection number.
- server-startup-time | sst :  doreadb server startup time[timestamp].
- total-index-number | tin :   doreadb maximum index number and current index number.
- connect-id | cid :           current connection id number[uuid].
- keys :                       current database key list.
- @{key_name} :
    - expire :                 data expire time[timestamp].
    - timestamp :              data expire time and modify time[timestamp].
    - weight :                 data weight, similar data size.
";

pub(crate) const SUBCOMMAND_EDIT_HELP: &'static str = "
- @{key_name} :
    - incr [number]            add [number] to value: (Number, List) type only.
    - expire [+|-|= number]    use operation [+|-|=] to set the expire time: support all type.
    - insert <key> <value>     insert value to [dict or list], just need one argument for `list`: (List, Dict) type only.
    - remove <key>             remove a value from [dict or list], <key> for list is the index: (List Dict) type only.
    - push <value>             push a value to the list: List type only.
    - pop                      pop the last value from the list: List type only.
    - sort [ASC|DESC]          ASC for positive sequence, and DESC for reverse sequence: List type only.
    - reverse                  reverse the data sequence: (List Tuple) type only.
";