<p align="center">
    <p align="center"><img src="./assets/DoreaDB.png" width="120"/></p>
	<p align="center">
    <a href="https://github.com/mrxiaozhuox/Dorea/actions">
    	<img alt="Build" src="https://img.shields.io/github/workflow/status/mrxiaozhuox/Dorea/Rust?style=for-the-badge" />
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

## Screenshot

![](http://wans-cloud.test.upcdn.net/uploads%2F2021%2F08%2F26%2F1_5PJTELnd_%E6%B7%B1%E5%BA%A6%E6%88%AA%E5%9B%BE_20210826191636.png)

## Docker

```
docker pull mrxiaozhuox/dorea
```