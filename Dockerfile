FROM rust:latest

LABEL MAINTAINER="ZhuoEr Liu <mrxzx@qq.com>"

WORKDIR /usr/src/doreadb

COPY env/ .cargo/
COPY . .

RUN cargo install --path .

EXPOSE 3450

VOLUME /root/.local/share/Dorea

CMD ["dorea-server", "--hostname", "0.0.0.0"]