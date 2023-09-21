Choose any droplet with ubuntu, any.

!!!. Compile for x86_64-unknown-linux-gnu and transfer binary and configuration there

Update system

    apt-get remove needrestart -y

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

    ln -s /etc/nginx/sites-available/phantie.com /etc/nginx/sites-enabled/
    nginx -t
    systemctl reload nginx

Setup TLS
    apt install certbot python3-certbot-nginx -y
    certbot --nginx -d phantie.com -d www.phantie.com
    nginx -t
    systemctl restart nginx

Verify no conficts

    systemctl status nginx

Check https://phantie.com and https://www.phantie.com (should succeed)

Forbid this type of connection
    
    ufw delete allow 8000