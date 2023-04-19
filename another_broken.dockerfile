# it does not work because:
#   1. can't find dist after all build
#   2. it's inconsistent, can return "wasm-bindgen call returned a bad status"
# if it's fixed: don't need to compile trunk each time, less CPU usage.
# disadvantage: still slow build, around seven minutes because of required platform linux/x86_64

# still the best way is to commit already built assets into rep, with disadvantage of growing the rep size
# P$EEN

FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM --platform=linux/x86_64 rust:1-slim as frontend_builder
RUN apt update && apt install -y curl wget pkg-config libssl-dev libpq-dev
RUN rustup target add wasm32-unknown-unknown
COPY . .
RUN tar -xzf frontend/trunk-x86_64-unknown-linux-gnu.tar.gz
# RUN mv ./trunk /usr/bin/
RUN cd frontend && ../trunk build
RUN ls .
RUN ls frontend

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .
COPY --from=frontend_builder frontend/dist frontend
# Build our project
RUN cargo build --release --bin api_aga_in

FROM debian:bullseye-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/api_aga_in api_aga_in
COPY --from=builder /app/backend/configuration backend/configuration
ENV APP_ENVIRONMENT production
# When `docker run` is executed, launch the binary
ENTRYPOINT ["./api_aga_in"]