# build dorea-core latest version
# dorea: https://github.com/doreadb/dorea.git
# crate: https://crates.io/crates/dorea
# document: https://dorea.mrxzx.info
# author: ZhuoEr Liu <mrxzx@qq.com> [ https://blog.wwsg18.com ]

# ==================== Build Stage ====================
FROM rust:1.82-bookworm AS builder

# some information
LABEL MAINTAINER="ZhuoEr Liu <mrxzx@qq.com>"
ARG DOREA_VERSION="0.4"

# dorea-core work dir
WORKDIR /usr/src/dorea-core

# 1. Copy cargo config first
COPY env/ .cargo/

# 2. Copy manifests for dependency caching
COPY Cargo.toml Cargo.lock ./

# 3. Create dummy main to cache dependencies
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/bin/server.rs && \
    echo "fn main() {}" > src/bin/cli.rs && \
    cargo build --release --no-default-features --features server && \
    rm -rf src

# 4. Copy actual source and build
COPY src/ src/
RUN touch src/bin/server.rs && \
    cargo build --release --no-default-features --features server

# ==================== Runtime Stage ====================
FROM debian:bookworm-slim

LABEL MAINTAINER="ZhuoEr Liu <mrxzx@qq.com>"
ENV DOREA_VERSION="0.4"
ENV DOREA_WEBSITE="https://dorea.mrxzx.info"

# Install runtime dependencies (ca-certificates for HTTPS if needed)
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    # Create non-root user
    useradd -r -s /bin/false dorea && \
    mkdir -p /data && \
    chown dorea:dorea /data

WORKDIR /app

# Copy binary from builder
COPY --from=builder /usr/src/dorea-core/target/release/dorea-server /usr/local/bin/

# Set ownership
RUN chown -R dorea:dorea /app

# Switch to non-root user
USER dorea

# expose port: 3450 (dorea-port)
EXPOSE 3450

# volume dorea storage dir
VOLUME /data

# Default workspace location
ENV DOREA_WORKSPACE=/data

# start the dorea server
CMD ["dorea-server", "--hostname", "0.0.0.0", "--port", "3450", "--workspace", "/data"]
