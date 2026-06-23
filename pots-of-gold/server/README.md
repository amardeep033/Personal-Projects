# Pots of Gold Server

Rust backend for the Pots of Gold game.

## Structure

- `src/main.rs` is the local Actix Web server.
- `pots-of-gold-api/` is the Shuttle deployment project.

## Local Run

From this directory:

```bash
cargo run
```

Default API routes:

- `POST /api/start-game` returns randomized pot values.
- `POST /api/optimal-move` returns the computer move for the selected difficulty.

## Shuttle Variant

`pots-of-gold-api/` contains the hosted version of the same API with Shuttle-specific setup.