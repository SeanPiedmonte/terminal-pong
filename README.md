# terminal-pong

Pong in the terminal, written in Rust. You play the left paddle against a simple AI, and every frame of game state is serialised with Protocol Buffers and streamed over TCP to a companion server process.

The game itself is single-player today. The client/server split is the groundwork for networked two-player: the wire format and connection handling are in place, and the server currently just prints each state it receives.

```
|
|                                    O
|                                                                               |
                                                                                |
                                                                                |

SCORE: 3 : 2
```

## How it works

| Piece | File | What it does |
|---|---|---|
| Game loop | `terminal-pong/src/game.rs` | 80×20 playfield drawn with [crossterm](https://crates.io/crates/crossterm) in raw mode, ticking every 200 ms. Handles ball physics, scoring, and the AI paddle, which predicts where the ball will land (including wall bounces) and moves toward it. |
| Client | `terminal-pong/src/client_handler.rs` | Converts each frame's `GameState` into a protobuf `Game_State` message and writes it to a TCP stream. |
| Server | `terminal-pong/src/server.rs` | Accepts a connection, decodes each `Game_State` message and prints it. |
| Wire format | `terminal-pong/proto/message.proto` | Ball position and velocity, both paddle positions, both scores. |

Rust types for the protobuf message are generated at build time by `build.rs` using `protobuf-codegen`'s pure-Rust parser, so you do **not** need `protoc` installed.

## Build

Requires a recent stable Rust toolchain (tested with 1.78).

```bash
cd terminal-pong
cargo build --release
```

## Run

The client connects to the server on start-up, so start the server first.

Terminal 1:

```bash
cargo run --release -- server
```

Terminal 2:

```bash
cargo run --release -- client
```

Both default to `127.0.0.1:7878`. Pass a different `host:port` as the second argument to either command, for example `cargo run --release -- server 0.0.0.0:9000`.

To check the protobuf encoding without opening a game:

```bash
cargo run --release -- selftest
```

## Controls

| Key | Action |
|---|---|
| `w` | Move paddle up |
| `s` | Move paddle down |
| `Esc` | Quit |

First to 10 points wins. Your terminal needs to be at least 81 columns by 22 rows.

## Roadmap

- Have the server own the game state and relay it between two clients for real two-player games.
- Length-prefix messages on the wire so frames can't be merged by TCP.
- Unit tests for ball physics and the AI's landing prediction.
