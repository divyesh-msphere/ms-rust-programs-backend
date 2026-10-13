# rust-service

`rust-service` is a Rust web service template built around clean architecture. It keeps business logic independent from delivery mechanisms and infrastructure, so HTTP, gRPC, database code, DTOs, and domain logic stay in their own layers.

The current implementation exposes an Actix Web HTTP server and includes a sample user flow:

```http
GET /api/v1/sample-user/{id}
```

The service loads configuration from `src/config/config.toml`, initializes telemetry through `ms-telemetry-rs`, builds application dependencies in `src/lib.rs`, and starts the HTTP server from `src/main.rs`.

## Tech Stack

- Rust 2024 edition
- Actix Web for HTTP APIs
- Tonic structure prepared for gRPC APIs
- SQLx for PostgreSQL access
- Serde for request and response DTOs
- Tracing and `ms-telemetry-rs` for telemetry

## Project Structure

```text
src/
  main.rs                         # Application entry point
  lib.rs                          # Dependency wiring
  config/                         # Application and telemetry config
  domain/                         # Enterprise/business entities
  application/
    usecases/                     # Business use cases
    repositories/                 # Repository traits owned by the application layer
    mappers/                      # Shared mapper traits
    errors/                       # Application-level error type
  adapters/
    inputs/
      http_apis/                  # HTTP controllers, routes, request mappers
      grpc_apis/                  # gRPC API structure
    services/
      postgres/                   # PostgreSQL repository implementations and DB models
  dto/
    payloads/                     # Input DTOs for HTTP/gRPC
    presenters/                   # Output DTOs for HTTP/gRPC
  infrastructure/                 # Server startup and framework integration
```

## Clean Architecture Flow

This project follows this dependency direction:

```text
HTTP/gRPC Controller
  -> DTO Payload
  -> Mapper
  -> Application Use Case
  -> Repository Trait
  -> Repository Implementation
  -> Database
  -> Domain Entity
  -> Use Case Output
  -> Mapper
  -> Presenter DTO
  -> HTTP/gRPC Response
```

Important rule: inner layers should not depend on outer layers.

- `domain` should not know about Actix, SQLx, DTOs, or config.
- `application` owns use cases and repository traits.
- `adapters` translate between external systems and application contracts.
- `infrastructure` starts framework-specific servers and wires shared services.

## Configuration

Default configuration lives in:

```text
src/config/config.toml
```

Current values include:

```toml
app_name = "rust-service"
http_port = 8080
grpc_port = 50051

[postgres]
host = "localhost"
port = 5432
database = "test_db"
username = "postgres"
password = "12341234"
max_connections = 10
```

Any config value can be overridden at runtime by environment variables passed on the command line (`VAR=value cargo run …`). Nested fields use `__` as the separator; a single `_` is preserved, because field names like `http_port` and `max_connections` already contain one.

`ApplicationConfig` loads `Environment::with_prefix("APP").prefix_separator("__").separator("__").try_parsing(true)`, so application config uses the `APP__` prefix:

```bash
APP__HTTP_PORT=9090 \                    # http_port
APP__GRPC_PORT=50052 \                   # grpc_port
APP__POSTGRES__HOST=db.internal \        # postgres.host
APP__POSTGRES__PORT=5432 \               # postgres.port
APP__POSTGRES__DATABASE=prod \           # postgres.database
APP__POSTGRES__MAX_CONNECTIONS=20 \      # postgres.max_connections
cargo run --bin rust_service
```

OpenTelemetry config (loaded by `ms-telemetry-rs`) uses the `TELEMETRY__` prefix with the same `__` nesting separator:

```bash
TELEMETRY__SERVICE_NAME=my_service                   # service_name
TELEMETRY__OTEL__ENDPOINT=http://host:4317           # otel.endpoint
TELEMETRY__OTEL__LOGS__LEVEL=debug                   # otel.logs.level
TELEMETRY__NATS__NKEY__SEED=...                      # nats.nkey.seed
```

Notes:

- Single `_` stays within a field name (`http_port` → `APP__HTTP_PORT`); `__` only marks a nested level (`postgres.host` → `APP__POSTGRES__HOST`).
- `try_parsing(true)` converts string env values into the field's type — required for numeric fields like the ports and `max_connections`.
- Environment sources are added **after** `config.toml`, so these overrides take precedence over the file.

## Run Locally

Make sure PostgreSQL is running and the configured database exists.

```bash
cargo run
```

The HTTP server listens on the configured `http_port`, currently `8080`.

Example request:

```bash
curl http://localhost:8080/api/v1/sample-user/<uuid>
```

Note: this repository depends on a private Git dependency:

```toml
ms-telemetry-rs = { git = "ssh://git@github.com/Mobilesentrix/ms-telemetry-rs.git", tag = "v0.1.0" }
```

Your SSH access to that repository must be configured before `cargo build` or `cargo run` can fetch dependencies.

## Servers And Binaries

The service now exposes the same use cases over **HTTP (Actix Web)** and **gRPC (Tonic)**. Three binaries are defined in `Cargo.toml`, all sharing the same library crate:

| Binary              | Source             | Runs                                                  |
| ------------------- | ------------------ | ----------------------------------------------------- |
| `rust_service`      | `src/main.rs`      | HTTP + gRPC together (one process, graceful shutdown) |
| `rust_service_http` | `src/main_http.rs` | HTTP only                                             |
| `rust_service_grpc` | `src/main_grpc.rs` | gRPC only                                             |

Because there are multiple binaries, select one with `--bin`:

```bash
cargo run --bin rust_service        # HTTP + gRPC
cargo run --bin rust_service_http   # HTTP only  (http_port, default 8080)
cargo run --bin rust_service_grpc   # gRPC only  (grpc_port, default 50051)
```

Sample endpoints:

```text
HTTP:  GET /api/v1/sample-users/{id}
gRPC:  sample_user_service.v1.SampleUserService/GetSampleUserById
```

## gRPC API

The gRPC server is implemented with Tonic and bootstrapped in `src/infrastructure/grpc_server.rs`. The service implementation lives in `src/adapters/inputs/grpc_apis/sample_user/` and maps proto requests into the **same use cases** the HTTP controllers call, mapping `ApplicationError` into `tonic::Status` via `GrpcErrorResponse`.

Generated proto code is committed under:

```text
src/adapters/generated/grpc_services/
```

It lives in the **adapter** layer (not `dto/`) because the generated bundle includes the service trait plus server/client — not just data — so it belongs with the adapters that implement and use it. Dependencies still point inward.

Test it (server reflection is not enabled, so point grpcurl at the proto from the central repo — note the `.v1` in the service name):

```bash
grpcurl -plaintext \
  -import-path ../ms-protos/proto \
  -proto rust-service-ms/v1/sample_user_service.proto \
  -d '{"id":"<uuid>"}' \
  localhost:50051 \
  sample_user_service.v1.SampleUserService/GetSampleUserById
```

## Protobuf And Code Generation (buf)

Proto contracts are owned by a central repository (`ms-protos`) and generated with **buf**. Configuration is in `buf.gen.yaml`:

- **Input:** `ssh://git@github.com/MobileSentrix/ms-protos.git`, `subdir: proto`, `branch: rust-service-test`, filtered to `paths: [rust-service-ms/v1]`.
- **Plugins → `src/adapters/generated/grpc_services`:**
  - `neoeinstein-prost` (remote) — message types
  - `neoeinstein-tonic` (remote) — service server + client
  - `protoc-gen-prost-crate` (local, `no_features`) — the aggregating `mod.rs` (run locally because, unlike remote plugins, it needs crate context)

Generated code is **committed**, so a normal `cargo build`/`cargo run` does **not** require buf — only regeneration does. To regenerate when the proto changes upstream:

```bash
# one-time: install the local plugin
cargo install protoc-gen-prost-crate

# from the repo root (so `out:` writes into src/)
buf generate
```

Prerequisites for regeneration: the [`buf`](https://buf.build) CLI, network access (the prost/tonic plugins run as remote buf plugins), and SSH access to the central proto repo. To pin a release instead of tracking a branch, replace `branch:` with `tag:` (or `ref:` for a commit) in `buf.gen.yaml`.

## Graceful Shutdown

Shutdown is centralized in `src/infrastructure/shutdown.rs`. `install_signal_handler()` spawns one listener for **Ctrl+C (SIGINT)** and **SIGTERM** and broadcasts via a `tokio::sync::watch` channel that each server observes:

- **HTTP (Actix):** built with `.disable_signals()` so Actix does not swallow the signal, then stopped gracefully via `ServerHandle::stop(true)`.
- **gRPC (Tonic):** uses `serve_with_shutdown(addr, ...)` driven by the same signal.

In the combined `rust_service` binary, one signal drains both servers so the whole process exits cleanly. Each standalone binary installs its own handler. Because `http_server` calls `disable_signals()`, every binary that runs it must pass a live shutdown receiver — which `install_signal_handler()` provides.

## How To Implement A New API

Use the existing sample user endpoint as the reference pattern. In this architecture, an API is not only a route and a controller. A complete API starts from the business concept, passes through a use case, and only then reaches HTTP, database, and DTO code.

The recommended order is:

```text
Entity
  -> Repository trait
  -> Use case
  -> DTOs
  -> Mapper
  -> Controller
  -> Route
  -> Database adapter
  -> Dependency wiring
```

This order matters because clean architecture protects the business rules from external details. HTTP paths, JSON field names, SQL table names, and framework types can change. The business entity and use case should remain stable.

### 1. Start With The Domain Entity

First identify the real business object your API is about. For example, before creating `GET /api/v1/order/{id}`, define what an `Order` means inside the application.

Create or update a file in:

```text
src/domain/
```

Example:

```text
src/domain/order_entity.rs
```

Then export it from:

```text
src/domain/mod.rs
```

Why this is necessary:

- The entity is the core model used by the application. Other layers should translate into or out of it.
- It gives the team one clear definition of the business object.
- It prevents database rows, HTTP request bodies, and response DTOs from becoming the application's source of truth.
- It keeps business code reusable if the same feature is later exposed through gRPC, a background job, or another interface.

Why we focus on the entity first:

If we start from the controller or database table, the business logic usually becomes shaped by delivery details. For example, an HTTP payload might contain strings because JSON has no UUID type, and a database model might contain optional fields because of joins or nullable columns. Those details are real, but they are not the business model.

The entity should answer questions like:

- What data does this concept need inside the system?
- Which fields are required for the business to work correctly?
- Are there any values that should never be allowed?
- Does this concept need helper methods or constructors?

Example:

```rust
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Order {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub status: String,
    pub total_amount: i64,
}

impl Order {
    pub fn new(id: Uuid, customer_id: Uuid, status: String, total_amount: i64) -> Self {
        Self {
            id,
            customer_id,
            status,
            total_amount,
        }
    }
}
```

Do not add Actix types, SQLx derives, JSON serialization rules, or request-specific fields here. Those belong to adapter and DTO layers.

### 2. Define The Repository Contract

After the entity is clear, define how the application wants to access that entity. This is a trait, not a PostgreSQL implementation.

Create a file in:

```text
src/application/repositories/
```

Example:

```text
src/application/repositories/order_repository.rs
```

Example contract:

```rust
use crate::application::errors::application_error::ApplicationError;
use crate::domain::order_entity::Order;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn get_by_id(&self, id: &Uuid) -> Result<Option<Order>, ApplicationError>;
}
```

Then export it from:

```text
src/application/repositories/mod.rs
```

Why this is necessary:

- The use case should depend on what it needs, not on how the data is stored.
- PostgreSQL, Redis, an external API, or an in-memory test implementation can all satisfy the same trait.
- It makes business logic easier to test because a fake repository can be passed into the use case.

What to keep in mind:

- Repository traits belong to the `application` layer because the application decides what data operations it needs.
- Return domain entities from repository traits, not database models.
- Use `ApplicationError` so errors are represented consistently across use cases.

### 3. Implement The Use Case

A use case represents one action the application can perform. Examples: `GetOrderById`, `CreateOrder`, `CancelOrder`, or `ListOrders`.

Create a use case under:

```text
src/application/usecases/<feature>/
```

Example:

```text
src/application/usecases/order/get_order_by_id.rs
```

The use case should define its own input and output structs:

```rust
pub struct GetOrderByIdInput {
    pub id: uuid::Uuid,
}

pub struct GetOrderByIdOutput {
    pub order: Order,
}
```

Then implement the common `UseCase` trait:

```rust
#[async_trait::async_trait]
impl UseCase<GetOrderByIdOutput, GetOrderByIdInput> for GetOrderByIdUseCase {
    async fn execute(
        &self,
        input: Option<GetOrderByIdInput>,
    ) -> Result<GetOrderByIdOutput, ApplicationError> {
        // validate input
        // call repository
        // apply business rules
        // return output
    }
}
```

Why this is necessary:

- The use case is where application-specific business flow lives.
- Controllers should stay thin and should not contain business decisions.
- Repository implementations should only fetch or persist data, not decide what the API means.
- The same use case can be called by HTTP, gRPC, tests, or other internal services.

What belongs in a use case:

- required input validation
- calling one or more repositories
- deciding what happens when data is missing
- applying business rules
- returning a clear success output
- returning `ApplicationError` for expected failure cases

What should not belong in a use case:

- Actix `HttpRequest` or `HttpResponse`
- SQL query strings
- JSON presenter structs
- route paths

Register the use case in:

```text
src/application/usecases/<feature>/mod.rs
src/application/usecases/mod.rs
```

This is similar to the current `SampleUserUseCases` structure.

### 4. Add DTO Payloads And Presenters

DTOs describe the external shape of the API. They are different from entities because API contracts often need different field names, formats, or validation behavior.

Input DTOs live in:

```text
src/dto/payloads/json/<feature>/
```

Output DTOs live in:

```text
src/dto/presenters/json/<feature>/
```

Example:

```text
src/dto/payloads/json/order/get_order_by_id.rs
src/dto/presenters/json/order/get_order_by_id.rs
```

Payloads usually derive `Deserialize`:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct GetOrderByIdPayload {
    pub id: String,
}
```

Presenters usually derive `Serialize`:

```rust
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct GetOrderByIdPresenter {
    pub id: String,
    pub customer_id: String,
    pub status: String,
    pub total_amount: i64,
}
```

Why this is necessary:

- DTOs keep the public API contract separate from the internal domain model.
- The API can expose strings, formatted dates, nested objects, or renamed fields without changing the entity.
- Sensitive internal fields can be hidden from the response.
- HTTP and gRPC can have different DTOs while sharing the same use case.

### 5. Add The HTTP API Mapper

The mapper connects the external API shape to the application shape.

Create a mapper in:

```text
src/adapters/inputs/http_apis/<feature>/<feature>_mappers.rs
```

The mapper converts:

```text
Payload DTO -> Use case input
Use case output -> Presenter DTO
```

Follow the existing mapper trait:

```rust
HttpApiMapper<UseCaseOutput, UseCaseInput, Presenter, Payload>
```

Why this is necessary:

- It keeps controllers simple.
- It keeps parsing and formatting out of use cases.
- It gives one obvious place to convert strings into UUIDs, domain values into response fields, and internal names into API names.

Example responsibility:

```rust
fn input(payload: GetOrderByIdPayload) -> GetOrderByIdInput {
    let id = uuid::Uuid::parse_str(&payload.id).unwrap_or_default();
    GetOrderByIdInput { id }
}
```

The use case can then decide whether a nil or invalid UUID should return a `400` error.

### 6. Add The HTTP Controller

The controller is the entry point for HTTP. It should be small because most work has already been separated into DTOs, mappers, use cases, and repositories.

Create or update:

```text
src/adapters/inputs/http_apis/<feature>/<feature>_controllers.rs
```

A controller should:

- receive path, query, or body data
- call the mapper to create use case input
- execute the use case from `AppState`
- map `ApplicationError` into `ErrorResponse`
- return the presenter DTO as JSON

Example shape:

```rust
#[get("/{id}")]
pub async fn get_order_by_id(
    data: web::Data<AppState>,
    path: web::Path<GetOrderByIdPayload>,
) -> Result<HttpResponse, ErrorResponse> {
    let payload = path.into_inner();
    let input = GetOrderByIdMapper::input(payload);

    let result = data
        .use_cases
        .order
        .get_order_by_id
        .execute(Some(input))
        .await
        .map_err(ErrorResponse::map_response_error)?;

    Ok(HttpResponse::Ok().json(GetOrderByIdMapper::output_success(result)))
}
```

Why this is necessary:

- Actix-specific code stays in the HTTP adapter layer.
- The controller coordinates the request but does not own business rules.
- Error conversion happens close to HTTP, where status codes and response bodies make sense.

### 7. Register The HTTP Route

Each feature should expose its own routes, then the shared route file attaches the feature under an API scope.

Expose feature-level routes from:

```text
src/adapters/inputs/http_apis/<feature>/mod.rs
```

Then register the API scope in:

```text
src/adapters/inputs/http_apis/shared/routes.rs
```

Example:

```rust
cfg.service(web::scope("/api/v1/order").configure(order_controllers::routes));
```

Why this is necessary:

- Route registration is the final HTTP wiring step.
- It keeps all top-level API prefixes visible in one shared place.
- It lets each feature own its own controller list without making the global route file too large.

### 8. Implement The PostgreSQL Adapter

Now add the actual database implementation. This comes after the entity, repository trait, and use case because the database is an outer detail. It should satisfy the application contract, not define it.

Add database-specific files under:

```text
src/adapters/services/postgres/<feature>/
```

Typical files are:

```text
<feature>_model.rs       # SQLx database row model
<feature>_mapper.rs      # DB model <-> domain entity mapper
<feature>_repository.rs  # Repository trait implementation
mod.rs
```

Why this is necessary:

- The SQLx model represents a database row, not the business entity.
- The mapper converts database-specific structure into the domain entity.
- The repository implementation contains SQL and database error mapping.
- If the storage engine changes later, the use case and controller should not need major changes.

The repository implementation should implement the trait from `application/repositories`:

```rust
#[async_trait::async_trait]
impl OrderRepository for OrderRepositoryPostgresImpl {
    async fn get_by_id(&self, id: &Uuid) -> Result<Option<Order>, ApplicationError> {
        // run SQL
        // map DB row into Order entity
    }
}
```

### 9. Wire The Repository Implementation

After creating the repository implementation, add it to the shared dependency container.

Update:

```text
src/application/repositories/shared.rs
```

Add a field to `Repositories` and initialize it with the concrete PostgreSQL implementation.

Then update use case wiring in:

```text
src/application/usecases/mod.rs
```

Why this is necessary:

- `Dependencies::load()` builds the real runtime object graph.
- Controllers access use cases through `AppState`.
- Use cases receive repository traits, but the application still needs one place that chooses the concrete implementation.

This repo currently wires dependencies in this flow:

```text
Dependencies::load()
  -> Repositories::new(...)
  -> UseCases::new(...)
  -> AppState
  -> Controller
```

### 10. Export New Modules

For every new folder or file, check the nearest `mod.rs` and export the module. Rust will not compile the new code until the module tree includes it.

Common files to update:

```text
src/domain/mod.rs
src/application/repositories/mod.rs
src/application/usecases/mod.rs
src/application/usecases/<feature>/mod.rs
src/dto/payloads/json/mod.rs
src/dto/payloads/json/<feature>/mod.rs
src/dto/presenters/json/mod.rs
src/dto/presenters/json/<feature>/mod.rs
src/adapters/inputs/http_apis/mod.rs
src/adapters/inputs/http_apis/<feature>/mod.rs
src/adapters/services/postgres/mod.rs
src/adapters/services/postgres/<feature>/mod.rs
```

## API Implementation Checklist

- Domain entity added
- Repository trait added in `application/repositories`
- Use case added and registered
- Payload DTO added
- Presenter DTO added
- HTTP mapper added
- HTTP controller added
- Route scope registered
- PostgreSQL model, mapper, and repository implementation added
- Repository implementation wired in `Repositories`
- New modules exported from `mod.rs`
- `cargo fmt` passes
- `cargo check` passes

## Development Commands

```bash
cargo fmt
cargo check
cargo run
```
# ms-rust-programs-backend
