Installation when app and database live on the same server

Start with 14$ 1vCPU 2Gb AMD Droplet 50Gb storage

Resize to 112$ 8vCPU 16Gb AMD Droplet 50Gb

Generate SSH key

    ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519 -N ''
    cat /root/.ssh/id_ed25519.pub


Add SSH key to Github

> (Optional) Use ./preinstall.sh to skip until downgrade steps

Get rep

    git clone git@github.com:phantie/wsite.git

Install rust

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source "$HOME/.cargo/env"


Update system

    apt-get remove needrestart -y &&
    apt update &&
    apt install net-tools build-essential -y
    apt-get install pkg-config libssl-dev -y


Install trunk

    rustup target add wasm32-unknown-unknown
    cargo install --locked trunk


Build libs and bins

    cd /root/wsite/database/cli/ && cargo build --release &&
    cd /root/wsite/database/http_server/ && cargo build --release &&
    cd /root/wsite/frontend/ && trunk build --release &&
    cd /root/wsite/backend/ && cargo build --release

**Downgrade Droplet to the initial size**

Allow SSH connections
    
    ufw allow ssh

Enable firewall
    
    sudo ufw enable

Allow Db public server info port
    
    ufw allow 4000

Allow Db server port

    ufw allow 5645

Start DB

    cd /root/wsite/database/http_server/ && cargo build --release && setsid nohup cargo run --release -- --start &>server.log &

Set DB password
    
    cd /root/wsite/database/cli/ && cargo run --release -- db admin password

Create Admin dashboard password
    
    cd /root/wsite/database/cli/ && cargo run --release -- dashboard admin replace

Start backend

Allow connections to backend server without TLS and domain (TEMP)

    ufw allow 8000

> (NOTE) Server IP
    
    curl -4 icanhazip.com

Check connection to http://{SERVER IP}:8000/

Modify A DNS records on DigitalOcean to point to {SERVER IP}

Check https://phantie.com and https://www.phantie.com (should not hang and immediately fail)

Setup nginx

    apt install nginx -y
    ufw allow 'Nginx Full'

Create file

    nano /etc/nginx/sites-available/phantie.com

With content:
```
server {
    listen 80;
    listen [::]:80;

    server_name phantie.com www.phantie.com;

    location / {
        proxy_pass http://127.0.0.1:8000;
    }

    location /ws/ {
        proxy_pass http://127.0.0.1:8000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

```
ln -s /etc/nginx/sites-available/phantie.com /etc/nginx/sites-enabled/
nginx -t
systemctl reload nginx
```

Setup TLS
<!--- apt-get purge nginx nginx-common nginx-full certbot python3-certbot-nginx -->
    apt install certbot python3-certbot-nginx -y
    certbot --nginx -d phantie.com -d www.phantie.com
    nginx -t
    systemctl restart nginx

Check https://phantie.com and https://www.phantie.com (should succeed)

Forbid this type of connection
    
    ufw delete allow 8000