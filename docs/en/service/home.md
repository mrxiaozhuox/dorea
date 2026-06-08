# Web Network Entry

In the Dorea system, we generally refer to it as: `Service`. It allows us to interact with the database using HTTP\HTTPS requests.

```
http://127.0.0.1:3451/@default/execute
body: {
    authorization: "JWT-Token",
    query: "get foo",
}
Result: "bar"
```

We only need to access the corresponding path, `XXX/@{database-name}/{target-command-name}` and pass in the corresponding verification key and parameters to run the command.

## Authentication

Currently we use the `JWT` scheme for authentication, which provides proper security for the interface.

```
http://127.0.0.1:3451/auth
body: {
    username: "master",
    password: "DOREA@SERVICE"
}
RESULT: { ... token: "JWT_TOKEN" }
```

Through the above method, you can obtain the `JWT` verification key. `Service` supports multi-account permission management.

We use the `master` account as the highest privilege, which has permissions equivalent to the `TCP` account, meaning it can execute any command without any restrictions!

Account structure:

```
{
    name: "lab",
    password: "XXXX",
    usa_db: ["lab", "default"],
    cls_command: [
        "service@account@set",
        "service@account@repwd",
        "service@account@lock",
        "service@account@unlock",
        "service@account@killall",
        "db@unload",
        "db@lock",
        "db@unlock",
        "db@preload"
    ],
    checker: "XXXX"
}
```

- `usa_db` represents the list of databases the current account is allowed to operate on
- `cls_command` represents commands not allowed for the current account (use `@` to declare subcommands not allowed)
- `checker` is used to check the account status

!> It is recommended to directly disable the above command calls for regular accounts, as these commands are quite dangerous!

## Calling Principle

`Dorea Service` currently opens a `TCP Client` for each call. This operating efficiency is not high, so we are working on solutions.
