# Using this server
- Install Node.js and ensure you have `npx` command in your PATH.
- Copy `db.example.json` to `db.json`.
- Run `npx json-server -p 9800 db.json` to start the server.
- Run Moly.

Moly should be using `http://localhost:9800` by default. This is configurable in the
settings screen.


# Alternative to this server
If you don't want to spin a serder, you can run Moly setting the env var `MOLY_ARENA_FAKE`
to any value (even empty). This will use a file that is never modified as the data source.

Ex: `MOLY_ARENA_FAKE= cargo run`