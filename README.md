# ACGMaster

<p align="center">
  <span>English</span> |
  <a href="./README.zh.md">中文</a>
</p>

## Installation

Generate a session signing key:
```bash
openssl rand -hex 64
```

Create a `.env` file: 
```.env
APP_ENV=production
REDIS_EXPOSE_PORT=
SESSION_SIGNING_KEY=<YOUR_KEY>
DOCKER_PATH=<YOUR_DOCKER_DATA_PATH>
FRONTEND_URL=http://localhost:8080
HOST=localhost
PORT=8066
```

Adjust `nginx.conf` if needed.

### Run directly with Docker Compose

Your can just run `docker-compose up -d ` on your server if your server's configuation is enough <br>

### Build and transfer the image manually

Build the image locally:
```bash 
docker buildx build --platform linux/amd64 -t acg-master_app:latest --load .
docker save -o acg-master_app.tar acg-master_app:latest
```

Copy the generated archive to your server, then load and run it:
```bash
docker load -i acg-master_app.tar
cd <src_path>
docker-compose up -d
```



