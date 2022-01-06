# Tracker Proxy

![Build](https://github.com/gwy15/tracker-proxy/workflows/Build/badge.svg)

Tracker Proxy 是一个用来本地代理 BT/PT 的小程序。

## 使用方法

### 启动
```bash
tracker-proxy --help
tracker-proxy socks5h://127.0.0.1:1080
# 默认就是 8080，你也可以使用其他的端口号
tracker-proxy socks5h://127.0.0.1:1080 -p 8080  
```
### 在 BT 软件中修改种子的 tracker
```
https://ourbits.club/announce.php?passkey=233333333
=> 跟上面的端口号要一致，注意下面是 http
http://127.0.0.1:8080/ourbits.club/announce.php?passkey=233333333
```

### 修改 PT 站的 RSS 订阅链接

你也可以将 RSS 的订阅链接修改成上面的格式，tracker proxy 会 **自动处理 RSS 订阅内的下载链接并修改种子内的 tracker**。

> 但需要注意的是，RSS 的这种修改目前只支持基于 NexusPHP 的 PT 站。如果你有其他的 RSS 修改需求，欢迎你修改本项目。

```
https://ourbits.club/torrentrss.php
=>
http://127.0.0.1:8080/ourbits.club/torrentrss.php
```


## 编译

你可以前往 [release](https://github.com/gwy15/tracker-proxy/releases) 或者 [GitHub Actions](https://github.com/gwy15/tracker-proxy/actions) 下载编译好的镜像（提供 Windows 和 Linux 的），
或者自行编译。

```bash
git clone https://github.com/gwy15/tracker-proxy.git
cd tracker-proxy
cargo build --release
```

## FAQ

Q: 为什么没有日志？

A: 有日志，默认日志级别是 error。你可以修改环境变量 `RUST_LOG` 为 `debug`，`info` 或者 `warn` 来查看更详细的日志。
设置环境变量，在 powershell 中使用 `$env:RUST_LOG='debug'`.
