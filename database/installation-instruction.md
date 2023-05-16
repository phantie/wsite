ssh-keygen -t ed25519 -C "@gmail.com"
cat /root/.ssh/id_ed25519.pub
git clone git@github.com:phantie/api_aga_in.git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
sudo apt install net-tools build-essential -y
sudo apt-get install pkg-config libssl-dev -y
sudo ufw allow ssh
sudo ufw allow 4000
sudo ufw allow 5645
sudo ufw enable
cd /api_aga_in/database/http_server/ && cargo run
