# Web Service

Dorea-Server has a built-in `Web Api Service`.

It is a service based on `HTTP protocol` and is **enabled** by default.

!> We recommend you enable it, because the SDKs officially developed and supported later will use Web Service.

[API Documentation](https://docs.apipost.cn/preview/81ea2366835dd14f/f7650f4ead7214fa)

Configuration file location:

```text
[DoreaConfigPath]/service.toml
```

The general content is (document edit version: #0.3.0-alpha):

```toml
[foundation]
switch = true
port = 3451
token = "YouNeedChangeThisString"

[account]
master = "YouNeedChangeThisString"
```

- `switch` is the toggle for `web service`. You can set it to `true` and restart the server to turn on Web service.
- `port` is the target port where the Web service runs. We default to using `DoreaPort + 1`, which is port `3451`.
- `token` is used for `JWT` generation, but you need to keep it safe. If leaked, anyone can generate `JWT` keys themselves.

## Account Management

`Web Service` has an extremely simple account management system that allows you to configure dedicated accounts for different databases.

The default `master` is equivalent to `Administrator` with the highest privileges. You can directly access all databases.

Later you can also define some standalone accounts:

```
default = "default-group@password"
setting = "setting-group@password"
```

You can define a password for each data group, so it can only be used under one group.

## API Access

Next, we will briefly introduce the API access format.

All requests for `Web Service` need to use `POST`.

Access a database:

```
http://127.0.0.1:3451/@default/[option]
```

For example, if we try to get basic information about `default`:

```
# Request path
http://127.0.0.1:3451/@default/list

# Request result
{
    "alpha": "OK",
    "data": {
        "group_name": "default",
        "key_list": [],
        "key_number": 0
    },
    "message": "",
    "resptime": 1629204449
}
```

- `alpha` field is used to immediately determine whether this request was successful (it's similar to the status in the Dorea protocol, with three values: "OK, ERR, NOAUTH")
- `data` data segment, will return different data according to different request types (operation type requests generally only have alpha field for checking, data is empty)
- `message` field is used to return error information (it only has content in `ERR` cases)
- `resptime` is the server response time, it's a **timestamp** data.
