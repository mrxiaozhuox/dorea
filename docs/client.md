# Client

In this chapter we will use `dorea-cli` to execute for `CURD` .

## Set

When the server first started, we didn't have any data available in our database.

so let's try to insert some value in to the db!

```dorea-cli
127.0.0.1:3450 ~> set foo "bar"
OK
```

try to run `set foo "bar"`, we will insert a value in to db.

```
key: foo
value: DataValue::String(bar)
expire: None
```
This is the actual structure of the data that we inserted.

### Data Type

Dorea will auto-derivation data-type for value.

for example:

```
String: enclosed in quotes      "Hello World"
Number: any numbers             3.14
Boolean: True or False          true
Dict: Json format               {"hello":"world"}
ByteVector: Byte[]              Byte[1,2,3,4,5,6,7,8,9,0]
```

### Expire Time

If you want the data to be valid only for a certain period of time, you can set the `expire`

```dorea-cli
127.0.0.1:3450 ~> set info {"username":"mrxiaozhuox"} 10
```

This data will expire in ten seconds. (system will be remove it)

## Update

if you want to change a value, you just need use `set` to overwrite it.

```dorea-cli
127.0.0.1:3450 ~> get foo
String("bar")
127.0.0.1:3450 ~> set foo 1
OK
127.0.0.1:3450 ~> get foo
Number(1)
```

## Get

Now, we will try to `get` some data from Dorea.

```dorea-cli
127.0.0.1:3450 ~> get foo
String("bar")
127.0.0.1:3450 ~> get undefined
Undefined()
```

the response will li like this: `Type(value)`. And if data not found, will return `Undefined()`.

### Fuzzy matching

`!Unstable` being development.


## Remove

Delete some value from Dorea.

```dorea-cli
127.0.0.1:3450 ~> remove foo
OK
127.0.0.1:3450 ~> get foo
Undefined()
```

## Select

If you wan't choose other `db-group`, you can use `select` to change it.

```dorea-cli
127.0.0.1:3450 ~> info current
db: default
127.0.0.1:3450 ~> select dorea-cli
OK
127.0.0.1:3450 ~> info current
db: dorea-cli
```
use `info current` can get the current database name.

### Default Group

- default      : Default group, highest priority.
- dorea        : Maybe will save some system data.
- file-storage : FileStorage group, save file section.


## Clean

You can use `clean` to remove all data in the group.

```dorea-cli
127.0.0.1:3450 ~> info cache-num
10
127.0.0.1:3450 ~> clean
OK
127.0.0.1:3450 ~> info cache-num
0
```

`info cache-num` can get cache data num (not all data in storage)