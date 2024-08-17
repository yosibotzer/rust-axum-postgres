### Build service docker
```
docker build --progress=plain -t rust-axum-postgres .
```

### Start Postgres docker
```
docker run --rm --name postgres -e POSTGRES_USER=docker -e POSTGRES_PASSWORD=docker -e POSTGRES_DB=docker -p 5432:5432 -d postgres
```

### Start service docker
```
docker run --rm -d --network host --name my-rust-axum-postgres rust-axum-postgres
```

