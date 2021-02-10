FROM rust:1.49 as builder

RUN USER=root cargo new --bin tunneload
WORKDIR ./tunneler
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/tunneload*
RUN cargo build --release

FROM debian:buster-slim
ARG APP=/usr/src/app

RUN mkdir -p ${APP}

COPY --from=builder /tunneler/target/release/tunneload ${APP}/tunneload

WORKDIR ${APP}

ENTRYPOINT ["./tunneload"]
