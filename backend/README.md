# ShelfBrain Backend

Rust AWS SAM MVP API backed by DynamoDB.

## API

All MVP requests use a temporary `x-user-id` header. Replace this with Cognito or another authorizer when account auth is added.

- `GET /health`
- `GET /shelves`
- `GET /items?shelf=top|middle|bottom&status=active|archived&category=...`
- `POST /items`
- `GET /items/{id}`
- `PATCH /items/{id}`
- `POST /items/{id}/move`
- `POST /items/{id}/archive`
- `GET /categories`
- `POST /categories`
- `GET /notification-preferences`
- `PUT /notification-preferences/{shelf_type}`

## Build

```sh
sam build
```

The SAM template uses `BuildMethod: rust-cargolambda`, so install `cargo-lambda` if your SAM environment does not already have it:

```sh
cargo install cargo-lambda
```

## Deploy

```sh
sam deploy --guided
```

After deployment, set the mobile app API base URL with:

```sh
EXPO_PUBLIC_SHELF_BRAIN_API_URL=https://your-api-id.execute-api.region.amazonaws.com
```
