# add ssh keys prior to the next steps
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y &&
source "$HOME/.cargo/env" &&
apt-get remove needrestart -y &&
apt update &&
apt install net-tools build-essential -y &&
apt-get install pkg-config libssl-dev -y &&
rustup target add wasm32-unknown-unknown &&
cargo install --locked trunk &&
cd /root/wsite/database/cli/ && cargo build --release &&
cd /root/wsite/database/http_server/ && cargo build --release &&
cd /root/wsite/frontend/ && trunk build --release &&
cd /root/wsite/backend/ && cargo build --release
