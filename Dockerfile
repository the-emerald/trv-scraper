FROM rust:bullseye AS chef

RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin trv-scraper

FROM debian:bullseye-slim AS runtime
RUN USER=root apt-get update
RUN USER=root apt-get install -y libssl-dev ca-certificates

COPY --from=builder /app/target/release/trv-scraper .
ENV RUST_LOG=info

CMD ["./trv-scraper"]