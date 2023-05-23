# NOTE seems it can send TCP, but not UDP

# requires /frontend/dist/ to be up to date with code
# until cost of bringing trunk to build pipeline become lower

FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app


# IT"S A LIE!!!! NO PACKETS ARRIVE TO 
# FROM debian:bullseye-slim AS ping_db
# RUN apt update
# RUN apt upgrade
# RUN apt install nmap -y
# ADD "https://www.random.org/cgi-bin/randbyte?nbytes=10&format=h" skipcache
# RUN nmap -sU -p 5645 209.38.192.88
# RUN touch blank

FROM ubuntu:latest AS ping_db
RUN apt-get update
RUN apt-get -y install netcat
ADD "https://www.random.org/cgi-bin/randbyte?nbytes=10&format=h" skipcache
RUN echo "some data\\" | timeout 10 netcat -t 209.38.192.88 5645 ; exit 0
ADD "https://www.random.org/cgi-bin/randbyte?nbytes=10&format=h" skipcache
RUN echo "some data\\" | timeout 10 netcat -u 209.38.192.88 5645 ; exit 0
ADD "https://www.random.org/cgi-bin/randbyte?nbytes=10&format=h" skipcache
RUN netcat -v -u -z -w 3 209.38.192.88 5645
RUN touch blank

RUN apt install net-tools
RUN netstat -tulpn


# FROM chef AS planner
# COPY . .
# # Compute a lock-like file for our project
# RUN cargo chef prepare --recipe-path recipe.json

# FROM chef AS builder
# COPY --from=planner /app/recipe.json recipe.json
# # Build our project dependencies, not our application!
# RUN cargo chef cook --recipe-path recipe.json
# # Up to this point, if our dependency tree stays the same,
# # all layers should be cached.
# COPY . .
# # Build our project
# RUN cargo build --bin api_aga_in

FROM ubuntu:latest AS runtime
# RUN apt update
# RUN apt upgrade

WORKDIR /app
# for step to run at all
COPY --from=ping_db blank blank
# COPY --from=builder /app/target/debug/api_aga_in api_aga_in
# COPY --from=builder /app/backend/configuration backend/configuration
# ENV APP_ENVIRONMENT production
# When `docker run` is executed, launch the binary
# ENTRYPOINT ["./api_aga_in"]
ENTRYPOINT ["ls ."]
