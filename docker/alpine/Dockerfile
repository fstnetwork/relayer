FROM alpine:edge AS builder

# show backtraces
ENV RUST_BACKTRACE 1

RUN apk add --no-cache \
  build-base \
  cargo \
  rust

WORKDIR /build
COPY . /build
RUN cargo build --release --target x86_64-alpine-linux-musl --verbose
RUN strip target/x86_64-alpine-linux-musl/release/fst-relayer


FROM alpine:edge

# show backtraces
ENV RUST_BACKTRACE 1

COPY --chown=root:root --from=builder /build/target/x86_64-alpine-linux-musl/release/fst-relayer /usr/bin/fst-relayer
RUN chmod a+x /usr/bin/fst-relayer

RUN useradd -ms /bin/bash relayer

USER relayer

EXPOSE 4982

WORKDIR /home/relayer

RUN mkdir -p /home/relayer/.local/share/network.fst.relayer/

ENTRYPOINT [ "fst-relayer", "version" ]
