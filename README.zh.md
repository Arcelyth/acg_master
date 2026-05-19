# ACGMaster

<p align="center">
  <a href="./README.md">中文</a> |
  <span>English</span>
</p>

## 介绍
一个使用 leptos 和 actix 搭建的关于ACG的猜谜游戏网站, 

[示例网站](https://acgmaster.com)
## 安装

生成会话签名密钥：
```bash
openssl rand -hex 64
```

创建一个`.env`文件：
```.env
APP_ENV=production
REDIS_EXPOSE_PORT=
SESSION_SIGNING_KEY=<YOUR_KEY>
DOCKER_PATH=<YOUR_DOCKER_DATA_PATH>
FRONTEND_URL=http://localhost:8080
HOST=localhost
PORT=8066
```

如有需要修改 `nginx.conf` 里的配置

### 直接运行Docker Compose

如果你的服务器配置足够好的话可以直接在服务器上运行`docker-compose up -d`

### 手动构建并传输镜像
先在本地构建镜像：
```bash 
docker buildx build --platform linux/amd64 -t acg-master_app:latest --load .
docker save -o acg-master_app.tar acg-master_app:latest
```

将生成的镜像文件复制到服务器后，执行：
```bash
docker load -i acg-master_app.tar
cd <src_path>
docker-compose up -d
```



