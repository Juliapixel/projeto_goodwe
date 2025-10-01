FROM caddy:latest
COPY Caddyfile /etc/caddy/Caddyfile
COPY frontend /var/www
