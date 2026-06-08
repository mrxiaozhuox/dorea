# build dorea-core latest version
# dorea: https://github.com/doreadb/dorea.git
# crate: https://crates.io/crates/dorea
# document: https://dorea.mrxzx.info
# author: ZhuoEr Liu <mrxzx@qq.com> [ https://blog.wwsg18.com ]

# dorea-server --hostname 0.0.0.0 --port 3450 --workspace .

# dorea key-value database

# use rust(latest version)
FROM rust:latest

# some information
LABEL MAINTAINER="ZhuoEr Liu <mrxzx@qq.com>"
ENV DOREA_VERSION="3.1"
ENV DOREA_WEBSITE="https://dorea.mrxzx.info"

# dorea-core work dir
WORKDIR /usr/src/dorea-core

# copy environment file to ".cargo"
COPY env/ .cargo/
# copy project to "dorea-core"
COPY . .

# try to install cargo package: (dorea-core)
RUN cargo install --path .

# expose port: 3450 (dorea-port) 3451 (dorea-service)
EXPOSE 3450
EXPOSE 3451

# volume dorea storage dir (data and config info)
VOLUME /root/.local/share/Dorea

# try to start the dorea server ( 0.0.0.0:3450 )
CMD ["dorea-server", "--hostname", "0.0.0.0"]