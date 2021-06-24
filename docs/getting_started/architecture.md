# Architecture

📦 Now we will talk about the `architecture` for `Dorea` 

## Data Storage

when you start the server, you can see the `Dorea` data storage path.

```
The storage dir: "/Users/liuzhuoer/Library/Application Support/Dorea"
```

the directory struct :

```
- Dorea
	- config.toml
	- storage
		- @default
		- @dorea
	- log
```

### config.toml

You can edit this file to change `Dorea` configure.

```toml
[common]
connect_password = ""
maximum_connect_number = 98
maximum_database_number = 20

[memory]
maximum_memory_cache = 512
persistence_interval = 40000

[database]
default_database = "default"
```

### storage

All data will save in this directory.

The sub-directory was `database-group` :

```
select dorea     // database-group: dorea
select user-info // database-group: user-info
```

Struct for data save :

```
- @default
	- f
		- o
			- o.db
```

This struct was save for key: `foo` , It looks like a BTree.

### log

The log directory will save all runtime log.

```
- log
	- curd.log			# curd operations
	- expired.log   # expired informations
	- handle.log		# handle file log
	- server.log		# server informations
```