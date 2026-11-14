# terminal-pong

Pong in the terminal, written in Rust. Play the computer at three difficulty levels, or play a friend over the network through a server that owns the game and streams it to both players with Protocol Buffers over TCP.

```
Player 1 (you): 3                                                    Player 2: 2
----------------------------------- rally 4 ------------------------------------
                                        :
|
|                                       :           O
|
                                        :                                      |
                                                                               |
                                        :                                      |
--------------------------------------------------------------------------------
w/s or arrow keys to move, q to quit - computer: medium
```

## Build

Requires a recent stable Rust toolchain (tested with 1.78). No `protoc` install is needed.

```bash
cd terminal-pong
cargo build --release
```

Your terminal must be at least 80 columns by 24 rows.

## Play the computer

```bash
cargo run --release -- solo --difficulty hard
```

`--difficulty` is `easy`, `medium` (the default) or `hard`.

## Play a friend

One person hosts the server; both players connect as clients. The match starts when the second player joins, and the server goes back to waiting when a match ends.

```bash
cargo run --release -- server 0.0.0.0:7878
```

```bash
cargo run --release -- client <server-ip>:7878
```

The address defaults to `127.0.0.1:7878` for both commands, which is all you need to try it on one machine in three terminals.

## Controls

| Key | Action |
|---|---|
| `w` / `↑` / `k` | Move paddle up |
| `s` / `↓` / `j` | Move paddle down |
| `q` / `Esc` | Quit |

First to 10 points wins. Each point starts with a one-second pause, then the ball is served toward whoever lost the last point.

## How the ball behaves

Nothing about the ball is random.

- **Speed** starts at 26 cells per second, rises by 3 with every paddle hit in a rally, and is capped at 80. It resets on each serve.
- **Direction** depends only on where the ball strikes the paddle. Dead centre sends it back flat; the further toward an edge, the steeper the return, up to 50°. Hit with the top of your paddle to send it up, the bottom to send it down.
- The ball leaves the paddle at the same speed wherever it struck, so angled shots are not faster shots.
- A hit is judged at the exact row the ball crossed the paddle's column, not where it ended up at the end of the tick.

## Computer opponent

The computer is deterministic too. Difficulty changes how it plays, not how lucky it is:

| | Easy | Medium | Hard |
|---|---|---|---|
| Paddle speed (rows/s) | 7 | 8 | 9 |
| Starts moving when the ball is within | 50% of the field | 60% | 75% |
| Aims for | the ball's current row | where the ball will arrive, bounces included | where the ball will arrive, bounces included |
| Return | flat | slightly angled away from you | sharply angled away from you |
| Returns to the middle between shots | no | no | yes |

Against a simulated opponent that never misses and always hits toward the far corner, easy returns about 1 ball before losing the point, medium 5–7 and hard 14–22. Hard is beatable, but only by building a rally fast enough to out-pace its paddle.

## Hit and miss log

To check hit detection, every paddle hit and miss can be logged. The server always logs to stdout (or to a file with `--log FILE`); solo mode logs when given `--log FILE`.

```
[tick     76  0-0 ] HIT   p2 offset=+0.23 of paddle, ball leaves at 29 cells/s
[tick    158  0-1 ] MISS  p1 ball_y=12.10 paddle_y=14 (covers 12.5..15.5) missed by 0.40 rows  <-- NEAR MISS, check hit detection
Final score 0-1 after 354 ticks: 3 hits, 1 misses, 1 near misses
```

Misses within half a row of the paddle are flagged, since those are the ones that look like they should have been hits on screen.

## How it works

| Piece | File | What it does |
|---|---|---|
| Rules | `src/physics.rs` | Ball movement, wall bounces, paddle hits, scoring. Pure functions of game state, no I/O. |
| Computer | `src/ai.rs` | The three difficulty profiles. |
| Wire format | `src/net.rs`, `proto/message.proto` | Protobuf messages, each prefixed with its length as a 4-byte big-endian integer so messages can't be merged or split by TCP. |
| Server | `src/server.rs` | Seats two players, applies their paddle moves, runs the physics at 30 ticks per second and sends every frame to both. A client can move at most one row per message. |
| Client | `src/client.rs` | Sends key presses, draws the newest frame from the server. |
| Solo | `src/solo.rs` | Local game loop against the computer. |
| Drawing | `src/render.rs` | Full-frame redraw on an alternate screen with crossterm; the terminal is restored on exit, including on errors. |
| Debug log | `src/eventlog.rs` | The hit and miss log. |

## Tests

```bash
cargo test
```

Unit tests cover ball physics (angles, speed, hit box, wall bounces, scoring, landing prediction against the simulation), the AI (each difficulty outlasts the one below, reaction distance, paddle speed), message framing (glued, split, oversized and truncated frames) and rendering. An integration test starts the real server on a local port and plays it with two TCP clients.
