# Ladefuchs

## API Beta Endpoint

```
https://beta.api.ladefuchs.app
```

## Live Endpoint

```
https://api.ladefuchs.app
```

## API Documentation

See:

* ```openapi.yml```
* Beta: [beta.api.ladefuchs.app/docs](https://beta.api.ladefuchs.app/docs/).
* Live: [api.ladefuchs.app/docs](https://api.ladefuchs.app/docs/).
## Configuration

**Environment variables** are used for the entire configuration.
For development we recommend [direnv](https://direnv.net/).

| Name                 | Type   | Default   | Values         |
|----------------------|--------|-----------|----------------|
| DATABASE_URL         | URI    |           |                |
| DATABASE_POOL_SIZE   | uint32 | 8         | 0...4294967295               |
| AUTH_TOKEN           | string |           |                |
| INTERVAL             | uint8  | 3         | 0...255        |
| LISTEN               | ipAddr | 127.0.0.1 |                |
| PORT                 | uint16 | 3000      | 0...65535      |
| LOG_TYPE             | enum   | Normal    | Normal \| Json |
| CHARGE_PRICE_API_URL | URI    | <https://api.chargeprice.app/v1/charge_prices>          |                |
| CHARGE_PRICE_API_KEY | string |           |                |
| IMAGE_PATH           | string |           | ./cards        |
| DOMAIN               | uri    |           | localhost      |
| SLACK_TOKEN          | string |     ""    |                |
| SLACK_CHANNEL        | string |     ""    |                |

### Example

```sh
export DATABASE_URL=postgres://ladeuser:secret@localhost:54321/ladefuchs

export AUTH_TOKEN=911xxxxxxxxxxxxxx

export CHARGE_PRICE_API_KEY=42xxxxxxxxxxx
export CHARGE_PRICE_API_URL=https://api.chargeprice.app/v1/charge_prices

# default is 3h
export INTERVAL = 3
# default
export PORT 3000
# default
export LISTEN=127.0.0.1
# default is Normal but Json is also supported 
export LOG_TYPE=Normal

export IMAGE_PATH="./cards"

export DOMAIN="https://example.com"
```

### Slack

If none Slack channel and none token was provided, the slack bot will be disabled for that instance. If you do want slack messages, do not forget to add the [RoboFuchs Bot](https://ladefuchs.slack.com/apps/A03KBQ15FRS-robofuchs?settings=1&tab=settings) into the selected channel.

(note improve config documentation)

## Locale Development

### Start Database

We have a prepared docker-compose file `docker-compose.dev.yml` you can use to spin up a database PostgreSQL Database instance. Every [configuration](#Configuration) is passed via an environment variable.

You can use this command:

```sh
sudo -E docker-compose -f docker-compose.dev.yml up
```

### Migration

You need to have [sqlx-cli](https://crates.io/crates/sqlx-cli) installed.

```sh
sqlx migrate run 
# or 
sqlx migrate revert 
```

### Compile & Run

Be sure that you set all necessary **environment variables** are set. Please take a look at the [configuration](#Configuration) section.

**SQL Errors**: In case of compile errors related to SQL queries, this could possible mean that [sqlx](https://crates.io/crates/sqlx) the used crate can't reach the database, to verify the correctness of every query. Thus, be sure to have a working database connection wit an initialized schema running.

Alternative you can set `SQLX_OFFLINE=true` so it will a least compile.

#### Debug Build

```sh
cargo r
```

#### Release Build

```sh
cargo r --release
```

## Docker

### Build Image

```sh
sudo docker build -t ladefuchs .   
```

### Docker Compose

* Development: `docker-compose.dev.yml`
* Production: `docker-compose.yml`
