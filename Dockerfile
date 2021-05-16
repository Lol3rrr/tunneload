FROM rust:1.52.1 as builder

RUN USER=root cargo new --bin tunneload
WORKDIR ./tunneload
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/tunneload*
RUN cargo build --release

FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update; apt-get upgrade -y; apt-get install libssl1.1

RUN mkdir -p ${APP}

COPY --from=builder /tunneload/target/release/tunneload ${APP}/tunneload

WORKDIR ${APP}

ENTRYPOINT ["./tunneload"]