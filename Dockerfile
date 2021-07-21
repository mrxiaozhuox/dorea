# use rust(latest version)
FROM rust:latest

LABEL MAINTAINER="ZhuoEr Liu <mrxzx@qq.com>"

# dorea-core work dir
WORKDIR /usr/src/dorea-core

# copy environment file to ".cargo"
COPY env/ .cargo/
# copy project to "dorea-core"
COPY . .

# try to install cargo package: (dorea-core)
RUN cargo install --path .

# expose port: 3450 (dorea-port)
EXPOSE 3450

# volume dorea storage dir (data and config info)
VOLUME /root/.local/share/Dorea

# try to start the dorea server ( 0.0.0.0:3450 )
CMD ["dorea-server", "--hostname", "0.0.0.0"]