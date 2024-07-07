FROM debian as tdlib-builder
RUN apt update && apt install git make cmake g++ libssl-dev zlib1g-dev gperf -y
RUN git clone https://github.com/tdlib/td
WORKDIR /td
RUN git checkout $TDLIB_COMMIT_HASH
WORKDIR /td/build
RUN cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX:PATH=../tdlib ..
RUN cmake --build . --target install

FROM rust as bot-builder
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
