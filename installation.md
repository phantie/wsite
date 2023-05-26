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
sudo apt update
sudo apt install net-tools build-essential -y
sudo apt-get install pkg-config libssl-dev -y

/// Setup nginx
sudo apt install nginx -y
sudo ufw allow 'Nginx Full'

sudo nano /etc/nginx/sites-available/phantie.com

    server {
            listen 80;
            listen [::]:80;

            server_name phantie.com www.phantie.com;

            location / {
                    proxy_pass http://127.0.0.1:8000;
            }
    }

/// Setup TLS
sudo apt install certbot python3-certbot-nginx -y
sudo certbot --nginx -d phantie.com -d www.phantie.com

sudo ln -s /etc/nginx/sites-available/phantie.com /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx

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

/// Enable firewall
sudo ufw enable
