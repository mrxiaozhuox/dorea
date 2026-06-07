<p align="center">
    <p align="center"><img src="./assets/DoreaDB.png" width="120"/></p>
 <p align="center">
    <a href="https://github.com/mrxiaozhuox/Dorea/actions">
     <img alt="Build" src="https://img.shields.io/github/actions/workflow/status/mrxiaozhuox/dorea/test.yml?style=for-the-badge" />
    </a>
    <a href="https://github.com/mrxiaozhuox/Dorea/blob/master/LICENSE">
      <img alt="GitHub" src="https://img.shields.io/github/license/mrxiaozhuox/Dorea?style=for-the-badge">
    </a>
    <a href="https://github.com/mrxiaozhuox/Dorea/blob/master/LICENSE">
   <img alt="Code" src="https://img.shields.io/github/languages/code-size/mrxiaozhuox/Dorea?style=for-the-badge">
    </a>
 </p>
 <p align="center">
    <strong>Dorea is a key-value data storage system. It is based on the Bitcask storage model</strong>
 </p>
 <p align="center">
    <a href="http://dorea.mrxzx.info/">Documentation</a> |
    <a href="https://crates.io/crates/dorea">Crates.io</a> |
    <a href="https://docs.rs/dorea/">API Doucment</a>
 </p>
 <p align="center">
    <a href="https://github.com/mrxiaozhuox/dorea/blob/master/README.CN.md">简体中文</a> |
    <a href="https://github.com/mrxiaozhuox/dorea/blob/master/README.md">English</a>
 </p>
</p>

### Data Sturct

`Dorea` have the basic data type and some compound type.

- String
- Number
- Boolean
- Binary
- List \<DataValue>
- Dict \<String, DataValue>
- Tuple \<DataValue, DataValue>

## Storage Model

`dorea` based on the `Bitcask` storage model. **(Log)**

All **insert, update, delete** operations are implemented as appends.

```
key: foo | value: "bar" | timestamp: 1626470590043 # Insert Value
key: foo | value: "new" | timestamp: 1626470590043 # Update Value (append info)
key: foo | value:  none | timestamp: 1626470590043 # Remove Value (append info)
```

When a storage file reaches a maximum capacity, it is archived and a new write file is created.

## Transport Protocol

Dorea uses a binary length-prefixed protocol over TCP. Each frame has the following format:

```
+----------+----------+----------+----------+
|  MAGIC   | VERSION  |  STATE   |   LEN    |
|  2 bytes | 1 byte   | 1 byte   | 4 bytes  |
+----------+----------+----------+----------+
|              PAYLOAD (LEN bytes)          |
+-------------------------------------------+
```

| Field | Size | Description |
|-------|------|-------------|
| MAGIC | 2 bytes | Fixed `0xD0 0x9A`, used for stream alignment and validation |
| VERSION | 1 byte | Protocol version, currently `0x01` |
| STATE | 1 byte | Status code: `0x00`=IGNORE, `0x01`=OK, `0x02`=ERR, `0x03`=EMPTY, `0x04`=NOAUTH |
| LEN | 4 bytes | Payload length in big-endian (u32), max 16MB |
| PAYLOAD | LEN bytes | Raw binary data (command text in UTF-8 or response bytes) |

Command arguments support double-quoted strings for values containing spaces:

```
set foo "hello world"        # key=foo, value="hello world"
set bar [1,2,3]              # key=bar, value=[1,2,3]
set baz "escaped \"quote\""  # key=baz, value=escaped "quote"
```

