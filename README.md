# 📨 FerrumServe
## Rust TCP Chat Server 🦀💬

A simple, asynchronous TCP-based chat server built using Rust, Tokio, and SQLite for user account management.

## Features ✨

- [x] User registration and login 🔐
- [x] Broadcasting messages to all connected users 📡
- [x] List of all users with their online/offline status 👥
- [x] Colorful terminal output with Colored and PrettyTable crates 🌈
- [x] Asynchronous I/O with Tokio ⚡️

## Prerequisites 📚

- Rust (stable) and Cargo 🦀 </br>
- SQLite 🗄️

## Getting Started 🚀

### Starting the server ⏫

To start the server download FerrumServe from the release page

``` bash
$ ~/ferrum-serve 127.0.0.1:6142
```
## Building 📦
#### Clone the repository

``` bash

$ git clone https://github.com/yourusername/rust-tcp-chat-server.git
$ cd rust-tcp-chat-server
```

#### Build the project

``` bash

$ cargo build --release
```

#### Run the server

By default, the server will listen on 127.0.0.1:6142. You can provide an optional IP address and port as a command-line argument.

``` bash

$ ./target/release/rust-tcp-chat-server [IP:PORT]
```

## 📝 License

Copyright © 2023 [Simon Guglberger](https://github.com/sxmon17) and [Aleksa Nikolic](https://github.com/aaaleks07).</br>
</br>
