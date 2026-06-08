# Installation & Deployment

> To do a good job, one must first sharpen one's tools.

## Cargo Installation

If you have a `Rust` development environment on your computer, you can install `Dorea` directly using `cargo`:

```shell
cargo install dorea
```

Or first clone the [repository](https://github.com/doreadb/dorea.git) and then install locally:

```shell
cargo install --path .
```

## Docker Setup

Build the image locally (need to `clone` the code first):

```shell
docker build -t dorea .
```

!> Docker-Hub URL: https://hub.docker.com/r/mrxiaozhuox/dorea

## Homebrew Installation

The latest version of `Dorea` has been published to `Homebrew`. You can install it directly using the following command:

```
brew install doreadb/brew/dorea
```

Using the above command will install the latest version of `Dorea` server and CLI tools.

## Binary Release Package

`Dorea` has deployed automatic compilation in `Github Action`. The system will automatically compile corresponding versions when `Release` is published:

[https://github.com/mrxiaozhuox/dorea/releases](https://github.com/mrxiaozhuox/dorea/releases)
