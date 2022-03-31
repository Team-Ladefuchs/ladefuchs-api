# Ladefuchs

**Important**: [sqlx-cli](https://crates.io/crates/sqlx-cli) requires a working database schema. Otherwise, it will throw a bunch of compile errors for missing relations.

## Compile & Run

### Debug Build

```sh
cargo r
```

### Release Build

```sh
cargo r --release
```

## DB

### Start

```sh
./docker_db.sh
```

### Migration

You need to have [sqlx-cli](https://crates.io/crates/sqlx-cli) installed.

```sh
sqlx migrate run 
# or 
sqlx migrate revert 
```

### Config

For development consider using [direnv](https://direnv.net/)

example:

```sh
export DATABASE_URL=postgres://ladeuser:secret@localhost:54321/ladefuchs

export AUTH_TOKEN=911xxxxxxxxxxxxxxxx

export CHARGE_PRICE_API_KEY=42xxxxxxxxxxxxxx
export CHARGE_PRICE_API_URL=https://api.chargeprice.app/v1/charge_prices

# default is 6h
export INTERVAL_V = 6
# default
export PORT 3000
# default
export LISTEN=127.0.0.1
# default is Normal but Json is also supported 
export LOG_TYPE=Normal
```

(note improve config documentation)
