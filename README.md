# 🦊 Ladefuchs - API

[mit-url]: https://github.com/Team-Ladefuchs/ladefuchs-api/blob/main/LICENSE
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[openapi-url]: https://api.ladefuchs.app/docs/

[![MIT licensed][mit-badge]][mit-url]
[![Documentation](https://img.shields.io/badge/docs-OpenAPI-green)][openapi-url]
[![made-with-rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)
[![Docker-Build](https://github.com/Team-Ladefuchs/ladefuchs-api/actions/workflows/docker-publish.yml/badge.svg)](https://github.com/Team-Ladefuchs/ladefuchs-api/actions/workflows/docker-publish.yml)

Documentation to access the Ladefuchs API. If you are an App Developer, please read the [OpenAPI][openapi-url] documentation.

## API Documentation

-   Production: [api.ladefuchs.app/docs][openapi-url]
-   `./docs/openapi.yml`

## Endpoints

Production

```
https://api.ladefuchs.app
```

## Configuration

**Environment variables** are used for the entire configuration.
For development we recommend [direnv](https://direnv.net/).

| Name                 | Type    | Default                       | Values                                                          |
| -------------------- | ------- | ----------------------------- | --------------------------------------------------------------- |
| DATABASE_URL         | URI     | None                          | postgres://adminfuchs:ringdingdingding@localhost:5432/ladefuchs |
| DATABASE_POOL_SIZE   | uint32  | 8                             | 0...4294967295                                                  |
| INTERVAL             | uint8   | 3                             | 0...255                                                         |
| LISTEN               | ipAddr  | 127.0.0.1                     | 0.0.0.0                                                         |
| LOG                  | string  | INFO                          | TRACE \| DEBUG \| INFO \| WARN \| ERROR                         |
| PORT                 | uint16  | 3000                          | 0...65535                                                       |
| CHARGE_PRICE_API_URL | URI     | <https://api.chargeprice.app> |                                                                 |
| CHARGE_PRICE_API_KEY | string  | ""                            |                                                                 |
| DOMAIN               | uri     | <http://127.0.0.1:3000>       | localhost:3000                                                  |
| ADMIN_DOMAIN         | uri     | <http://127.0.0.1:8080>       | 127.0.0.1:8080                                                  |
| ADMIN_USER           | string  | None                          | admin_user                                                      |
| ADMIN_PWD            | string  | None                          | admin_pwd                                                       |
| SLACK_TOKEN          | string  | ""                            |                                                                 |
| SLACK_CHANNEL        | string  | ""                            |                                                                 |
| REPLICATION          | boolean | false                         | true \| false                                                   |

### Example

```sh
export DATABASE_URL=postgres://ladeuser:secret@localhost:54321/ladefuchs


export CHARGE_PRICE_API_KEY=42xxxxxxxxxxx
export CHARGE_PRICE_API_URL=https://api.chargeprice.app

# default is 3h
export INTERVAL = 3
# default
export PORT 3000
# default
export LISTEN=127.0.0.1
# default
export LOG=DEBUG
# Domain for the images
export DOMAIN="https://api.ladefuchs.app"
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

-   Development: `docker-compose.dev.yml`
-   Production: `docker-compose.yml`
