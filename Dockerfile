FROM lukemathwalker/cargo-chef:latest as chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
COPY ./migration ./migration
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .
RUN apt update && apt install -y protobuf-compiler
RUN cargo build --release
RUN mv ./target/release/basic-rust ./app

FROM debian:stable AS runtime
WORKDIR /app
RUN apt update && apt install -y protobuf-compiler
COPY --from=builder /app/app /app/config.toml /app/
ENTRYPOINT ["./app"]
