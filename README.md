# Tracker Proxy

![Build](https://github.com/gwy15/tracker-proxy/workflows/Build/badge.svg)

A local proxy for BitTorrent trackers.

## Usage

### Start the proxy
```bash
tracker-proxy socks5h://127.0.0.1:1080
tracker-proxy -p 8080 socks5h://127.0.0.1:1080
tracker-proxy --help
```
### Change tracker in BitTorrent client
```
https://ourbits.club/announce.php?passkey=233333333
=> 
http://127.0.0.1:8080/ourbits.club/announce.php?passkey=233333333
```

## Build

### Build your own
```bash
git clone https://github.com/gwy15/tracker-proxy.git
cd tracker-proxy
cargo build --release
```

### Download prebuilt binaries
Please go to [GitHub Actions](https://github.com/gwy15/tracker-proxy/actions) and look for workflow artifacts.
