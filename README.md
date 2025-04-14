# API For XAI Project

## Cargo Setup
Tokio's macros and rt-multi-thread features have to be enabled
``` 
cargo add tokio --features macros,rt-multi-thread
```

## PostgreSQL Setup
```
docker build -t app-postgres .
docker run -d --name app-db -p 5432:5432 app-postgres
```

## Format the code
```
cargo fmt
```