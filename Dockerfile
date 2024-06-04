FROM debian as builder

RUN apt update && apt install git make cmake g++ libssl-dev zlib1g-dev gperf -y

RUN git clone https://github.com/tdlib/td
WORKDIR /td
RUN git checkout $TDLIB_COMMIT_HASH
WORKDIR /td/build
RUN cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX:PATH=../tdlib ..
RUN cmake --build . --target install

FROM rust

COPY --from=builder /td/tdlib/lib /usr/local/lib
RUN ldconfig

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src src
COPY .cargo .cargo
RUN cargo install --path .

CMD ["craiyon-bot"]
