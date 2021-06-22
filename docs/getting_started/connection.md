# Connection

Dorea will create a `TCP` server, so you can use any `TCP Tool` to connect it.

## Netcat

`nc` can help us connect the tcp server.

```shell
nc -v 127.0.0.1 3450
```

```
found 0 associations
found 1 connections:
     1:	flags=82<CONNECTED,PREFERRED>
	outif lo0
	src 127.0.0.1 port 51394
	dst 127.0.0.1 port 3450
	rank info not available
	TCP aux info available

Connection to 127.0.0.1 port 3450 [tcp/castorproxy] succeeded!
+connected
```

try to input `info current` to get current db.

```
info current
+db: default
```

If it has a similar return value, you have succeeded.

> The native tcp connection does not support Dorea well, it is recommended to use cli.

## Dorea Cli

`dorea-cli` will use `dorea-client` to interaction with `dorea-server` .

```shell
# [default] hostname: "127.0.0.1" port: 3450 password: ""
dorea-cli

# -h & --hostname: HOSTNAME
# -p & --port    : PORT
# -a & --password: PASSWORD
dorea-cli -h 127.0.0.1 -p 3450 -a 123456
```

Then you can connect to the server:

```
127.0.0.1:3450 ~> 
```

Try to use `select` command.

```
127.0.0.1:3450 ~> select my-app
OK
127.0.0.1:3450 ~> 
```

Great, it worked !