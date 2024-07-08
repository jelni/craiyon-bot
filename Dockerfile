FROM debian as tdlib-builder
RUN apt update && apt install git make cmake g++ libssl-dev zlib1g-dev gperf -y
RUN git clone https://github.com/tdlib/td
WORKDIR /td/build
RUN git checkout $TDLIB_COMMIT_HASH
RUN cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX:PATH=../tdlib ..
RUN cmake --build . --target install

FROM rust as bot-builder
COPY --from=tdlib-builder /td/tdlib/lib /usr/local/lib
RUN ldconfig
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src src
COPY .cargo .cargo
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=target \
    cargo build --release \
    && cp target/release/craiyon-bot craiyon-bot

FROM rust
COPY --from=tdlib-builder /td/tdlib/lib /usr/local/lib
RUN ldconfig
COPY --from=bot-builder /app/craiyon-bot /app/
WORKDIR /data
ENTRYPOINT ["/app/craiyon-bot"]
