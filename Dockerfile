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
RUN apt-get update && apt install -y protobuf-compiler libprotobuf-dev
RUN cargo build --release
RUN mv ./target/release/quotes-rs ./app

FROM debian:stable AS runtime
WORKDIR /app
RUN apt-get update && apt install -y openssl ca-certificates
COPY --from=builder /app/app /app/
ENTRYPOINT ["./app"]
