

Setup proxy to real server
==========================

 > socat TCP-LISTEN:8000,fork,reuseaddr,bind=localhost OPENSSL:elsa:8006,verify=0


 trunk serve --proxy-backend=http://localhost:8000/api2/
