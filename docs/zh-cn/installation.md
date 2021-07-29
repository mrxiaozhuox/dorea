# 安装 & 部署

> 工欲善其事，必先利其器。

## Cargo 安装

如果你的电脑上拥有 `rust` 的开发环境，那你可以直接使用 `cargo` 来安装 `Dorea`

```shell
cargo install dorea
```

或者先将 [仓库](https://github.com/doreadb/dorea.git) Clone 下来，再进行本地安装：

```shell
cargo install --path .
```

## Docker 搭建

在本地构造镜像（需先 `clone` 代码）：

```shell
docker build -t dorea .
```

!> 由于最新 `Release` 版本还未发布，暂未 `Pull` 到 `Docker Hub`