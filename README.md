# 📨 FerrumServe
## Rust TCP Chat Server 🦀💬

A simple, asynchronous TCP-based chat server built using Rust, Tokio, and SQLite for user account management.

## Features ✨

- [x] User registration and login 🔐
- [x] Broadcasting messages to all connected users 📡
- [x] List of all users with their online/offline status 👥
- [x] Colorful terminal output with Colored and PrettyTable crates 🌈
- [x] Asynchronous I/O with Tokio ⚡️
- [x] SQLite database for user account management 🗄️
- [x] Logging with log and env_logger 📝
- [x] Admin account with special privileges and commands 🛡️

## Commands 📜
- `/help` - List all available commands
- `/listusers` - List all users with their online/offline status and their role
- `/history [blank, username, all]` - Show msg history
- `/whisper <username> <message>` - Send a private message to a user
- `/changepw <old_password> <new_password>` - Change your password
- `/color <color_name>` - Change your username color
- `/admin <password>` - Become an admin
- `/ban <username>` - Ban a user
- `/unban <username>` - Unban a user
- `/mute <username>` - Mute a user
- `/unmute <username>` - Unmute a user
- `/quit` - Quit the chat


## Prerequisites 📚

- Rust (stable) and Cargo 🦀
- SQLite 🗄️
- A terminal emulator with 256-color support (e.g. iTerm2, Alacritty, Konsole) 🖥️

## Getting Started 🚀

### Starting the server ⏫

To start the server download FerrumServe from the release page

``` bash
$ ~/ferrum-serve 127.0.0.1:6142 admin1234
```

### Connecting to the server 📡

To connect to the server, use a telnet client (e.g. telnet, iTerm2, Alacritty, Konsole) and connect to the server's IP address and port.

``` bash
$ telnet 127.0.0.1:6142
```

## Building 📦
#### Clone the repository

``` bash
$ git clone https://github.com/FerrumLink/FerrumServe
$ cd FerrumServe
```

#### Build the project

``` bash
$ cargo build --release
```

#### Run the server

By default, the server will listen on 127.0.0.1:6142. You can provide an optional IP address and port as a command-line argument.

``` bash
$ ./target/release/ferrum-serve [IP:PORT]
```

## 📝 License

Copyright © 2023 [Simon Guglberger](https://github.com/sxmon17).</br>
</br>
