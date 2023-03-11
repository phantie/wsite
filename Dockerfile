
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --recipe-path recipe.json
# Build application
# WORKDIR /app
COPY . .
RUN cargo build

# We do not need the Rust toolchain to run the binary!
FROM debian:bullseye-slim AS runtime
WORKDIR app
COPY --from=builder /app/target/debug/api_aga_in /usr/local/bin
WORKDIR /
COPY . .
ENV APP_ENVIRONMENT production
ENTRYPOINT ["/usr/local/bin/api_aga_in"]