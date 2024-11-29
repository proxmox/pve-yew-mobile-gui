# Run with proxy to real server

Note: you need an TLS key (api.key) and certificate (api.pem).

trunk serve --proxy-backend=https://${SERVER}:8006/api2/ --proxy-insecure --tls-key-path api.key --tls-cert-path api.pem
