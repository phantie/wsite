Installation when database and live on the same server

/// SSH
ssh-keygen -t ed25519 -C "@gmail.com"
cat /root/.ssh/id_ed25519.pub
add ssh to github

/// Get rep
git clone git@github.com:phantie/api_aga_in.git

/// Install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

/// Update system
sudo apt install net-tools build-essential -y
sudo apt-get install pkg-config libssl-dev -y

/// Install trunk
rustup target add wasm32-unknown-unknown
cargo install --locked trunk

/// Allow SSH connections
sudo ufw allow ssh

// if there's app instances outside of the current server
// for example to access db from locally started app
/// Db public server info port
sudo ufw allow 4000
/// Db server port
sudo ufw allow 5645

/// App http port, no tls now
sudo ufw allow 80

/// Enable firewall
sudo ufw enable

