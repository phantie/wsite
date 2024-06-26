Start with 14$ 1vCPU 2Gb AMD Droplet 50Gb storage

Resize to 112$ 8vCPU 16Gb AMD Droplet 50Gb

Generate SSH key

    ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519 -N ''
    cat /root/.ssh/id_ed25519.pub


Add SSH key to Github

Get rep

    git clone git@github.com:phantie/wsite.git

> (Optional) Use the next command to skip until downgrade steps

    chmod +x ~/wsite/docs/preinstall.sh && ~/wsite/docs/preinstall.sh

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
    tar -xzf ~/wsite/frontend/trunk-x86_64-unknown-linux-gnu.tar.gz -C ~/wsite/frontend/


Build libs and bins

    cd /root/wsite/frontend/ && ~/wsite/frontend/trunk build --release &&
    cd /root/wsite/backend/ && cargo build --release

**Downgrade Droplet to the initial size**

Allow SSH connections
    
    ufw allow ssh

Enable firewall
    
    sudo ufw enable

Create DB

    touch "/root/db"

Start backend

Allow connections to backend server without TLS and domain (TEMP)

    ufw allow 8000

> (NOTE) Server IP
    
    curl -4 icanhazip.com

Check connection to http://{SERVER IP}:8000/

Modify A DNS records on DigitalOcean to point to {SERVER IP}

Check https://phantie.site and https://www.phantie.site (should not hang and immediately fail)

Setup nginx

    apt install nginx -y
    ufw allow 'Nginx Full'

Create file

    nano /etc/nginx/sites-available/phantie.site

With content:
```
server {
    listen 80;
    listen [::]:80;

    server_name phantie.site www.phantie.site;

    proxy_set_header X-Forwarded-For $remote_addr;

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
ln -s /etc/nginx/sites-available/phantie.site /etc/nginx/sites-enabled/
nginx -t
systemctl reload nginx
```

Setup TLS
<!--- apt-get purge nginx nginx-common nginx-full certbot python3-certbot-nginx -->
    apt install certbot python3-certbot-nginx -y
    certbot --nginx -d phantie.site -d www.phantie.site
    nginx -t
    systemctl restart nginx
    systemctl status nginx

Check https://phantie.site and https://www.phantie.site (should succeed)

Forbid this type of connection
    
    ufw delete allow 8000