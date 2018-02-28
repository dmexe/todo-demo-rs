FROM registry.bookmate.services/base/rust:stable as builder

RUN mkdir -p /app
WORKDIR /app

ADD . /app

RUN cargo build --release && \
    cp ./target/release/todo-demo . && \
    mkchroot.sh /app/todo-demo

FROM scratch

COPY --from=builder /var/chroot /

ENV LD_LIBRARY_PATH="/usr/lib64/mysql" \
    RUST_LOG="info" \
    PORT="3000" \
    RUST_BACKTRACE="1"

WORKDIR /app
CMD ["./todo-demo"]
