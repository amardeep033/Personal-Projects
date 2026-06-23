# Pots of Gold

Two-player Pots of Gold game project with a React frontend and Rust backends.

## Structure

- `screen/` contains the React client.
- `server/` contains Rust services for the game API.
- `server/pots-of-gold-api/` contains the Shuttle deployment variant.

## Frontend

From `screen/`:

```bash
npm install
npm start
```

Useful scripts:

- `npm test`
- `npm run build`

## Backend

From `server/`:

```bash
cargo run
```

The local server exposes:

- `POST /api/start-game`
- `POST /api/optimal-move`

The Shuttle app lives in `server/pots-of-gold-api/`. If Shuttle CLI is installed, run it from that directory.