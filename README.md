# Ladefuchs

### Run

```sh
cargo r
```

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
export POSTGRES_USER=adminfuchs
export POSTGRES_PASSWORD=ringdingdingdingdingading
export POSTGRES_PORT=65430
export POSTGRES_DB=ladefuchs
export DATABASE_URL=postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@localhost:$POSTGRES_PORT/$POSTGRES_DB
export AUTH_TOKEN=52549df1xxxxxxxxxxxxxxxx

export CHARGE_PRICE_API_KEY=34xxxxxxxxxxxxxx
export CHARGE_PRICE_API_URL=https://api.chargeprice.app/v1/charge_prices

# default is 6h
export INTERVAL_V = 6
# default
export PORT 3000
```

(note improve config documentation)
