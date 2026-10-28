# Parked: Requirements, Approval steps, Applications, Signoffs

These four modules, and the two pages they belong to (**Requirements** and **Applications & Approvals**), are built and tested but
**switched off**. Nothing was deleted. One setting turns them back on, and a checklist at the bottom removes them for good.

| | Off (the default) | On |
|---|---|---|
| Setting | `features.enrollment_review = false` | `features.enrollment_review = true` |
| HTTP | the 20 paths do not exist (404) | served |
| gRPC | the 20 rpcs answer `UNIMPLEMENTED` | served |
| Module tree and manifests | the two pages are not in `GET /module-catalog`, not in any program's manifest, and cannot be selected (422) | present |
| `openapi.yaml` | does not list them | `openapi.enrollment-review.yaml` lists them |
| Postman | `programs.postman_collection.json` (the "Parked modules" folder proves they are off) | also run `enrollment-review.postman_collection.json` |

## What stays when they are off

- All code, the proto messages and rpcs, and the four tables (`requirements`, `approval_steps`, `applications`, `signoffs`) with their data.
  Nothing a program already did is lost.
- **Enrolled Stores** still works. A store is enrolled by hand (`POST /enrollments`, `:batchCreate`); with no applications, every
  enrollment has `application_id = null`.
- **Overview** still answers. The application counters (`total_applications`, `pending_approvals`) and the funnel read zero for a
  program that never had applications.
- **Publishing** still works. The bundle keeps its `requirements` and `approval_steps` keys, empty unless the program had data from
  before the switch was turned off.
- **Enrollment** in the module tree keeps only its Stores page, so the Create Program wizard shows one page under it, not three.

## How it works

1. `config.toml` has `[features] enrollment_review = false`. Override it with the environment variable
   `APP__FEATURES__ENROLLMENT_REVIEW=true`.
2. **HTTP**: [routes.rs](../src/adapters/inputs/http_apis/shared/routes.rs) registers the four modules' routes only when the setting is on.
3. **gRPC**: [programs_service_impl.rs](../src/adapters/inputs/grpc_apis/programs_service_impl.rs) checks the setting first in each of the 20 methods.
4. **The module tree**: at start-up [feature_switch.rs](../src/infrastructure/feature_switch.rs) moves the two pages' rows out of
   `module_definitions` and `program_modules` into `parked_module_definitions` and `parked_program_modules` (created by
   `migrations/20261027100000_parked_modules.up.sql`), or moves them back when the setting is on. It runs in one transaction under an
   advisory lock, and running it twice changes nothing. Because the rows really move, no query needed to change.
   Verified: parking and unparking leaves both tables byte-identical (compared by hash).

## Turn them on

```bash
# apply the migration once (it only creates the two holding tables)
APP__FEATURES__ENROLLMENT_REVIEW=true cargo run --bin programs_http     # or set features.enrollment_review = true in config.toml
```

The first start moves the pages back. Then run `postman/enrollment-review.postman_collection.json` and open the second spec in Swagger UI
(the drop-down at the top lists both). Turn them off again by removing the setting and restarting.

## Remove them for good

Do this only if you will never use them. Each step is a plain delete; `cargo build` after step 3 shows anything missed.

1. **Delete the module folders and files** (for each of `requirement`, `approval_step`, `application`, `signoff`):
   `src/domain/<m>.rs`, `src/adapters/services/postgres/<m>/`, `src/application/usecases/<m>/`,
   `src/adapters/inputs/http_apis/<m>/`, `src/adapters/inputs/grpc_apis/<m>/`, `src/dto/payloads/json/<m>.rs`, `src/dto/presenters/json/<m>.rs`.
2. **Remove the wiring**: the `pub mod <m>;` lines in the six `mod.rs` files (`domain`, `application/usecases`, `adapters/services/postgres`,
   `http_apis`, `grpc_apis`, `dto/payloads/json`, `dto/presenters/json`); the four repository fields and four use-case fields in
   `infrastructure/repositories.rs` and `application/usecases/mod.rs`; the four service fields and the 20 methods in `programs_service_impl.rs`;
   the `configure` block in `routes.rs`.
3. **Remove the shared code that mentions them**: `enroll_after_approval` and the `application_id` handling in
   `postgres/enrollment/enrollment_repository.rs`; the `requirements` and `approval_steps` parts of the bundle in
   `postgres/publication/publication_repository.rs`; the applications funnel and counters in
   `postgres/program_overview/program_overview_repository.rs`; `bump_stats` and the counters in `program_stats` if you want them gone.
4. **Remove the switch itself**: `feature_switch.rs` (its `ENROLLMENT_REVIEW_PAGES` and the call in `repositories.rs`), `FeaturesConfig`
   in `config/mod.rs`, the `[features]` block in `config.toml`.
5. **Proto**: delete the messages and rpcs of the four modules in `proto/programs/v1/`, then
   `buf generate proto --path proto/programs/v1` and `buf build proto --path proto/programs/v1 -o src/adapters/generated/grpc_services/programs.binpb`.
6. **Database**: add a migration that drops `signoffs`, `applications`, `approval_steps`, `requirements`, deletes the two pages from
   `parked_module_definitions` (and the matching `parked_program_modules`), and drops `enrollments.application_id`; add its down file.
7. **Docs and tests**: delete `openapi.enrollment-review.yaml`, `postman/enrollment-review.postman_collection.json`, the second entry in
   `docker-compose.yml` (`URLS` and its mount), the `openapi.enrollment-review.yaml#...` lines in `.spectral.yaml`, the "Parked modules"
   folder in the main Postman collection, and the Requirements, Applications & Approvals sections of the guides.

The shared database-error helper now lives in `postgres/db_error.rs`, so deleting the four modules does not break the other repositories.
