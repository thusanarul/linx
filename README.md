

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
