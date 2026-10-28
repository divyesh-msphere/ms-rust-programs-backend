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

## 1. Module catalog (the catalog)

**What it is.** The read-only list of every module a program can have (Overview, Applications, Enrolled Stores, and so on),
arranged as a tree of groups and leaves. It is seeded by migrations and never changed through the API.

**Endpoints.** `GET /module-catalog` (list, also as a tree), `GET /module-catalog/{module_catalog_item}`.
gRPC: `ListModuleCatalog`, `GetModuleCatalogItem`.

**Files.**
- Domain: `src/domain/module_catalog_item.rs`
- SQL: `src/adapters/services/postgres/module_catalog_item/` (`module_catalog_item_repository.rs`, `_mapper.rs`, `_model.rs`)
- Use cases: `src/application/usecases/module_catalog_item/` (`list_module_definitions.rs`, `get_module_catalog_item.rs`, `get_module_catalog_tree.rs`)
- HTTP / gRPC: `http_apis/module_catalog_item/`, `grpc_apis/module_catalog_item/`
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
| `selectable` | Whether the admin can pick it (Overview and Danger zone are not selectable, they are automatic; Notifications is selectable) |
| `allow_components` | Whether the leaf can hold a Rule or Settings |
| `default_mode`, `default_rule` | A page that starts configured: `default_mode` (`rule` or `settings`) is the mode a program gets when it selects the page, and `default_rule` is a starter rule saved as draft 1. Only **Fees** (`order_transaction.core_returns.fees`, replacing the old Core Return Fees Panel) has them: it is a Rule leaf with one decision-table row (every return, fee 0) until the admin edits it. `default_mode` is in the API; `default_rule` is internal |
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
| `config_version` | Starts at 1. **Bumped by every configuration change**: a changed `level`, `active` flag or module list, setting values, requirements, approval steps, store discounts and rule saves (a rename or a new description does not). The Overview compares it with the last publication to say "unpublished changes". Applications do **not** bump it. |
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
- global-settings client with a cache that refreshes in the background (no webhook): `src/adapters/services/global_settings/global_settings_client.rs`,
  trait in `src/domain/library_catalog.rs`

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

**Endpoints.** `GET /programs/{program}/setting-values[?module=<key>]`, `POST /programs/{program}/setting-values:batchUpdate`.
gRPC: `ListSettingValues`, `BatchUpdateSettingValues`.

**Files.**
- Domain: `src/domain/program_setting_value.rs` (type checks per setting type, tests)
- SQL: `src/adapters/services/postgres/program_setting_value/program_setting_value_repository.rs`
- Use cases: `src/application/usecases/program_setting_value/` (`list_setting_values`, `batch_update_setting_values`)
- HTTP / gRPC: `http_apis/program_setting_value/`, `grpc_apis/program_setting_value/`

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
It is **written by the Applications, Signoffs and Enrollments modules**: applications move `total_applications` and `pending_approvals`, a final signoff takes 1 off pending, and enrollments move `enrollments_total` and `active_enrollments`.

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

## 6a. Notifications (premade page in Settings)

**What it is.** The grid of trigger points x channels (Email, SMS, Push, In-App). It is a **premade** page
(`configuration_oversight.settings.universal.notifications`, `handler_key notifications`, `api_base notifications`) that the admin **picks** in Create
program (`selectable = true`, not `universal`). It then arrives with the normal modules load (`GET /programs/{p}/modules`), and the page calls its own API.
Migration `20261028110000_notifications_premade.up.sql` made the page premade and added `program_notification_triggers`.

| API | What it does | Tables |
|---|---|---|
| `GET /programs/{p}/notifications` | `{path, version, channels, triggers:[{key, label, email, sms, push, in_app}]}`; 404 without the page | R `program_notification_triggers`, `programs` |
| `PUT /programs/{p}/notifications` | Saves the whole grid (ticks, added and removed triggers) in one transaction; `version` checked (409 when stale); unchanged = no write | W `program_notification_triggers`, `programs.config_version`, `audit_log` (`notifications_updated`), `outbox` |

A program starts with 8 usual triggers, seeded once when the page is added (`ensure_modules`), not again after the admin empties the grid.
**Code:** `domain/notifications.rs`, `postgres/notifications/`, `application/usecases/notifications/`, `http_apis/notifications/`, `grpc_apis/notifications/`.

## 6b. Danger zone (premade page in Settings)

**What it is.** The card at the bottom of Settings: **Archive program** and **Delete program**. It is a **premade** page
(`configuration_oversight.settings.universal.danger_zone`, `handler_key danger_zone`, `api_base danger-zone`) that is on **every program** without
being selected: `selectable = false`, `universal = true`, and `ensure_modules` adds every `premade` + `universal` page (with its parents) to a new
program. Migration `20261028100000_danger_zone_premade.up.sql` did the same for existing programs.

**Archive vs delete.** *Archive* is a separate, reversible state: `programs.archive_time` (NULL = not archived). `state` stays `ACTIVE`, so the program
is still readable and editable; only the program list leaves it out (`show_archived=true` lists archived ones). *Soft delete* (`DELETE /programs/{p}`,
`state DELETED`) is unchanged. *Delete permanently* removes the row, and everything cascades.

| API | What it does | Tables |
|---|---|---|
| `GET /programs/{p}/danger-zone` | `archived`, `archive_time`, `can_archive`, `can_unarchive`, `can_delete`, `delete_blockers`, `delete_blocked_reason` | R `programs`, counts of `enrollments`, `applications`, `publications`, `documents` |
| `POST /programs/{p}/danger-zone:archive` | Sets `archive_time`; 400 if already archived | W `programs`, `audit_log` (`archived`), `outbox` |
| `POST /programs/{p}/danger-zone:unarchive` | Clears it; 400 if not archived | W `programs`, `audit_log` (`unarchived`), `outbox` |
| `POST /programs/{p}/danger-zone:deletePermanently` | 204; **409** while the program has history | D `programs` (cascade), `audit_log` (`deleted_permanently`, kept) |

**History** (what blocks a permanent delete) is any enrollment, application, publication or document. Documents count because their files are in object
storage and would be left behind. The check and the delete run in one transaction with the program row locked.
**Code:** `domain/danger_zone.rs`, `postgres/danger_zone/`, `application/usecases/danger_zone/`, `http_apis/danger_zone/`, `grpc_apis/danger_zone/`.

---

## 7. Requirements

> **Parked.** Switched off by default (`features.enrollment_review`); nothing is deleted. See [parked-enrollment-review.md](parked-enrollment-review.md).

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

> **Parked.** Switched off by default (`features.enrollment_review`); nothing is deleted. See [parked-enrollment-review.md](parked-enrollment-review.md).

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

> **Parked.** Switched off by default (`features.enrollment_review`); nothing is deleted. See [parked-enrollment-review.md](parked-enrollment-review.md).

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
| `status` | `draft` → `submitted` → (`in_review` → `approved` / `rejected`) or `withdrawn`. Moved by submit, withdraw and signoffs. |
| `current_step_id` → `approval_steps` | The step it waits at (cleared on withdraw). The list joins this for the step name. |
| `requirements_snapshot` | JSON frozen at **submit**: the program's requirements at that moment |
| `answers` | The store's answers, keyed by requirement id (up to 64 KB; not checked against requirements yet) |
| `submitted_time`, `decided_time` | Set by submit; `decided_time` by withdraw and by a final approve / reject |
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

## 10. Signoffs (approve and reject)

> **Parked.** Switched off by default (`features.enrollment_review`); nothing is deleted. See [parked-enrollment-review.md](parked-enrollment-review.md).

**What it is.** An approver's decision on the approval step an application is waiting at. It is what moves an application from
`submitted` to `in_review` to `approved`, or to `rejected`.

**Endpoints.** `GET|POST /programs/{program}/applications/{application}/signoffs`, `GET .../signoffs/{signoff}`.
gRPC: `ListSignoffs`, `GetSignoff`, `CreateSignoff`. There is no update or delete: a decision is final.

**Files.**
- Domain: `src/domain/signoff.rs` (`outcome()` decides what a decision does to the application, with unit tests)
- SQL: `src/adapters/services/postgres/signoff/signoff_repository.rs` (shares `bump_stats` and `db_error` with the Applications repository)
- Use cases: `src/application/usecases/signoff/` (`list_signoffs`, `get_signoff`, `create_signoff`)
- HTTP / gRPC: `http_apis/signoff/`, `grpc_apis/signoff/`

**Table `signoffs`** (`UNIQUE (application_id, approval_step_id)`: one decision per step per application):

| Column | Meaning |
|---|---|
| `id` | UUIDv7; the list orders by it, which is the order decisions were made |
| `application_id` → `applications` | Which application (cascades if the application is removed) |
| `approval_step_id` → `approval_steps` | The step decided. `ON DELETE RESTRICT`: a step with signoffs cannot be removed. |
| `decision` | `approved` or `rejected` |
| `comment` | Optional, up to 1000 characters |
| `actor` | The approver (`X-User-Id`) |
| `create_time` | When |

**What one `POST .../signoffs` does (all in one transaction):**
1. Lock the application row (`FOR UPDATE`) so two decisions take turns.
2. Refuse (400) unless it is `submitted` or `in_review` and waiting at a step (`applications.current_step_id`).
3. If the request carries `approval_step_id` and it is not the current step: **409**, nothing written (double-click guard).
4. Find the next step by `(position, id)`. `approved` with a next step → `in_review` at that step; `approved` on the last step →
   `approved`, step cleared; `rejected` → `rejected`, step cleared.
5. Insert the `signoffs` row; update `applications` (`status`, `current_step_id`, `decided_time` when final).
6. If it was a final decision: `program_stats.pending_approvals` -1.
7. Audit entry (`step_approved` / `approved` / `rejected`, with before and after) and outbox event.

Not enforced yet: that the approver has the step's `approver_role` (waits for authentication).

---

## 11. Enrollments (Enrolled Stores)

**What it is.** A store enrolled in a program on one website: the Enrolled Stores page. The end of the line for an approved application.

**Endpoints.** `GET|POST /programs/{program}/enrollments`, `POST .../enrollments:batchCreate`, `GET|PATCH .../enrollments/{enrollment}`.
gRPC: `ListEnrollments`, `GetEnrollment`, `CreateEnrollment`, `BatchCreateEnrollments`, `UpdateEnrollment`.

**Files.**
- Domain: `src/domain/enrollment.rs` (statuses, the allowed status moves, validation, the batch outcome type, tests)
- SQL: `src/adapters/services/postgres/enrollment/enrollment_repository.rs`. It also holds `enroll_after_approval`, which the Signoffs
  repository calls when the last step is approved.
- Use cases: `src/application/usecases/enrollment/` (`list`, `get`, `create`, `batch_create`, `update`)
- HTTP / gRPC: `http_apis/enrollment/`, `grpc_apis/enrollment/`

**Table `enrollments`**:

| Column | Meaning |
|---|---|
| `id` | UUIDv7; the list pages newest-first on it |
| `program_id` → `programs` | Owner |
| `application_id` → `applications` | The application that produced it; `NULL` for an admin enroll (`ON DELETE SET NULL`) |
| `store_id`, `store_name` | The store. The name is supplied by the caller. |
| `store_resolved` | `false` when no name was known: the page shows "Unresolved store" |
| `website_id`, `website_name` | The website |
| `customer_group` | Free text per enrollment (for example `tier-1`) |
| `status` | `pending`, `active`, `suspended`, `ended`. Moves: `pending -> active`, `active <-> suspended`, anything `-> ended` (final). |
| `enrolled_time` | When it first became active (`NULL` while pending) |
| `contracts_count`, `last_contract_time` | Always 0 / `NULL` until agreements are built |
| `create_time`, `update_time` | Timestamps |

Indexes: `(program_id, id)` for the list, `(program_id, website_id, id)` for All websites, `(program_id, lower(store_name) text_pattern_ops)`
for the store search, and `uq_enrollments_live (program_id, store_id, website_id) WHERE status IN ('pending','active','suspended')` so a
store is enrolled once per website (a second is a 409; after `ended` it can be enrolled again).

**What writes it (all in one transaction with the counters, the audit entry and the outbox event):**

| Action | Rules | Overview counters |
|---|---|---|
| Last approval step approved (in the Signoffs transaction) | Skipped if the store already has a live enrollment on the website | total +1, active +1 |
| `POST .../enrollments` | One live enrollment per store and website | total +1, active +1 if `active` |
| `:batchCreate` | One statement for up to 500 items; each item gets its own outcome | by the number created |
| `PATCH .../enrollments/{id}` | Status move must be allowed (409 otherwise); a request that changes nothing writes nothing | active ±1 as the status enters or leaves `active` |

Enrollments are data, not configuration: they do **not** bump `programs.config_version` and do not lock the program row.

---

## 12. Store discounts

**What it is.** The Operate → Store discounts page: each enrolled store's effective discount and which level it came from. The module is
`catalog_pricing.store_discounts.control_panel` (it requires publish), and a program must have it switched on.

**Endpoints.** `GET /programs/{p}/store-discounts[/{enrollment}]`, `PATCH .../store-discounts/{enrollment}`, `POST .../store-discounts:batchUpdate`,
`GET .../store-discount-levels`, `PATCH .../store-discount-levels/program`, `PATCH .../store-discount-levels/domains/{website_id}`.
gRPC: `ListStoreDiscounts`, `GetStoreDiscount`, `UpdateStoreDiscount`, `BatchUpdateStoreDiscounts`, `ListDiscountLevels`, `UpdateDiscountLevel`.

**Files.**
- Domain: `src/domain/store_discount.rs` (the three levels, `resolve()` field by field, `DiscountChange`, validation, 10 unit tests)
- SQL: `src/adapters/services/postgres/store_discount/store_discount_repository.rs`
- Use cases: `src/application/usecases/store_discount/` (six files)
- HTTP / gRPC: `http_apis/store_discount/`, `grpc_apis/store_discount/`

**How a discount is worked out.** Three levels, lowest to highest priority: Program, Domain (website), Store. Each of the four fields
(`enabled`, `type`, `amount`, `currency_code`) takes the value of the highest level that set it, so a store can override only the amount.
The Set At badge is the highest level that set anything. With nothing set anywhere: disabled, a percentage, of 0.

**Where each level is stored:**

| Level | Table and column | Key |
|---|---|---|
| Program | `program_modules.config` → `discount` (the Store Discounts module's own row) | `program_id` + the module |
| Domain | `program_overrides` (`scope_type = 'domain'`) | `scope_id` = the website id |
| Store | `program_overrides` (`scope_type = 'store'`) | `scope_id` = the enrollment id |

`program_overrides` columns: `id`, `program_id`, `scope_type`, `scope_id`, `config` (JSON `{enabled, type, amount, currency_code}`, only the fields
that level sets), `create_time`, `update_time`; unique on `(program_id, scope_type, scope_id)`, so every lookup is one index probe. A level that
sets nothing has no row. The stores listed come from `enrollments` (pending, active or suspended; not `ended`).

**What one edit does (all in one transaction):** lock the program row, check the module is on, work out each store's new level and refuse the
whole request if any resulting discount is invalid (a percentage over 100), write the changed levels with one statement (rows that became empty are
deleted with one statement), bump `programs.config_version` **once**, write one audit entry and one outbox event. A request that changes nothing
writes nothing. Publishing the resolved map into the program bundle is not built yet.

---

## 13. Rule sets, versions and publishing

**What it is.** The rules of a Rule leaf (a module with mode `rule`, for example Warranty), saved as immutable versions, and the **Publish** step that
freezes the whole program configuration into one bundle. The rule engine is a frontend component: the backend only keeps and publishes what it saves.

**A rule page that starts with a rule.** **Fees** (Order & Transaction → Core Returns → Fees) is a Rule leaf from the moment a program selects it, with no Settings or Rules choice, and its rule set already holds a simple starter rule as draft 1 (a GoRules decision graph: request, one decision table with a single row that charges 0, response). It can be published as it is; the admin's first save becomes version 2. The starter comes from `module_definitions.default_rule`; see `seed_default_rules` in `postgres/program/module_changes.rs`.

**Endpoints.** `GET /programs/{p}/rule-sets`, `GET .../rule-sets/{module_key}`, `POST|GET .../rule-sets/{module_key}/versions`, `GET .../versions/{version}`,
`POST|GET /programs/{p}/publications`, `GET .../publications/{publication}`.
gRPC: `ListRuleSets`, `GetRuleSet`, `CreateRuleVersion`, `ListRuleVersions`, `GetRuleVersion`, `ListPublications`, `CreatePublication`, `GetPublication`.

**Files.**
- Domain: `src/domain/rule_set.rs` (the rule set, version, validation, `state()`), `src/domain/publication.rs`
- SQL: `src/adapters/services/postgres/rule_set/rule_set_repository.rs`, `src/adapters/services/postgres/publication/publication_repository.rs` (the bundle is built here)
- Use cases: `src/application/usecases/rule_set/` (5), `src/application/usecases/publication/` (3)
- HTTP / gRPC: `http_apis/rule_set/`, `http_apis/publication/`, `grpc_apis/rule_set/`, `grpc_apis/publication/`

**Table `rule_sets`** (one per rule leaf; `UNIQUE (program_id, module_id)`; the row is created when the leaf is made a rule leaf):

| Column | Meaning |
|---|---|
| `id` | UUIDv7 |
| `program_id`, `module_id` | Foreign key to `program_modules`: only a module the program chose |
| `rule_type` | Informational; defaults from the module's `rule_family` |
| `draft_version` | The newest saved version number (`NULL` until the first save) |
| `published_version` | The live version number (`NULL` until the first publish) |
| `settings` | JSON, reserved |
| `create_time`, `update_time` | Timestamps |

**Table `rule_versions`** (append-only, `UNIQUE (rule_set_id, version)`):

| Column | Meaning |
|---|---|
| `id` | UUIDv7; the list pages newest-first on it |
| `rule_set_id` → `rule_sets` | Owner |
| `version` | 1, 2, 3, ... |
| `status` | `draft` (the newest save), `published` (live), `superseded` |
| `definition` | JSON, the rule model the builder draws (up to 1 MB), kept as is |
| `compiled` | JSON, the generated decision graph, when the rule engine made one |
| `valid`, `validation_errors` | The rule engine's verdict. An invalid newest draft blocks publishing. |
| `content_hash` | `md5` of the definition: saving the same definition again creates no new version |
| `size_bytes`, `created_by`, `create_time` | Size, the saving user, when |

**Table `publications`** (append-only): `id`, `program_id`, `config_version` (the program version it covers), `config_hash`, `bundle` (JSON, with a
`summary` inside), `created_by`, `create_time`. `(program_id, id)` is indexed, so "the latest publication" is one step back.

**What a save does:** lock the program, check the module is switched on with mode `rule`, hash the definition (unchanged: write nothing), insert the next
version as `draft`, supersede older drafts, point `rule_sets.draft_version` at it, bump `programs.config_version`, audit `rule_version_saved`, outbox.

**What a publish does (one transaction, program locked):** refuse when a rule's newest draft is invalid or when nothing is new; promote every changed rule
leaf's draft to `published` (the old one becomes `superseded`, `rule_sets.published_version` follows); build the bundle (program, enabled modules with settings
and values, requirements, approval steps, the store → discount map of active stores, the published rules) and insert the publication; audit `published`, outbox.
It records `config_version` but does not change it, so the Overview shows **in sync**.

---

## 14. Documents

**What it is.** The Configure → Documents page: a file manager per program with folders, locations (All files, Recent, Needs review, Quarantined), search, upload,
review and download. The module is `document_agreement.documents.documents_panel`; a program must have it switched on.

**The key idea.** Postgres keeps only the **metadata**; the **bytes live in object storage** (S3 or MinIO). The service never carries them: it signs a short-lived URL, the
browser uploads or downloads directly, and the service later asks the storage what arrived.

**Endpoints.** `GET /programs/{p}/documents:summary`; `GET|POST .../documents`; `GET|PATCH|DELETE .../documents/{d}`; `POST .../documents/{d}:complete`, `:download`, `:review`;
`GET|POST .../document-folders`; `PATCH|DELETE .../document-folders/{f}`; `POST /webhooks/document-scan`. gRPC: 13 methods, no bytes.

**Files.**
- Domain: `src/domain/document.rs` (statuses, name cleaning, the upload policy with its 413 / 415, scan and review rules, the repository trait), `src/domain/object_storage.rs` (the storage trait)
- Storage: `src/adapters/services/object_storage/s3_client.rs` (AWS Signature V4 presigning by hand, checked against AWS's documented example; `HEAD` through a presigned URL)
- SQL: `src/adapters/services/postgres/document/document_repository.rs`
- Use cases: `src/application/usecases/document/` (14, including `scan_document` for the webhook)
- HTTP / gRPC: `http_apis/document/` (the webhook is in the same controller file), `grpc_apis/document/`
- Config: `[storage]` in `src/config/config.toml`; MinIO in `docker-compose.yml`

**Table `documents`:**

| Column | Meaning |
|---|---|
| `id` | UUIDv7; lists page newest-first on it |
| `program_id` → `programs` | Owner |
| `folder_id` → `document_folders` | Where it is (`ON DELETE SET NULL`); defaults to the General folder |
| `name` | A label (slashes and control characters become `_`); the storage key never comes from it |
| `content_type`, `size_bytes` | Declared at create; the storage refuses an upload that differs |
| `file_key` | `programs/{program}/documents/{id}`, generated by the server |
| `status` | `uploading` (hidden) → `pending_scan` → `ok` / `needs_review` / `quarantined` |
| `source_type`, `source_id` | Which other module created it (none for a manual upload); reserved |
| `uploaded_by`, `checksum` | The uploader; the declared SHA-256 |
| `scan_time`, `review_note`, `reviewed_by`, `reviewed_time` | The scanner's and the reviewer's results |
| `state` | `ACTIVE` or `DELETED` (soft delete) |
| `create_time`, `update_time` | Timestamps |

**Table `document_folders`:** `id`, `program_id`, `parent_id` (self reference), `name` (unique per parent, ignoring case), `key` (`general`, `import_artifacts`,
`program_documents`, `store_enrollment_documents`; `NULL` for a custom folder), `system` (cannot be renamed or deleted), `document_count`, timestamps. The four system folders
are created with each program (and were backfilled for the programs that already existed).

**Table `program_storage`** (one row per program, the counters the page shows): `file_count`, `total_bytes`, `needs_review_count`, `quarantined_count`, `update_time`. They move
in the same transaction as every complete, delete, move and status change, so the page never runs `COUNT(*)`.

**What each action does (one transaction each):**

| Action | Writes |
|---|---|
| `POST .../documents` | The row as `uploading`. Nothing counted, nothing audited. Refused first for size, type, quota (10,000 files) or a bad checksum. |
| `:complete` | After asking the storage (exists, same size, type and SHA-256): `pending_scan`, files +1, bytes +size, folder +1, audit `uploaded`, outbox (the scanner listens) |
| scan webhook | `ok` / `needs_review` / `quarantined`; the review or quarantine counter +1; audit `scanned` (actor = the scanner). A file not `pending_scan` is left alone. |
| `:review` | Only from `needs_review`: `ok` or `quarantined`; review counter −1, quarantine +1 on reject; audit `reviewed` |
| `PATCH` document | Rename and/or move; the two folder counters follow; audit `updated` |
| `DELETE` document | `state = DELETED`; files −1, bytes −size, folder −1, the status counter −1; audit `deleted` |

Files are data, not configuration: nothing here bumps `config_version`.

---

## 15. What is built and what is not

Built and in use (21 tables): `module_definitions`, `programs`, `program_modules`, `program_stats`, `program_fields`, `field_options`,
`program_field_values`, `requirements`, `approval_steps`, `applications`, `signoffs`, `enrollments`, `program_overrides`, `rule_sets`, `rule_versions`, `publications`, `documents`, `document_folders`, `program_storage`, plus `audit_log` and `outbox` (written by every module).

In the schema but not built yet: `operations`. The Imports module (`import_jobs`, `import_job_rows`) is **dropped**: it will not be built.
