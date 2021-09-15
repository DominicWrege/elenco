## Getting Started
This is a simple podcast directory backend. I was written and developed by myself for my bachelor thesis.

### Start the database
```
docker-compose up -d
```

### Run in Development
```
cargo run
```
### Run in Release
```
cargo run --release
```


## Example Enviroment

```
export DB_USERNAME=usertest
export DB_PASSWORD=hundpwd
export DB_HOST=127.0.0.1
export DB_DATABASENAME=podcast
export DB_PORT=5432
# Cookie Option, default: localhost
export DOMAIN=elenco-podcast.com 
```