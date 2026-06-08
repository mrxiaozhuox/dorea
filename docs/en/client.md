# DoreaDB Client

> Dorea system provides various types of client tools.

## Dorea CLI

This tool is bundled when installing `dorea-server`. You can connect to the database using `cli` mode.

```
dorea 0.4.0
YuKun Liu <mrxzx.info@gmail.com>
A Key-Value Storage System

USAGE:
    dorea-cli [OPTIONS]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --hostname <HOSTNAME>    Set the server hostname
    -a, --password <PASSWORD>    Connect password
    -p, --port <PORT>            Set the server port
```

This tool connects via `TCP Connect` and directly uses the `Dorea protocol` to connect.

```
@default> set foo "hello world"
[OK]: Successful
@default> get foo
[OK]: "hello world"
@default> binary upload file /Users/liuzhuoer/Documents/hello.img
[OK]: Successful
@default> get file
[OK]: binary!(....)
```

It also has simple encapsulation for `Binary` data types, allowing quick upload and download of binary data.

!> Given the design pattern of `DoreaDB`, it is not suitable for file storage, but it can store some small-scale binary datasets.

```
@default> set bin binary!(/* This is the direct storage format for binary datasets, containing Base64-encoded binary data */)
```

### Command Set

- Get retrieve data: `get ${key}`
- Set write data: `set ${key} ${value}`
- Setex write with expiration: `setex ${key} ${value} ${expire}`
- Delete remove data: `delete ${key}`
- Clear all data: `clean`
- Select switch database: `select ${db-name}`
- Info get information: `info ${option}`
- Edit modify composite data: `edit @${key} ${operation} ${args...}`
- Ping test connection: `ping`
- Binary binary-specific: `binary ${operation} ${key} ${value}`
- Db database management: `db ${operation} ${args...}`
- Value output format: `value style ${format}`

> Some commands are from the `Dorea` command set itself. For details, please refer to the command set page.

## Language SDKs

> Language SDKs are packages encapsulated for other languages, you can use programming languages to directly connect and operate the database.

Most language SDKs are developed using [`WebService`](/en/web-service), and they depend on Web Service.

- **Python-Driver**: [PyDorea](https://pypi.org/project/pydorea/) - Contributor: mrxiaozhuox
- **Deno-Driver**: [Dorea4d](https://deno.land/x/dorea4d/) - Contributor: mrxiaozhuox
