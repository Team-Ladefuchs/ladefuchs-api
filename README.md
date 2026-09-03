# 🦊 Ladefuchs - API

[mit-url]: https://github.com/Team-Ladefuchs/ladefuchs-api/blob/main/LICENSE
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[openapi-url]: https://admin.ladefuchs.app/documentation

[![MIT licensed][mit-badge]][mit-url]
[![Documentation](https://img.shields.io/badge/docs-OpenAPI-green)][openapi-url]
[![made-with-rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)
[![Docker-Build](https://github.com/Team-Ladefuchs/ladefuchs-api/actions/workflows/docker-publish.yml/badge.svg)](https://github.com/Team-Ladefuchs/ladefuchs-api/actions/workflows/docker-publish.yml)

Documentation to access the Ladefuchs API. If you are an App Developer, please read the [OpenAPI][openapi-url] documentation.

## API Documentation

- Production: [admin.ladefuchs.app/documentation][openapi-url]
- `./docs/openapi.yml`

## Endpoints

Production

```
https://api.ladefuchs.app
```

## Configuration

**Environment variables** are used for the entire configuration.
For development we recommend [direnv](https://direnv.net/).

> [!IMPORTANT]
> Do not forget to set the JWT_KEY for the admin auth.

| Name                 | Type   | Default                 | Values                                                          |
| -------------------- | ------ | ----------------------- | --------------------------------------------------------------- |
| DATABASE_URL         | URI    | None                    | postgres://adminfuchs:ringdingdingding@localhost:5432/ladefuchs |
| DATABASE_POOL_SIZE   | uint32 | 8                       | 0...4294967295                                                  |
| LISTEN               | ipAddr | 127.0.0.1               | 0.0.0.0                                                         |
| LOG                  | string | INFO                    | TRACE \| DEBUG \| INFO \| WARN \| ERROR                         |
| PORT                 | uint16 | 3000                    | 0...65535                                                       |
| ECO_MOVEMENT_API_KEY | string | ""                      |                                                                 |
| DOMAIN               | uri    | <http://127.0.0.1:3000> | localhost:3000                                                  |
| ADMIN_DOMAIN         | uri    | <http://127.0.0.1:8080> | 127.0.0.1:8080                                                  |
| ADMIN_USER           | string | None                    | admin_user                                                      |
| ADMIN_PWD            | string | None                    | admin_pwd                                                       |
| JWT_KEY              | string | None                    |                                                                 |
| SLACK_TOKEN          | string | ""                      |                                                                 |
| SLACK_CHANNEL        | string | ""                      |                                                                 |
| CRON_SCHEDULE        | string | "0 45 23 \* \* \*"      | cron expression when to start the import                        |
| DOCS_DIR             | Path   | .docs                   | valid path                                                      |

### Slack

If none Slack channel and none token was provided, the slack bot will be disabled for that instance. If you do want slack messages, do not forget to add the [RoboFuchs Bot](https://ladefuchs.slack.com/apps/A03KBQ15FRS-robofuchs?settings=1&tab=settings) into the selected channel.

(note improve config documentation)

## Local Development

### Start Database

We have a prepared docker-compose file `docker-compose.dev.yml` you can use to spin up a database PostgreSQL Database instance. Every [configuration](#Configuration) is passed via an environment variable.

You can use this command:

```sh
sudo -E docker-compose -f ./docker-compose/docker-compose.yml up
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

### Run tests

To run the tests there is a small helper script. It sources the `config.env` and enables the right features. To run the
tests simply execute:

```sh
./scripts/run-tests.sh
```

## Monitoring

[uptime.ladefuchs.app/status/api](https://uptime.ladefuchs.app/status/api)

## Docker

### Build Image

```sh
sudo docker build -t ladefuchs .
```

##
