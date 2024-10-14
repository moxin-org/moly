- Install Node.js and ensure you have `npx` command in your PATH.
- Copy `db.example.json` to `db.json`.
- Run `npx json-server -p 9800 db.json` to start the server.
- Run Moly with the env var `MOLY_BATTLE_API=http://localhost:9800`.
  - Ex: `MOLY_BATTLE_API=http://localhost:9800 cargo run`

Note: The env var is read at build/compile time.
