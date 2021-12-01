# 运行本文件可以让 DoreaDB 直接运行在项目目录下的 .env 下，方便开发和调试。

cargo build --bin dorea-server && clear && ./target/debug/dorea-server --workspace ./.env

# DoreaDB 默认会存储到系统文档保存目录中，但是您也可以通过 --workspace 自定义它的运行路径（同时也方便单机运行多台数据库）