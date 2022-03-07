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

```sh
sqlx migrate run 
# or 
sqlx migrate revert 
```
