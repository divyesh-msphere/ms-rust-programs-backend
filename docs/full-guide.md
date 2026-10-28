# The Programs service: the full guide

Read this once, top to bottom, and you will know what every module is for, which APIs it has, which database tables it touches, and why it was built
that way. It is the "big picture" companion to the more detailed files:

| File | What it adds |
|---|---|
| [modules-walkthrough.md](modules-walkthrough.md) | Every column of every table, module by module |
| [database-diagram.md](database-diagram.md) | The ER diagram (paste it into mermaid.live) |
| [postman-guide.md](postman-guide.md) | How to test each module, with request examples |
| [program-settings-plan.md](program-settings-plan.md) | The original design and the decisions taken |
| `openapi.yaml` | The HTTP contract (Swagger: `docker compose up -d swagger`, http://localhost:8082) |
| `proto/programs/v1/*.proto` | The gRPC contract |

---

## 1. What this service is

A **program** is an offering a store can join, for example KBB, CPO or Credit Line. This service lets an admin **build** a program (choose which pages it
has, add settings and rules), **run** it (stores apply, approvers decide, stores get enrolled, files are kept) and **publish** it (freeze its
configuration into a snapshot other services read).

The admin screen for one program has three tabs:

| Tab | Meaning | Pages (modules) |
|---|---|---|
| **Overview** | The default page every program has | Overview (counters, funnel, rule sync, activity) |
| **Configure** | Setting the program up | Requirements, Documents, settings and rule pages, the rule builder pages |
| **Operate** | Running it day to day | Applications (with approve / reject), Stores, Store discounts |

Every page is a **module**. A program only has the modules its admin picked.

There are three kinds of module (`module_definitions.source`):

| Kind | What it is | Examples |
|---|---|---|
| `default` | On every program, not selectable | Overview |
| `premade` | A page written in code; the admin only switches it on (or, when `universal`, it is on every program: Danger zone) | Applications, Enrolled Stores, Store Discounts, Requirements, Documents, Notifications, Danger zone |
| `container` | An empty page; the admin decides what it is: a **Rule** page (the frontend rule engine) or a **Settings** page (typed settings) | Warranty, Pre-Enrollment, Program Settings, Shopping Cart |

**Who calls the service:** the admin frontend (HTTP), other services (gRPC, and by reading the published bundle), the storage and the scanner (webhooks),
and the service calls global-settings for library options.

---

## 2. How the code is organized

One idea runs through everything: **every module has the same layers**, and a request goes down them and the answer comes back up.

```
HTTP (actix)  or  gRPC (connectrpc)                 src/adapters/inputs/http_apis/<module>/   and  .../grpc_apis/<module>/
        |   controller / service  +  mappers        turn the request into a use-case input, the result into JSON / protobuf
        v
   Use case                                         src/application/usecases/<module>/<operation>.rs     (one file per operation)
        |   checks the program exists, validates
        v
   Domain                                           src/domain/<module>.rs      types, rules, validation, the repository TRAIT, tests
        |   (no SQL here)
        v
   Repository (Postgres)                            src/adapters/services/postgres/<module>/<module>_repository.rs      all the SQL
        |   (and for Documents, the S3 client:      src/adapters/services/object_storage/s3_client.rs)
        v
   Postgres  /  object storage  /  global-settings
```

Other folders worth knowing:

| Folder | Holds |
|---|---|
| `src/dto/payloads/json/`, `src/dto/presenters/json/` | The JSON request shapes and response shapes |
| `src/adapters/inputs/http_apis/shared/routes.rs` | Where every HTTP route is mounted |
| `src/adapters/inputs/grpc_apis/programs_service_impl.rs` | The single gRPC service; it hands each call to a per-module service |
| `src/infrastructure/repositories.rs`, `src/application/usecases/mod.rs`, `src/lib.rs` | The wiring (who gets which repository, the S3 client, the global-settings client) |
| `src/adapters/generated/` | Code generated from the `.proto` files (`buf generate`); never edited by hand |
| `src/config/config.toml` | Settings (ports, database, global-settings, object storage); any value can be overridden with an `APP__...` environment variable |
| `migrations/` | The database, as numbered `*.up.sql` files (idempotent) with a matching `down/` file |
| `postman/` | The Postman collection and environment |

**A request, traced.** `POST /programs/{p}/requirements`:
1. `requirement_controllers.rs` receives the JSON and the `X-User-Id` header (the actor).
2. `requirement_mappers.rs` builds the use-case input.
3. `create_requirement.rs` loads the program (404 if it does not exist) and validates the type, name and config.
4. `requirement_repository.rs` opens ONE transaction: it locks the program row, inserts the requirement, bumps `programs.config_version`, writes an `audit_log`
   row and an `outbox` row, and commits.
5. The result goes back up, through the presenter, as JSON. The gRPC call `CreateRequirement` runs the same use case through the other mappers.

---

## 3. The rules every module follows

These are the reason the modules feel the same. If you know them, you can predict how any endpoint behaves.

| Rule | What it means |
|---|---|
| **One transaction per write** | A change, its audit entry, its outbox event and any counter commit together or not at all. |
| **Audit and outbox** | Every change writes `audit_log` (who, what, before, after) and `outbox` (an event for other services). The helpers are `record_change` and `record_event` in `postgres/program/program_repository.rs`. |
| **IDs** | UUIDv7: they are in time order, so "newest first" and paging are index range scans. A caller may choose the id of a new resource with `?id=` (AEP-133). |
| **Programs by id or slug** | `{program}` in any path is the UUID or the slug. A slug is lowercase letters, digits and single hyphens, cannot change, and cannot look like a UUID. |
| **Paging** | Keyset paging: `max_page_size` (default 50, capped at 200) and `page_token` (the last id). A deep page costs the same as the first. Small lists (folders, steps, requirements, rule sets) are returned whole. |
| **Soft delete** | Programs, documents and settings use `state = DELETED`, reversible where it makes sense (`:undelete` for programs). Requirements, approval steps and folders are removed for good. |
| **Custom methods** | Actions that are not plain create/read/update/delete use a colon: `:submit`, `:withdraw`, `:complete`, `:download`, `:review`, `:batchUpdate`, `:batchCreate`, `:summary`, `:undelete`. |
| **`config_version`** | `programs.config_version` goes up whenever the program's **configuration** changes (see section 7). Publishing records it. The Overview compares the two to show "unpublished changes". |
| **Nothing changed, nothing written** | Saving the same values again, reordering to the same order, a PATCH that changes nothing: no write, no version bump, no audit entry. |
| **The actor** | The `X-User-Id` header (gRPC: `x-user-id` metadata) is recorded as the actor; `system` when absent. There is no authentication yet. |
| **Module must be on** | Store Discounts and Documents answer 400 if the program did not pick that module. (The older modules do not check this yet.) |
| **HTTP and gRPC together** | Every operation exists on both. Each change updates the proto, `openapi.yaml` and the Postman collection. |
| **Lowest latency** | Pages are built from one query or one counters row, never a `COUNT(*)`; partial indexes; row-level locks only where two writers could collide. Typical reads take 2 to 3 ms. |

**Errors**

| Situation | HTTP | gRPC |
|---|---|---|
| Malformed input, missing field, bad id, bad page token | 400 | `invalid_argument` |
| An action the current state does not allow ("cannot submit an application that is already submitted", module not switched on) | 400 | `failed_precondition` |
| A value that breaks a format rule (a percentage over 100, an empty name) | 422 | `invalid_argument` |
| Not found | 404 | `not_found` |
| Already exists (a second signup requirement, a store already enrolled) | 409 | `already_exists` |
| Lost a race or state conflict (stale `version`, step with signoffs, folder not empty) | 409 | `aborted` |
| File too big / file type not allowed | 413 / 415 | `resource_exhausted` / `invalid_argument` |
| A bug or an unreachable dependency | 500 | `internal` |

---

## 4. The database in one page

21 tables are in use (the ER diagram is in [database-diagram.md](database-diagram.md)). Grouped by what they are for:

| Group | Tables | Purpose |
|---|---|---|
| The catalog | `module_definitions` | Every page a program can have, as a tree. Read-only, seeded by migrations. |
| The program | `programs`, `program_modules`, `program_stats` | The program, the modules it chose (and what each leaf is), and its Overview counters |
| Settings leaves | `program_fields`, `field_options`, `program_field_values` | A setting's definition, its manual options, its saved value |
| Rule leaves and publishing | `rule_sets`, `rule_versions`, `publications` | Versioned rules, and the published snapshots |
| Enrollment | `requirements`, `approval_steps`, `applications`, `signoffs`, `enrollments` | The application form, who approves, the applications, the decisions, the enrolled stores |
| Store discounts | `program_overrides` (plus `program_modules.config`) | The domain and store discount levels (the program level lives in the module's config) |
| Documents | `documents`, `document_folders`, `program_storage` | File metadata, folders, the page's counters (the bytes are in object storage) |
| Records of change | `audit_log`, `outbox` | History and events; written by every module; no foreign keys on purpose |

The 22nd table, `operations`, exists but nothing uses it.

---

## 5. The modules, one by one

Each module below answers: **why it exists**, **where it is in the UI**, **its APIs and the tables each one touches**, and **the rules worth remembering**.
(In the API tables, "R" is a read and "W" is a write; writes always include `audit_log` and `outbox` unless stated.)

### 5.1 Module catalog: the catalog

**Why.** The frontend and the program builder need the list of every page that exists, and how each one behaves (premade or container, which code handles it,
which API backs it). It is data, not code, so adding a page is a migration, not a deployment.

**UI.** The "pick the modules" step when creating a program.

| API | Tables |
|---|---|
| `GET /module-catalog` (`parent_id` for children, `full_tree=true` for everything nested in one response) | R `module_definitions` |
| `GET /module-catalog/{id or key}` | R `module_definitions` |

**Table `module_definitions`**: `key` (unique, like `enrollment.applications_approvals.application_panel`), `kind` (`group` or `leaf`), `parent_id`, `ancestry` (an `ltree`, so
a whole subtree is one query), `source`, `handler_key`, `api_base`, `ui_component`, `rule_family`, `requires_publish`, `selectable`, `allow_components`, `default_mode`, `default_rule`, `permission_key`.
`default_mode` (`rule` or `settings`) and `default_rule` (a starter rule) let a page start configured: today only **Fees** (Core Returns) has them, so a program that selects it gets a Rule leaf with a simple GoRules decision table already saved as draft 1.
**Code:** `domain/module_catalog_item.rs`, `postgres/module_catalog_item/`, `usecases/module_catalog_item/`.

### 5.2 Programs

**Why.** The root object. Everything else hangs off a program.

**UI.** The Programs list, and Create / Edit program. The header with the name, the Active badge, History and Create like this.

| API | Tables |
|---|---|
| `GET /programs` (`show_deleted`, paging, `total_size`) | R `programs` |
| `POST /programs` | W `programs`, `program_modules` (picked modules plus their parents, their children, and Overview), `program_stats`, `program_storage`, `document_folders` (4 system folders), and for settings or rule leaves `program_fields`, `field_options`, `rule_sets` |
| `GET /programs/{p}` | R `programs` |
| `PATCH /programs/{p}` | W `programs`; with `modules` also `program_modules` and the leaf tables |
| `DELETE /programs/{p}` | W `programs` (`state = DELETED`; the slug becomes free) |
| `POST /programs/{p}:undelete` | W `programs` |

**Rules.** Creating a program is **one transaction** that builds everything it needs, so a program is never half-made. Library-backed settings are checked against
global-settings *before* the transaction opens. `PATCH` bumps `config_version` only when `level`, `active` or the modules change; a rename or a new
description does not. `level` (`user`, `organization`, `location`) says what kind of customer the program is for; it is stored and validated but nothing reads it yet.
**Code:** `domain/program.rs`, `postgres/program/` (`program_repository.rs`, and `module_changes.rs` for the modules a create or update carries).

### 5.3 Program modules: the manifest

**Why.** The frontend cannot know in advance what a program contains. The **manifest** is the program's table of contents: which modules it has, in tree order, what
each is (`premade` page, `rule` page, `settings` page), which handler renders it, which API backs it, and for settings pages the settings with their saved values.

**UI.** The menu and tabs of a program page, and which page opens for each item.

| API | Tables |
|---|---|
| `GET /programs/{p}/modules` (`enabled_only`, `include_options`) | R `program_modules`, `module_definitions`, `program_fields`, `field_options`, `program_field_values`, `rule_sets`; library options come from the global-settings cache |

**Rules.** Always returned whole (about 70 entries), parents before children. Switching a module off hides it (`enabled = false`); nothing is deleted.
**Table `program_modules`** (key `(program_id, module_id)`): `enabled`, `mode` (`rule` or `settings` for a container leaf, null otherwise), `position`, `config`, `version`.
**Code:** `domain/program_module.rs`, `postgres/program_module/` (`fetch_modules` builds the whole manifest).

### 5.4 The builder: settings and options

**Why.** On a container page the admin decides what it is. A **Settings** page holds typed settings: boolean, number, text, radio, checkbox, dropdown. A dropdown's
options come from **one** source only: a global-settings **library** (loaded dynamically and kept up to date) or **manual** options typed in. Never mixed.

**UI.** Create program and Edit program: "add a setting". Then the page itself under Configure.

**API.** There is no separate endpoint: settings travel inside `POST /programs` and `PATCH /programs/{p}` as `modules: [{key, mode, enabled, settings: [...]}]` (a plain
string key just switches a premade module on).

**Rules.** The settings list **replaces** the page's settings: a setting with an `id` is updated, one without is created, one left out is soft-deleted. Options are matched by `code`
and removed ones are deactivated. A setting's `key` is generated from its name; its type cannot change once values exist.
**Tables:** `program_fields` (`name`, `key`, `field_type`, `options_source`, `library_id`, `required`, `default_value`, `position`, `state`), `field_options` (`code`, `label`, `active`).
**Code:** `postgres/program/module_changes.rs` (`replace_settings`, `sync_manual_options`), `domain/program_setting.rs`, `application/services/library_options.rs`.

**The global-settings connection.** Library options are read from global-settings and **cached in memory** (30 seconds, then served stale while refreshed in the background),
warmed at start-up. There is no webhook from global-settings, so a changed option shows up within about 30 seconds (the first read after that still shows the old copy while the new one loads). If global-settings is down the page still loads, with a `library_error` marker.
**Code:** `adapters/services/global_settings/global_settings_client.rs`, `domain/library_catalog.rs`.

### 5.5 Setting values

**Why.** A setting is a *question*; its value is the *answer* the admin saves. They change at different times and for different reasons, so they are stored apart.

**UI.** The Configure pages of type Settings: "N unsaved changes" then Save.

| API | Tables |
|---|---|
| `GET /programs/{p}/setting-values[?module=key]` | R `program_field_values`, `program_fields`, `programs` (the version) |
| `POST /programs/{p}/setting-values:batchUpdate` `{version?, values: {setting_id: {value or option_ids} or null}}` | W `program_field_values`, `programs` (version bump) |

**Rules.** The whole page saves in **one request and one transaction**. Every value is checked first against its type and options, and *all* problems are reported together.
`version` (the `config_version` you last read) makes a stale save a 409, so two admins cannot overwrite each other. Only values that differ are written. A boolean takes
`value`; radio, checkbox and dropdown take `option_ids`; a required setting cannot be cleared.
**Table `program_field_values`** (key `(program_id, field_id, scope_type, scope_id)`): `value` (JSON), `option_ids` (uuid[]), `library_id`.
**Code:** `domain/program_setting_value.rs`, `postgres/program_setting_value/`.

### 5.6 Overview and the audit log

**Why.** The first thing an admin sees: how the program is doing, and what changed.

**UI.** The Overview tab, and the History button.

| API | Tables |
|---|---|
| `GET /programs/{p}/overview` | R (one query) `programs`, `program_stats`, `applications` (counts by status), `rule_sets` and `program_modules`, `publications`, `audit_log` |
| `GET /programs/{p}/audit-logs` (`action=a,b`, paging) | R `audit_log` |

**What the Overview returns:** the program; `stats` (`total_applications`, `pending_approvals`, `active_enrollments`, `enrollments_total`); the `funnel` (all six application statuses,
zero-filled); `published_rules` (each rule leaf's state, `in_sync` count, `drift_detected`); `config_version`, `published_config_version` and `unpublished_changes`; and the latest 10 activities.
**Rules.** `unpublished_changes` is true when the program changed since it was last published (or changed since creation and was never published). A rule leaf's state is `empty`
(nothing saved, not drift), `never`, `drifted` or `in_sync`. The audit log is newest first, paged by time, with the `before` and `after` of each change; `summary` is the line shown in the list.
**Code:** `domain/program_overview.rs`, `postgres/program_overview/`, `postgres/audit_log/`.

### 5.6a Notifications (premade page in Settings, picked in Create program)

**Why.** Each trigger point (Enrollment Approved, Payment Due, ...) tells the customer through some channels (Email, SMS, Push, In-App). The grid is the same shape on every program, so it is a premade page, not a form of settings.

**UI.** The Notifications card: the grid, **Add trigger** (key + label), the bin icon, and the page's **Save changes**. The program has it when it is picked in the Create program module list; it then comes back in `GET /programs/{p}/modules` as a `premade` module with `api_base notifications`, so the page loads with the rest of Settings.

| API | Tables |
|---|---|
| `GET /programs/{p}/notifications` | R `programs.config_version`, `program_notification_triggers` |
| `PUT /programs/{p}/notifications` | W `program_notification_triggers`, `programs.config_version`, `audit_log` (`notifications_updated`), `outbox` |

**Rules.** A new program starts with 8 usual trigger points (seeded once, when the page arrives). `PUT` sends the whole grid: missing triggers are removed, new keys added, list order = row order. Key: lower-case letters, digits, `_`, starts with a letter, unique; label 1-100 characters; at most 100 triggers. `version` (the program's `config_version`) is checked like the setting values (409 when stale; omit = last write wins). Saving the grid unchanged writes nothing. 404 on a program that did not pick the page. **Code:** `domain/notifications.rs`, `postgres/notifications/`.

### 5.6b Danger zone (premade page in Settings, on every program)

**Why.** An admin needs to take a program out of use without losing it (Archive), and to remove one made by mistake (Delete).

**UI.** The Danger zone card at the bottom of Settings, with **Archive program** and **Delete program**. It is on every program without being selected.

| API | Tables |
|---|---|
| `GET /programs/{p}/danger-zone` | R `programs`, `enrollments`, `applications`, `publications`, `documents` (counts) |
| `POST /programs/{p}/danger-zone:archive` / `:unarchive` | W `programs.archive_time`, `audit_log`, `outbox` |
| `POST /programs/{p}/danger-zone:deletePermanently` | D `programs` (cascade); `audit_log` entry stays |

**Rules.** Archive is reversible and keeps everything (`state` stays ACTIVE); the program list leaves archived programs out (`GET /programs?show_archived=true` lists them). Delete permanently
is allowed only with no operational history (no enrollments, applications, publications or documents); otherwise **409**, and the card's `delete_blockers` say what is in the way. `DELETE /programs/{p}`
is still the reversible soft delete. **Code:** `domain/danger_zone.rs`, `postgres/danger_zone/`.

### 5.7 Requirements

> **Parked.** Switched off by default (`features.enrollment_review`); nothing is deleted. See [parked-enrollment-review.md](parked-enrollment-review.md).

**Why.** What a store must provide to enroll. The ordered list *is* the program's application form: sign up, card on file, credit application, contract, and any number of custom fields.

**UI.** Configure → Enrollment Requirements.

| API | Tables |
|---|---|
| `GET /programs/{p}/requirements` | R `requirements` |
| `POST /programs/{p}/requirements` `{type, name, config?, position?}` | W `requirements`, `programs` (lock + version bump) |
| `GET/PATCH/DELETE /programs/{p}/requirements/{id}` | R/W `requirements`, `programs` |
| `POST /programs/{p}/requirements:batchUpdate` `{requirements: [{id, position}]}` (reorder) | W `requirements`, `programs` |

**Rules.** One each of `signup`, `card_on_file`, `credit_application`, `contract` per program (a unique index; a second is 409); `custom_field` repeats. `type` never changes. Delete is for good.
**Table `requirements`**: `type`, `name`, `config` (JSON, up to 16 KB), `position`. **Code:** `domain/requirement.rs`, `postgres/requirement/`.

### 5.8 Approval steps

> **Parked.** Switched off by default (`features.enrollment_review`); nothing is deleted. See [parked-enrollment-review.md](parked-enrollment-review.md).

**Why.** The ordered chain of sign-offs an application goes through, for example "Credit review" by `credit_manager`, then "Final approval" by `admin`.

**UI.** The configuration behind Applications & Approvals (your designs have no separate screen for it).

| API | Tables |
|---|---|
| `GET/POST /programs/{p}/approval-steps`, `GET/PATCH/DELETE .../{id}`, `POST .../approval-steps:batchUpdate` | R/W `approval_steps`, `programs` (lock + version bump) |

**Rules.** A step that has recorded signoffs cannot be removed (409): `signoffs.approval_step_id` is `ON DELETE RESTRICT` and the API turns that into a clear message.
`approver_role` is stored and shown but **not enforced yet** (no authentication).
**Table `approval_steps`**: `name`, `approver_role`, `position`. **Code:** `domain/approval_step.rs`, `postgres/approval_step/`.

### 5.9 Applications

> **Parked.** Switched off by default (`features.enrollment_review`); nothing is deleted. See [parked-enrollment-review.md](parked-enrollment-review.md).

**Why.** A store's request to join a program. The page reads only the `applications` table, so it is fast.

**UI.** Operate → Applications: status chips (All, Submitted, In review, Approved, Rejected, Draft, Withdrawn), the table, and the Actions column.

| API | Tables |
|---|---|
| `GET /programs/{p}/applications` (`status=a,b`, paging) | R `applications`, `approval_steps` (the step name) |
| `POST /programs/{p}/applications` `{store_id, website_id, store_name?, website_name?, answers?}` | W `applications` (a draft), `program_stats` (total +1) |
| `GET /programs/{p}/applications/{id}` | R `applications` (with `answers` and `requirements_snapshot`) |
| `POST .../applications/{id}:submit` | W `applications` (`submitted`, first step, frozen requirements), `program_stats` (pending +1) |
| `POST .../applications/{id}:withdraw` | W `applications` (`withdrawn`), `program_stats` (pending −1 if it was pending) |

**Rules.** A store can have **one open** application (draft, submitted or in review) per program and website: a partial unique index, so two simultaneous requests cannot both win. Submit
freezes the program's current requirements into `requirements_snapshot` and places the application at the first approval step. A move the status does not allow is a 400. Each row
carries `actions`: what can still be done. Applications are data, not configuration: they never bump `config_version` and do not lock the program.
**Table `applications`**: `store_id`, `store_name`, `website_id`, `website_name`, `status`, `current_step_id`, `requirements_snapshot`, `answers`, `submitted_time`, `decided_time`.
**Code:** `domain/application.rs`, `postgres/application/`.

### 5.10 Signoffs: approve and reject

> **Parked.** Switched off by default (`features.enrollment_review`); nothing is deleted. See [parked-enrollment-review.md](parked-enrollment-review.md).

**Why.** An approver's decision on the step an application is waiting at. It moves the application along, and keeps the decision history.

**UI.** The Approve and Reject buttons in the Applications Actions column.

| API | Tables |
|---|---|
| `POST .../applications/{id}/signoffs` `{decision: approved or rejected, comment?, approval_step_id?}` | W `signoffs`, `applications`, `program_stats` (pending −1 on a final decision), and **on the last approval `enrollments`** and `program_stats` (enrollments +1, active +1) |
| `GET .../applications/{id}/signoffs`, `GET .../signoffs/{id}` | R `signoffs`, `approval_steps` |

**Rules.** `approved` with a next step puts the application in review at that step; approving the **last** step approves it and **enrolls the store in the same transaction**; `rejected`
rejects it at any step. Only a submitted or in-review application waiting at a step can be decided. The application row is locked, so two decisions take turns; `approval_step_id`
(the step the UI showed) makes a double click a 409. One decision per step (a unique key). **Not enforced yet:** that the approver has the step's `approver_role`.
**Table `signoffs`**: `application_id`, `approval_step_id`, `decision`, `comment`, `actor`. **Code:** `domain/signoff.rs` (`outcome()` is the decision logic), `postgres/signoff/`.

### 5.11 Enrollments

**Why.** A store enrolled in a program on one website: the end of the road for an approved application, and the list pricing will read.

**UI.** Operate → Stores: the "All websites" filter, "N STORES", Enroll store, Bulk enroll, the status dropdown.

| API | Tables |
|---|---|
| `GET /programs/{p}/enrollments` (`website_id`, `status=a,b`, `store` prefix, paging, `total_size`) | R `enrollments` |
| `POST /programs/{p}/enrollments` | W `enrollments`, `program_stats` |
| `POST /programs/{p}/enrollments:batchCreate` (up to 500, an outcome per item) | W `enrollments` (one statement), `program_stats` |
| `GET/PATCH /programs/{p}/enrollments/{id}` (status, customer group) | R/W `enrollments`, `program_stats` |

**Rules.** A store is enrolled **automatically** when its application's last step is approved, or by an admin directly, or in bulk. One live enrollment (pending, active or suspended) per store,
program and website (a unique index; a second is 409; after `ended` the store can be enrolled again). Status moves: `pending → active`, `active ⇄ suspended`, anything `→ ended` (final).
`active_enrollments` and `enrollments_total` move in the same transaction. Names are supplied by the caller (no storefront lookup yet), so a nameless store shows as "Unresolved store".
**Table `enrollments`**: `application_id`, `store_id`, `store_name`, `store_resolved`, `website_id`, `website_name`, `customer_group`, `status`, `enrolled_time`, `contracts_count`, `last_contract_time`.
**Code:** `domain/enrollment.rs`, `postgres/enrollment/` (also `enroll_after_approval`, called by signoffs).

### 5.12 Store discounts

**Why.** Each enrolled store's effective discount, and where it came from. Three levels, lowest to highest priority: **Program → Domain (website) → Store**.
**Every field inherits on its own** (`enabled`, `type` percentage or fixed, `amount`, `currency_code`): a store can set only the amount and still take `enabled` from the program.

**UI.** Operate → Store discounts: the table (Store, Website, Resolved Discount, Set At), the edit panel with a badge and a revert button per field, "Revert all to inherited", and Mass edit.

| API | Tables |
|---|---|
| `GET /programs/{p}/store-discounts` (`website_id`, `store` prefix, paging, `total_size`) | R `enrollments`, `program_overrides`, `program_modules` (the program level) |
| `GET /programs/{p}/store-discounts/{enrollment}` | R the same, one store |
| `PATCH /programs/{p}/store-discounts/{enrollment}` `{discount: {...} or null}` | W `program_overrides` (the store level), `programs` (version bump) |
| `POST /programs/{p}/store-discounts:batchUpdate` `{ids, discount}` (up to 500, one statement, one version bump) | W `program_overrides`, `programs` |
| `GET /programs/{p}/store-discount-levels` | R `program_modules` (the program level), `program_overrides` (the domains) |
| `PATCH .../store-discount-levels/program`, `PATCH .../store-discount-levels/domains/{website_id}` | W `program_modules.config` or `program_overrides`, `programs` |

**Rules.** In a PATCH: a field you send is set, a field sent as `null` is reverted to the inherited value, `discount: null` reverts everything, and a body without `discount` is refused.
A percentage cannot pass 100 (422), an amount has at most 2 decimals, and a mass edit that would leave any store invalid is refused whole. The program level is stored in the module's own
`program_modules.config` under `discount`; domain and store levels are `program_overrides` rows (`scope_id` is the website id, or the enrollment id). The program must have the module on.
**Code:** `domain/store_discount.rs` (`resolve()` is the heart), `postgres/store_discount/`.

### 5.13 Rule sets, versions and publishing

**Why.** The rule engine is a frontend component, so the backend only **keeps what it saves, versioned**, and **publishes**. The core rule: **a draft is not live**; only Publish changes what the outside world sees.

**UI.** Catalog & Pricing → Products & Rules (Pre-Enrollment, Post-Enrollment, Warranty), Pricing → Discount Rules and Markup, and any container page set to Rule. Publishing: the program header and the Overview's "Published rules" card.

| API | Tables |
|---|---|
| `GET /programs/{p}/rule-sets` | R `program_modules`, `module_definitions`, `rule_sets` |
| `GET /programs/{p}/rule-sets/{module_key}` (newest version in full) | R `rule_sets`, `rule_versions` |
| `POST .../rule-sets/{module_key}/versions` (save the draft) | W `rule_versions`, `rule_sets`, `programs` (version bump) |
| `GET .../versions`, `GET .../versions/{n}` | R `rule_versions` |
| `POST /programs/{p}/publications` (**Publish**) | W `rule_versions`, `rule_sets`, `publications`; R requirements, steps, modules, values, overrides, enrollments |
| `GET /programs/{p}/publications`, `GET .../publications/{id}` (with the bundle) | R `publications` |

**Rules.** A save is an immutable numbered version; the same definition again creates nothing; an incomplete draft can be saved but blocks publishing. Publish runs in one transaction
**under the program lock**: it refuses if nothing is new or a draft is invalid, promotes each leaf's draft (the old one becomes `superseded`), builds the **bundle** (program, enabled modules with
settings and values, requirements, approval steps, the store → discount map of active stores, the published rules) and stores it append-only. It records the `config_version` it covers
without changing it.
**Tables:** `rule_sets` (`draft_version`, `published_version`), `rule_versions` (`definition`, `compiled`, `valid`, `status`, `content_hash`), `publications` (`config_version`, `bundle`).
**Code:** `domain/rule_set.rs`, `domain/publication.rs`, `postgres/rule_set/`, `postgres/publication/`.

### 5.14 Documents

**Why.** A file manager per program. Postgres keeps only the **metadata**; the **bytes live in object storage** (S3 or MinIO) and never pass through this service.

**UI.** Configure → Documents: locations (All files, Recent, Needs review, Quarantined), folders, search, Upload, the storage line.

| API | Tables / storage |
|---|---|
| `GET /programs/{p}/documents:summary` | R `program_storage`, `document_folders` |
| `GET /programs/{p}/documents` (`location`, `folder_id`, `q` prefix, paging) | R `documents` |
| `POST /programs/{p}/documents` (**start an upload**) | W `documents` (a hidden `uploading` row); signs a PUT URL |
| (the browser PUTs the bytes to the signed URL) | the object storage |
| `POST .../documents/{id}:complete` (**finish**) | asks the storage what arrived; W `documents`, `program_storage`, `document_folders` |
| `GET/PATCH/DELETE .../documents/{id}` (rename, move, soft delete) | R/W `documents`, `document_folders`, `program_storage` |
| `POST .../documents/{id}:download` | R `documents`; signs a GET URL |
| `POST .../documents/{id}:review` | W `documents`, `program_storage` |
| `POST /webhooks/document-scan` (the scanner's verdict) | W `documents`, `program_storage` |
| `GET/POST .../document-folders`, `PATCH/DELETE .../document-folders/{id}` | R/W `document_folders` |

**Rules.** The signed URL is for one exact object key, expires in 10 minutes, and has the **type, size and SHA-256 signed in**, so the storage itself refuses a different file. The key is generated by the
server, so a name can never choose a path. `:complete` asks the storage what really arrived and compares. Nothing is `ok` until scanned; a quarantined file cannot be downloaded; only a file that
needs review can be reviewed. 25 MB and 7 types by default (413 / 415), 10,000 files per program. The four system folders cannot be renamed or deleted. Counters move in the same transaction as every
change. Files are data: no `config_version` bump. The program must have the module on.
**Tables:** `documents` (`status`, `file_key`, `checksum`, `state`...), `document_folders` (`key`, `system`, `document_count`), `program_storage` (`file_count`, `total_bytes`, `needs_review_count`, `quarantined_count`).
**Code:** `domain/document.rs`, `domain/object_storage.rs`, `adapters/services/object_storage/s3_client.rs` (AWS Signature V4 presigning, tested against AWS's example), `postgres/document/`.

---

## 6. How the modules work together: one program's life

1. **Build.** The admin creates a program: `POST /programs` with the modules they want. One transaction creates the program, its modules, a settings page's settings, a rule page's empty rule set, the Overview counters, the Documents counters and four system folders. *(Programs, Program modules, Builder.)*
2. **Configure.** They save setting values (`setting-values:batchUpdate`), add requirements and approval steps, set the program and website discounts, and build rules in the rule builder. Every change bumps `config_version`, so the Overview shows **unpublished changes**. *(Setting values, Requirements, Approval steps, Store discounts, Rule sets.)*
3. **Publish.** `POST /publications` freezes all of that into a bundle. The Overview shows **in sync**. *(Publishing.)*
4. **A store applies.** `POST /applications`, then `:submit`: the requirements are frozen into the application and it waits at the first approval step. The Overview's pending count goes up. *(Applications.)*
5. **Approvers decide.** Each `POST /signoffs` moves the application on; the last approval approves it and **enrolls the store in the same transaction**. *(Signoffs, Enrollments.)*
6. **Run.** Admins change a store's status or customer group, bulk enroll stores, and adjust a store's discount (which raises "unpublished changes" again, to be published). *(Enrollments, Store discounts.)*
7. **Keep files.** Contracts and other documents are uploaded to the storage through signed URLs, scanned, reviewed and downloaded. *(Documents.)*
8. **Watch.** The Overview shows the funnel and counters; History shows every change with who and when. *(Overview, Audit log.)*

---

## 7. What changes `config_version`, and who owns each counter

**Bumps `config_version`** (the "unpublished changes" signal): changing a program's `level`, `active` flag or modules; saving setting values that changed; any change to requirements or approval steps; any store-discount edit (store, domain or program level); saving a rule version.
**Does not bump it:** renaming a program or changing its description; applications, signoffs, enrollments and documents (data, not configuration); publishing (it records the version).

| Counter | Table and column | Who moves it |
|---|---|---|
| Applications | `program_stats.total_applications` | Applications: create (+1) |
| Pending approvals | `program_stats.pending_approvals` | Applications: submit (+1), withdraw (−1 if pending); Signoffs: a final decision (−1) |
| Enrollments | `program_stats.enrollments_total`, `active_enrollments` | Enrollments (create, bulk, status change) and Signoffs (auto-enroll) |
| Files | `program_storage.file_count`, `total_bytes`, `needs_review_count`, `quarantined_count` | Documents: complete, scan, review, delete |
| Files per folder | `document_folders.document_count` | Documents: complete, move, delete |

---

## 8. Running and testing it

| What | How |
|---|---|
| Database | Postgres in the `ms-rust-global-settings-postgres` container, database `programs`. Apply `migrations/*.up.sql` in order (idempotent). |
| HTTP server | `APP__HTTP_PORT=8090 APP__CORS_ALLOWED_ORIGIN=http://localhost:8082 cargo run --bin programs_http` |
| gRPC server | `APP__GRPC_PORT=50061 cargo run --bin programs_grpc` (reflection is on, so Postman and `buf curl` can list the methods) |
| global-settings (library options) | Must be running on port 8000 for the library-backed tests |
| Object storage (Documents) | `docker compose up -d minio` (S3 API 9000, console 9001; the bucket is created for you) |
| Swagger | `docker compose up -d swagger`, then http://localhost:8082 |
| Unit tests | `cargo test --lib` (86 tests) |
| Proto | `buf lint proto`, then `buf generate proto --path proto/programs/v1`, then `buf build proto --path proto/programs/v1 -o src/adapters/generated/grpc_services/programs.binpb`. Never run a plain `buf generate`: it would read the remote repository. |
| API tests | Import `postman/programs.postman_collection.json` and the environment file. `newman run` on the main collection runs 900 assertions with the parked modules off; with `APP__FEATURES__ENROLLMENT_REVIEW=true`, also run `postman/enrollment-review.postman_collection.json` (327 assertions). Each module has a folder and a "negative checks" folder; the negative folders set up their own data, so any request can be sent alone. |
| Parked modules | Requirements, Approval steps, Applications and Signoffs are off by default: `APP__FEATURES__ENROLLMENT_REVIEW=true` turns them on. See [parked-enrollment-review.md](parked-enrollment-review.md). |
| OpenAPI | `spectral lint openapi.yaml` (one accepted false positive on the Overview's operation id) |

---

## 9. What is not built

| Item | Status |
|---|---|
| Imports (Upload Consignment Orders, Import History) | **Dropped**, and removed from the module catalog. The `import_jobs` API is not planned. |
| Contracts and agreements | Parked. `enrollments.contracts_count` and `last_contract_time` stay empty. |
| Reports (Enrollment Reports Panel) | **Dropped**, and removed from the module catalog. |
| Custom CSV rules, manual products, rule test cases, simulating a rule set | Parked (they belong to the rule engine). |
| Anything that **reads** the published bundle (pricing) | Not part of this service yet. |
| Authentication | Approver roles, reviewers and the scanner webhook are not authenticated beyond an optional shared secret. |
| Documents cleanup | No job removes uploads stuck in `uploading`; a deleted file's object stays in the storage (use a bucket retention rule). The scanner itself is not built, only its callback. |
| The module-on check for the older modules | Requirements, Approval steps, Applications, Signoffs, Enrollments, Rule sets and Publications work for any program. |
| Load test | Single requests take 2 to 3 ms; nothing has been tested at volume. |
| Placeholders in `openapi.yaml` | **Removed.** The 64 unserved operations (blueprints, module fields and records, program domains, agreement templates and contracts, import jobs, SKU prices, core return fees (the page is now Fees, a rule page), report definitions and runs, operations) were dropped from the spec because the new design does not use them. `openapi.yaml` now lists exactly the operations served by default (48); the 20 of Requirements, Approval steps, Applications and Signoffs are in `openapi.enrollment-review.yaml`. If a page such as Price Import or Core Return Fees later needs an API, it is added back as a real, built endpoint. |

---

## 10. Glossary

| Term | Meaning |
|---|---|
| Program | An offering a store can join (KBB, CPO, Credit Line) |
| Module | A page of the admin screen; a program has the ones its admin picked |
| Leaf, group | A leaf is a real page; a group is a menu heading with children |
| Premade, container, default | Kinds of module: written in code, built by the admin, on every program |
| Manifest | A program's list of modules (`GET .../modules`): its table of contents |
| Setting, value | The definition of a field, and the answer saved for it |
| Rule leaf, rule set, version | A page that holds rules; its record; one numbered save |
| Draft, published, superseded | The newest save; the live version; an older replaced one |
| Bundle, publication | The frozen snapshot of a program, and the record that stores it |
| `config_version` | A counter of configuration changes; the base of "unpublished changes" |
| Enrollment | A store enrolled in a program on one website |
| Signoff | One approver's decision on one approval step |
| Presigned URL | A short-lived, signed link the browser uses to talk to the object storage directly |
| Outbox | A table of events written with every change, for other services to pick up |
| Keyset paging | Paging by "everything after this id", so every page costs the same |
| AEP | The API style used: resource paths, `:custom` methods, standard errors |
