# Modules walkthrough: code, files, tables and columns

Read this top to bottom. Each section is one module: what it does, its endpoints, the files that
hold its code, the tables and columns it manages, and what happens on one request.

## 0. How every module is laid out

Every module has the same layers. A request goes down the list and the answer comes back up:

| Layer | Folder | What lives there |
|---|---|---|
| Route | `src/adapters/inputs/http_apis/<module>/<module>_controllers.rs` (HTTP), `src/adapters/inputs/grpc_apis/<module>/<module>_service_impl.rs` (gRPC) | Receives the call, builds the use-case input, returns the answer. No business rules. |
| Mappers | `.../<module>_mappers.rs` (both transports) | Turns request payloads into use-case input and domain objects into JSON / protobuf. |
| DTOs | `src/dto/payloads/json/<module>.rs` (request shapes), `src/dto/presenters/json/<module>.rs` (response shapes) | The JSON the outside world sees. |
| Use case | `src/application/usecases/<module>/<operation>.rs` | One file per operation. Checks the program exists, validates, calls the repository. |
| Domain | `src/domain/<module>.rs` | The types, the validation rules, the repository **trait** (an interface) and the unit tests. No SQL. |
| Postgres | `src/adapters/services/postgres/<module>/<module>_repository.rs` | The SQL. Implements the domain trait. |
| Contract | `proto/programs/v1/types.proto` and `programs.proto`, `openapi.yaml`, `postman/` | gRPC messages and methods, the HTTP spec, the Postman collection. |

Where the modules are wired together:
- HTTP routes: `src/adapters/inputs/http_apis/shared/routes.rs`
- gRPC routing: `src/adapters/inputs/grpc_apis/programs_service_impl.rs` (one `ProgramsService`, it fans out to each module)
- Repositories: `src/infrastructure/repositories.rs`; use cases: `src/application/usecases/mod.rs`
- Tables: `migrations/*.up.sql` (22 tables; the ones in use are listed below)

Three things every write does (shared helpers, so every module behaves the same):
1. **One transaction.** The change, the audit entry, the outbox event and any counter commit together or not at all.
2. **Audit entry** into `audit_log` (who, what, before and after). Helper: `record_change` / `record_event` in
   `src/adapters/services/postgres/program/program_repository.rs`.
3. **Outbox event** into `outbox` (for other services to pick up later). Same helper.

---

## 1. Module definitions (the catalog)

**What it is.** The read-only list of every module a program can have (Overview, Applications, Enrolled Stores, and so on),
arranged as a tree of groups and leaves. It is seeded by migrations and never changed through the API.

**Endpoints.** `GET /module-definitions` (list, also as a tree), `GET /module-definitions/{module_definition}`.
gRPC: `ListModuleDefinitions`, `GetModuleDefinition`.

**Files.**
- Domain: `src/domain/module_definition.rs`
- SQL: `src/adapters/services/postgres/module_definition/` (`module_definition_repository.rs`, `_mapper.rs`, `_model.rs`)
- Use cases: `src/application/usecases/module_definition/` (`list_module_definitions.rs`, `get_module_definition.rs`, `get_module_definition_tree.rs`)
- HTTP / gRPC: `http_apis/module_definition/`, `grpc_apis/module_definition/`
- Seeds: `migrations/20261014*`, `20261015*`, `20261016100000_seed_module_classification`

**Table `module_definitions`** (read only):

| Column | Meaning |
|---|---|
| `id` | UUID primary key |
| `key` | Unique name like `enrollment.applications_approvals.application_panel`. Programs refer to modules by this. |
| `name` | Label shown to the admin |
| `kind` | `group` (has children) or `leaf` (a real page) |
| `parent_id` | The parent group (tree) |
| `ancestry` | `ltree` path of the keys. Lets one query fetch a whole subtree. |
| `position` | Display order among siblings |
| `source` | `premade` (code-backed page the admin selects), `container` (admin decides what the leaf is) or `default` (on every program: Overview) |
| `handler_key`, `api_base`, `ui_component` | What the frontend renders and which API backs a premade page |
| `selectable` | Whether the admin can pick it (Overview is not selectable, it is automatic) |
| `allow_components` | Whether the leaf can hold a Rule or Settings |
| `requires_publish`, `rule_family`, `universal`, `permission_key` | Publishing and permission hints |
| `create_time`, `update_time` | Timestamps |

---

## 2. Programs

**What it is.** A program (KBB, CPO, Credit Line). The root object everything else hangs off.

**Endpoints.** `GET|POST /programs`, `GET|PATCH|DELETE /programs/{program}`, `POST /programs/{program}:undelete`.
`{program}` is an id or a slug. gRPC: `ListPrograms`, `CreateProgram`, `GetProgram`, `UpdateProgram`, `DeleteProgram`, `UndeleteProgram`.

**Files.**
- Domain: `src/domain/program.rs` (slug and level rules, tests)
- SQL: `src/adapters/services/postgres/program/` (`program_repository.rs`; `module_changes.rs` handles the modules a create or update carries)
- Use cases: `src/application/usecases/program/` (`create_program`, `get_program`, `list_programs`, `update_program`, `delete_program`, `undelete_program`)
- HTTP / gRPC: `http_apis/program/`, `grpc_apis/program/`

**Table `programs`**:

| Column | Meaning / who changes it |
|---|---|
| `id` | UUIDv7 (time ordered; used for paging). The client may choose it with `?id=`. |
| `name`, `description` | Editable |
| `slug` | Lowercase letters, digits, hyphens; **cannot change** after create; must not look like a UUID. A partial unique index (`uq_programs_slug_active_ci`) keeps live slugs unique, case-insensitively. |
| `level` | `user`, `organization` or `location` |
| `active` | On/off switch, editable |
| `state` | `ACTIVE` or `DELETED`. Delete is soft: it only flips this, undelete flips it back. |
| `config_version` | Starts at 1. **Bumped by every configuration change**: program edits, module and settings changes, setting values, requirements, approval steps. The Overview compares it with the last publication to say "unpublished changes". Applications do **not** bump it. |
| `config_hash` | `md5(id:version)`, changes with the version |
| `create_time`, `update_time` | Timestamps |

**One request: `POST /programs`.** In one transaction: insert into `programs`, insert the picked modules (and their parents and
children) into `program_modules`, add Overview, insert a `program_stats` row, write the audit entry (`created`) and the outbox event.
If a library is referenced, global-settings is checked first, before the transaction opens.

---

## 3. Program modules (the manifest)

**What it is.** Which modules this program has switched on, and what each leaf is. The admin picks modules at create time (or
later with PATCH); this module is how they are stored and read back.

**Endpoint.** `GET /programs/{program}/modules` (query: `enabled_only`, `include_options`). gRPC: `ListProgramModules`.
Writes happen through Programs (`POST /programs`, `PATCH /programs/{program}` with a `modules` list), not a separate endpoint.

**Files.**
- Domain: `src/domain/program_module.rs` (the manifest shape), `src/domain/program_setting.rs` (setting types, `ModuleChange`, validation)
- SQL read: `src/adapters/services/postgres/program_module/program_module_repository.rs` (`fetch_modules` attaches settings, options, saved values, rule summary)
- SQL write: `src/adapters/services/postgres/program/module_changes.rs` (`check_keys`, `ensure_modules`, `apply_changes`)
- Use case: `src/application/usecases/program_module/list_program_modules.rs`
- HTTP / gRPC: `http_apis/program_module/`, `grpc_apis/program_module/`

**Table `program_modules`** (one row per program and module; primary key `(program_id, module_id)`):

| Column | Meaning |
|---|---|
| `program_id` → `programs` | The program |
| `module_id` → `module_definitions` | Which module |
| `enabled` | Switched on or off (`enabled: false` hides it, nothing is deleted) |
| `mode` | For a container leaf: `rule` (the frontend renders the rule engine) or `settings` (typed settings). `NULL` for premade and default modules. |
| `position` | Order |
| `config` | Free-form JSON for the module |
| `version` | Row version |
| `create_time`, `update_time` | Timestamps |

The read joins `module_definitions` (names, keys, handler), `program_fields` / `field_options` (the settings),
`program_field_values` (saved values) and the rule tables (rule summary).

---

## 4. Builder: settings and options (what an admin adds to a leaf)

**What it is.** On a leaf whose `mode` is `settings`, the admin adds typed settings: boolean, number, text, radio, checkbox or
dropdown. Options come from **one** source only: a global-settings **library** (loaded dynamically) or **manual** options.

**Where the code is.** It has no endpoint of its own: it rides on create and PATCH of a program, and shows up in the manifest.
- Write: `module_changes.rs` → `replace_settings` and `sync_manual_options`
- Library options: `src/application/services/library_options.rs` (validation, resolving options, fresh and stale handling)
- global-settings client with cache and webhook invalidation: `src/adapters/services/global_settings/global_settings_client.rs`,
  trait in `src/domain/library_catalog.rs`, webhook at `POST /webhooks/global-settings` (`http_apis/webhook/`)

**Replace semantics.** The settings list you send *replaces* the module's settings: a setting with an `id` is kept and updated, one
without an `id` is created, and one you leave out is soft-deleted. Options are matched by `code`; removed ones are deactivated.

**Table `program_fields`** (a setting):

| Column | Meaning |
|---|---|
| `id` | Setting id |
| `program_id`, `module_id` | Foreign key to `program_modules`: a setting can only live in a module the program chose |
| `name` | What the admin typed |
| `key` | Generated from the name (nobody types an identifier); unique per module while active |
| `field_type` | `boolean`, `number`, `text`, `radio`, `checkbox`, `dropdown`. Cannot change once values exist. |
| `options_source` | `library` or `manual` (never mixed) |
| `library_id` | The global-settings library, when the source is `library` |
| `required`, `default_value`, `position` | Behaviour and order |
| `state` | `ACTIVE` or `DELETED` (soft delete) |

**Table `field_options`** (manual options only): `id`, `field_id` → `program_fields`, `code` (unique per setting), `label`,
`position`, `active`, timestamps. Library-backed settings keep **no** options here; they are read from the library.

---

## 5. Setting values

**What it is.** The values the admin saved for those settings ("N unsaved changes" → Save).

**Endpoints.** `GET /programs/{program}/field-values[?module=<key>]`, `POST /programs/{program}/field-values:batchUpdate`.
gRPC: `ListFieldValues`, `BatchUpdateFieldValues`.

**Files.**
- Domain: `src/domain/program_field_value.rs` (type checks per setting type, tests)
- SQL: `src/adapters/services/postgres/program_field_value/program_field_value_repository.rs`
- Use cases: `src/application/usecases/program_field_value/` (`list_field_values`, `batch_update_field_values`)
- HTTP / gRPC: `http_apis/program_field_value/`, `grpc_apis/program_field_value/`

**Table `program_field_values`** (primary key `(program_id, field_id, scope_type, scope_id)`):

| Column | Meaning |
|---|---|
| `program_id`, `field_id` | Which program and setting |
| `scope_type`, `scope_id` | `program` and `''` today; room for per-store values later |
| `value` | JSON, for boolean / number / text settings |
| `option_ids` | UUID array, for radio / checkbox / dropdown |
| `library_id` | Which library the options came from |
| `create_time`, `update_time` | Timestamps |

**One request: `:batchUpdate`.** Validates every value first (all problems are reported together). Then in one transaction: lock the
program row (`FOR UPDATE`), check the `version` you sent against `config_version` (**409** if stale), write **only values that
differ** in one statement, bump `config_version`, write the audit entry (`values_updated`, with before and after) and the outbox event.
Saving a page nobody edited changes nothing.

---

## 6. Overview and audit log

**What it is.** The default page every program has: counters, the application funnel, rules sync state, "unpublished changes", and
the latest activity. Plus the full history.

**Endpoints.** `GET /programs/{program}/overview`, `GET /programs/{program}/audit-logs[?action=a,b]`.
gRPC: `GetOverview`, `ListAuditLogs`.

**Files.**
- Domain: `src/domain/program_overview.rs` (the Overview shape, `AuditEntry`, the summary text, paging cursor)
- SQL: `src/adapters/services/postgres/program_overview/program_overview_repository.rs` (ONE query builds the whole page),
  `src/adapters/services/postgres/audit_log/audit_log_repository.rs`
- Use cases: `src/application/usecases/program_overview/get_overview.rs`, `src/application/usecases/audit_log/list_audit_logs.rs`
- HTTP / gRPC: `http_apis/program_overview/`, `grpc_apis/program_overview/` (both endpoints live here)

**Tables it reads**: `programs` (version), `program_stats` (counters), `applications` (funnel by status), `rule_sets` and `publications`
(rules state, empty until rules are built), `audit_log` (activity).

**Table `program_stats`** (one row per program; a primary-key read, never a `COUNT(*)`):
`total_applications`, `pending_approvals`, `active_enrollments`, `enrollments_total`, `update_time`.
It is **written by the Applications module** (create adds 1 to total, submit adds 1 to pending, withdraw takes 1 off pending).

**Table `audit_log`** (monthly partitions, newest-first index per program):

| Column | Meaning |
|---|---|
| `id`, `create_time` | Primary key (the time puts the row in its monthly partition) |
| `program_id` | Which program's history it belongs to |
| `actor` | The `X-User-Id` header (gRPC: `x-user-id`), or `system` |
| `action` | `created`, `updated`, `values_updated`, `requirement_created`, `submitted`, ... |
| `target_type`, `target_id` | `program` / `application` and its id |
| `before`, `after` | JSON of the state before and after |

**Table `outbox`** (written next to every audit entry): `id`, `topic` (`program.updated`, `application.submitted`, ...), `event_key`
(the program id), `payload`, `status` (`pending`), `attempts`, `create_time`, `published_time`. Nothing reads it yet; it is the hook
for other services.

---

## 7. Requirements

**What it is.** What a store must satisfy before it can be approved: sign up, card on file, credit application, contract, and any
number of custom fields.

**Endpoints.** `GET|POST /programs/{program}/requirements`, `GET|PATCH|DELETE .../requirements/{requirement}`,
`POST .../requirements:batchUpdate` (reorder). gRPC: `ListRequirements`, `GetRequirement`, `CreateRequirement`, `UpdateRequirement`,
`DeleteRequirement`, `BatchUpdateRequirements`.

**Files.**
- Domain: `src/domain/requirement.rs`
- SQL: `src/adapters/services/postgres/requirement/requirement_repository.rs`
- Use cases: `src/application/usecases/requirement/` (six files)
- HTTP / gRPC: `http_apis/requirement/`, `grpc_apis/requirement/`

**Table `requirements`**:

| Column | Meaning |
|---|---|
| `id` | UUIDv7 |
| `program_id` → `programs` | Owner |
| `type` | `signup`, `card_on_file`, `credit_application`, `contract` or `custom_field`. **Cannot change.** A partial unique index (`uq_requirements_fixed_type`) allows one of each fixed type per program; `custom_field` repeats. |
| `name` | 1-120 characters |
| `config` | JSON object, up to 16 KB |
| `position` | Display order (not unique, because a reorder rewrites many rows at once) |
| `create_time`, `update_time` | Timestamps |

**Behavior.** Create, update, delete and reorder each lock the program row, bump `programs.config_version`, and write an audit entry
(`requirement_created`, `_updated`, `_deleted`, `requirements_reordered`) in the same transaction. A request that changes nothing writes
nothing. Delete is a hard delete.

---

## 8. Approval steps

**What it is.** The ordered sign-offs an application goes through (for example Credit review, then Final approval).

**Endpoints.** Same shape as Requirements, under `/approval-steps`: list, add, get, update, remove, `approval-steps:batchUpdate`.
gRPC: `ListApprovalSteps`, `GetApprovalStep`, `CreateApprovalStep`, `UpdateApprovalStep`, `DeleteApprovalStep`, `BatchUpdateApprovalSteps`.

**Files.** `src/domain/approval_step.rs`, `src/adapters/services/postgres/approval_step/approval_step_repository.rs`,
`src/application/usecases/approval_step/`, `http_apis/approval_step/`, `grpc_apis/approval_step/`.

**Table `approval_steps`**:

| Column | Meaning |
|---|---|
| `id` | UUIDv7 |
| `program_id` → `programs` | Owner |
| `name` | 1-120 characters |
| `approver_role` | The role allowed to decide this step (1-64 characters) |
| `position` | Order; the lowest is the first step |
| `create_time`, `update_time` | Timestamps |

**Behavior.** Same write pattern as Requirements (program lock, version bump, audit, outbox). One extra rule: `signoffs.approval_step_id`
is `ON DELETE RESTRICT`, so a step that has recorded signoffs cannot be removed. The repository turns that database error into a
clear **409**. Applications read this table: submit picks the first step by `(position, id)`.

---

## 9. Applications

**What it is.** A store's request to join a program (the Application Panel page).

**Endpoints.** `GET|POST /programs/{program}/applications`, `GET .../applications/{application}`, `POST .../{application}:submit`,
`POST .../{application}:withdraw`. gRPC: `ListApplications`, `GetApplication`, `CreateApplication`, `SubmitApplication`, `WithdrawApplication`.

**Files.**
- Domain: `src/domain/application.rs` (statuses, which move is allowed, the status filter, validation, tests)
- SQL: `src/adapters/services/postgres/application/application_repository.rs`
- Use cases: `src/application/usecases/application/` (`list`, `get`, `create`, `submit`, `withdraw`)
- HTTP / gRPC: `http_apis/application/`, `grpc_apis/application/`
- Migration for the one-open-application rule: `migrations/20261022100000_applications_open_unique.up.sql`

**Table `applications`**:

| Column | Meaning |
|---|---|
| `id` | UUIDv7; the list pages newest-first on it |
| `program_id` → `programs` | Owner |
| `store_id`, `store_name` | The applying store |
| `website_id`, `website_name` | The website it applies for |
| `status` | `draft` → `submitted` → (`in_review` → `approved` / `rejected`) or `withdrawn` |
| `current_step_id` → `approval_steps` | The step it waits at (cleared on withdraw). The list joins this for the step name. |
| `requirements_snapshot` | JSON frozen at **submit**: the program's requirements at that moment |
| `answers` | The store's answers, keyed by requirement id (up to 64 KB; not checked against requirements yet) |
| `submitted_time`, `decided_time` | Set by submit and by withdraw (later also by approve / reject) |
| `create_time`, `update_time` | Timestamps |

Indexes that make the list fast: `(program_id, id)` for All, `(program_id, status, id)` for a chip,
`uq_applications_open (program_id, store_id, website_id) WHERE status IN ('draft','submitted','in_review')` so a store has one open
application per website (a second is a 409; after a decision or withdrawal it can apply again).

**What each action writes (all in one transaction):**

| Action | Rules | Writes |
|---|---|---|
| `POST .../applications` | One open application per store and website | new `applications` row (`draft`), `program_stats.total_applications` +1, audit `created`, outbox |
| `:submit` | Only a `draft`; locks the application row | `status='submitted'`, `current_step_id` = first approval step, `requirements_snapshot` filled, `submitted_time`, `program_stats.pending_approvals` +1, audit `submitted`, outbox |
| `:withdraw` | `draft`, `submitted` or `in_review` | `status='withdrawn'`, `current_step_id` cleared, `decided_time`; `pending_approvals` -1 only if it was pending; audit `withdrawn`, outbox |

Applications are data, not configuration: they do **not** bump `programs.config_version`, and they do not lock the program row.

---

## 10. What is built and what is not

Built and in use (12 tables): `module_definitions`, `programs`, `program_modules`, `program_stats`, `program_fields`, `field_options`,
`program_field_values`, `requirements`, `approval_steps`, `applications`, plus `audit_log` and `outbox` (written by every module).

In the schema but not built yet: `signoffs` (approve / reject, next), `enrollments`, `program_overrides`, `rule_sets`, `rule_versions`,
`publications`, `program_storage`, `document_folders`, `documents`, `operations`.
