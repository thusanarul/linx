


## How to run
with `docker compose` from root folder `/linx`
```
docker compose up
```

with `docker` from root folder `/linx`
```
docker build -t linx -f ./api/Dockerfile ./api
docker run --rm -p 3000:3000 linx
```

with cargo from folder `/linx/api`
```
cargo run
```

## How to test
Either run the application following the steps above and do requests against `http://localhost:3000` or use `https://linx.arul.no`.

Example `curl` commands:
```
curl "http://localhost:3000/weather?date=2026-02-09T21:42:00%2B01:00"
curl "https://linx.arul.no/weather?date=2026-02-09T20:42:00Z"
curl "https://linx.arul.no/weather?date=2026-02-09"
```

Some info about the api is available on root path of server.
