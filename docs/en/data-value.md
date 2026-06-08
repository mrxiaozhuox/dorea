# Data Types

> The current version of Dorea supports most commonly used basic types and composite types.

## String

String is one of the most important data types. It allows us to store any data composed of characters:

```
"hello"                      // Simple characters
"12345"                      // Characters with numbers
"{\"name\": \"mrxiaozhuox\"}" // Some serialized format
"aGVsbG8gd29ybGQ="           // Binary data can also be serialized as string
```

It can cover almost any content and data! **I call it the most powerful data type!**

It uses double quotes `""` to wrap.

## Number

The `Number` type in this system uses `f64` for storage because it supports both decimals, positive/negative numbers, and integers:

```
18      // Basic integer
3.14    // Decimal
18.0    // Decimal
-231    // Negative number
3.1415  // Maximum precision to 4 decimal places, rest will be truncated
```

The maximum value of `f64` is: `1.7976931348623157e+308`

## Boolean

Boolean is a logical value. Logical values only distinguish between `true` and `false`, so there are only two possibilities:

```
true   // True
false  // False
```

## Binary

This type was introduced in `Dorea-V0.3.0`. It supports direct storage of serialized binary data.

```
binary!(
    PHAgYWxpZ249ImNlbnRlciI...
)
```

It uses `Base64` for serialization, but SDKs in various languages will encapsulate the storage function. **[The essential storage method is u8 array]**

## List

`List` in `DoreaDB` is essentially an `Array`. You can store a series of data in it:

```
[
    3.14,
    "string",
    false,
    binary!(...),
    [
        binary!(...),
        "hello world",
    ],
]
```

You can nest any data type inside, including nesting a new list.

> This type can be operated and modified using the `Edit` command.

## Dict

`Dict` in `DoreaDB` is essentially a `Map` key-value pair. You can store a series of data in it (including keys):

```
{
    "hello": "string",
    "pi": 3.14,
    "bin": binary!(...),
    "sub": {
        "name": "mrxiaozhuox",
        "age": 18
    }
}
```

You can nest any data type inside, including nesting a new dictionary.

> This type can be operated and modified using the `Edit` command.

## Tuple

A tuple is an `array` that can only hold two pieces of data:

```
("LiuYuKun", 18)
("SongHaoXin": 18)
("XuGuanSen": 17)
("ZhouQiXiang": 17)
```

You can nest any data type inside, including nesting a new tuple.

## Data Style

The above data structures are displayed using `Doson`, which is a variant of `Json` with some added syntax, so it requires a special parser.

However, `Dorea` supports changing the type return style:

```
@default> value style json
[OK]: Successful
@default> get foo
[OK]: {"String": "bar"}
@default> get dict
[OK]: 
{
    "Dict": {
        "PI": {"Number": 3.14}, 
        "Bin": {"Binary": [...]}, 
        "Tuple": {"Tuple": [{...}, {...}]}
    }
}
```

Currently, `doson` parsers are only provided for `Rust` and `Python` languages. Other languages will use `Json` for parsing.

### Web-Service Switch Style

In `Web-Service`, to select `Value Style`, you just need to add a field in the `Form`:

```
form: {
    style: "json", // This field is used to change Style
    query: "....",
}
```
