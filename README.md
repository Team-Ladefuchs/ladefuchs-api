# Ladefuchs

## API Beta Endpoint

```
https://beta.api.ladefuchs.app
```

## API Documentation

We have an ```openapi.yml``` file or go to [beta.api.ladefuchs.app/docs](https://beta.api.ladefuchs.app/docs/).

## Compile & Run

**Important**: [sqlx-cli](https://crates.io/crates/sqlx-cli) requires a working database schema. Otherwise, it will throw a bunch of compile errors for missing relations.

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

## Configuration

For development consider using [direnv](https://direnv.net/)

| Name                 | Type   | Default   | Values         |
|----------------------|--------|-----------|----------------|
| DATABASE_URL         | URI    |           |                |
| AUTH_TOKEN           | string |           |                |
| INTERVAL_V           | uint8  | 6         | 0...255        |
| LISTEN               | ipAddr | 127.0.0.1 |                |
| PORT                 | uint16 | 3000      | 0...65535      |
| LOG_TYPE             | enum   | Normal    | Normal \| Json |
| CHARGE_PRICE_API_URL | URI | <https://api.chargeprice.app/v1/charge_prices>          |                |
| CHARGE_PRICE_API_KEY | string |           |                |



### example
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
