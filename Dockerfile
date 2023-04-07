FROM rust:latest
RUN rustup target add wasm32-unknown-unknown

WORKDIR /usr/src/site
COPY . .

RUN cargo install trunk
RUN cd frontend && trunk build

RUN cargo build
ENTRYPOINT ["./target/debug/api_aga_in"]