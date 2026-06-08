`Dorea` is a high-performance Key-Value storage system developed in `Rust`.

## Storage Model

`Dorea` uses a log-based storage solution: **Bitcask**

Log storage has only one operation: **Append**. All data processing is done by appending.

The three operations - Create, Delete, and Update - are all represented in this system as append operations.

The advantage of this storage method is that its write efficiency is extremely fast. It doesn't need to locate where data is stored or edit existing information. It simply inserts a new piece of information.

For data deletion, it only inserts a specific deletion marker for the lookup program to identify whether the information has been deleted.

### Index Positioning

In `Dorea`, the index of the currently used database is loaded directly into `memory`.

To explain simply:

This system supports multiple `Group` settings. It's similar to the 12 databases in `Redis`, but in `Dorea` you can create more databases.

When a database is loaded, it loads index information from local data (data volume under 100k+ won't have significant delay).

The system also supports developers to define `default load databases` and `maximum index volume` in the configuration file.

#### Index Preload

!> :ghost: This is a feature under development!

We will provide a special statement `preload` to initiate a preload request. The database system will spawn a separate coroutine to load this database.

Next time we use `select` to switch to this database, it will already be pre-processed (suitable for `groups` with large data volumes).

#### Index Structure

```rust
struct IndexInfo {
    file_id: u32,
    start_position: u64,
    end_position: u64,
    time_stamp: (i64, u64),
}
```

The above is the storage structure of `index information`, and its internal information is very concise.

- `file_id` - Storage file ID
- `start_position` - Start position of data in file
- `end_position` - End position of data in file
- `time_stamp` - Data generation time and valid time

Except for `time_stamp` which is used to optimize information expiration handling, all other fields are essential!

## Data Structure

`Dorea's data structure is relatively simple`, but it supports basic types and some composite types.

- String - Character string
- Number - Numeric value
- Boolean - Logical value
- Binary - Binary data
- List - Array
- Dict - Dictionary
- Tuple - Tuple

In `Dorea`, composite types can be nested recursively:

```text
List
[
    [1, 2, 3]
    [4, 5, 6]
    [7, 8, 9]
]
```

The specific content will be discussed in detail in the formal chapters.
