# requires /frontend/dist/ to be up to date with code
# until cost of bringing trunk to build pipeline become lower

FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .
# Build our project
RUN cargo build --release --bin api_aga_in

FROM debian:bullseye-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/api_aga_in api_aga_in
COPY --from=builder /app/backend/configuration backend/configuration
ENV APP_ENVIRONMENT production
# When `docker run` is executed, launch the binary
ENTRYPOINT ["./api_aga_in"]