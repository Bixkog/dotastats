# Dotastats

Dotastats is a dota 2 guild stats provider. Its powered by Rust, with Rocket for HTTP API and MongoDB database. It is used as a backend for the dota2stats.ml.

## How to run

Server requires running mongoDB, config.json fields: "mongodb_host" and "mongodb_port" should be filled with mongoDB server address. You should also provide file containing database user credentials (by default mongodb_user.json). E.g.
```json
{
    "username": "dotastats_user",
    "password": "password"
}
```

You should also download lates dota 2 heroes constants, store it in file defined by the "heroes_info_filename" config field.

```bash
wget https://raw.githubusercontent.com/odota/dotaconstants/master/build/heroes.json
```

After configurations you can just build it with cargo

```bash
cargo run
```
