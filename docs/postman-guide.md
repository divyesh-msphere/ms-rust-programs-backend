# Testing the Programs API in Postman

What exists today: **Programs** (create, get, list, update, delete, undelete), the **module manifest**
(`GET /programs/{program}/modules`) and the **module registry** (`/module-catalog`). Everything is on **HTTP** and
**gRPC**, and `openapi.yaml` describes the HTTP side.

## 1. Start the servers

Postgres runs in your existing container (`ms-rust-global-settings-postgres`); the service uses its own database,
`programs`, so the `global-settings` data in `test` is never touched.

```bash
# once: create the database and apply the migrations (safe to re-run)
for f in $(ls migrations/*.up.sql | sort); do psql -d programs -f "$f"; done

# HTTP (8080 is often taken, so use 8090) and gRPC (50051 is often taken, so use 50061)
APP__HTTP_PORT=8090  cargo run --bin programs_http
APP__GRPC_PORT=50061 cargo run --bin programs_grpc
```

Check: `curl localhost:8090/health` → `{"status":"ok","service":"programs",...}`.

**Documents also need object storage** (the file bytes; Postgres keeps only the metadata). Start the S3-compatible MinIO from the
compose file, which also creates the bucket:

```bash
docker compose up -d minio      # S3 API http://localhost:9000, console http://localhost:9001 (programs / programs-local-secret)
```

The service reads it from the `[storage]` section of `src/config/config.toml` (override with `APP__STORAGE__ENDPOINT`, ...). The collection's
**Documents** folders upload real files to it.

## 2. Import into Postman (HTTP)

1. **Import** → both files from the [postman/](../postman/) folder:
   - `programs.postman_collection.json`
   - `programs.local.postman_environment.json`
   - `enrollment-review.postman_collection.json` (Requirements, Approval steps, Applications, Signoffs; see below)
2. Top right, choose the environment **Programs local**. Its variables: `base_url` = `http://localhost:8090`,
   `grpc_host` = `localhost:50061`.
3. Run the collection (**Run** → **Run Programs API (local)**), or send requests one by one, **top to bottom**.

**Two collections.** Requirements, Approval steps, Applications and Signoffs are switched off by default, so their requests live in
`enrollment-review.postman_collection.json`. Run the main collection with the server as it starts (switch off): the **Parked modules
(switched off)** folder checks that those paths answer 404 and the two pages cannot be selected. To run the other collection, start the
server with `APP__FEATURES__ENROLLMENT_REVIEW=true` first. Run each collection against the matching server; the main one against a
server with the switch on would fail its "Parked modules" folder. See [parked-enrollment-review.md](parked-enrollment-review.md).

The collection generates a fresh slug (`pm-<time>`) on the first run, keeps it in the `slug` and `program_id`
variables, and **deletes its test program at the end**, so you can run it again and again. Expected result:
**91 requests, 186 checks, 0 failed.**

| Folder | What it shows |
|---|---|
| **Programs** | create → list → get by slug / by id → module manifest → update → delete → list deleted → undelete |
| **Settings and rules** | build a program with a **rule** leaf and a **settings** leaf (every type), read it back, replace its settings, change modes, switch modules off, add a new module |
| **Setting values** | save the values admins enter (every type, in one request), read them back, the version check, clearing, and options that disappear |
| **Programs - negative checks** | duplicate slug 409, bad slug / level / module / name 422, unknown program 404, undelete of a live program 400 |
| **Settings and rules - negative checks** | 21 rejected payloads (bad type, library + own options, unknown library, duplicate names, wrong default, ...) |
| **Setting values - negative checks** | 14 rejected saves (wrong type, wrong option, required cleared, unknown setting, ...) |

The collection covers the **Programs resource only**. (The module registry, `/module-catalog`, is not part of it.)

Every request has **Tests** (the checks), so a red cross tells you exactly which rule broke.

## 3. How the responses look

**Create a program** — `POST /programs` (header `X-User-Id: divyesh` is recorded in the audit log; without it the actor is `system`):

```json
{
  "name": "KBB",
  "slug": "kbb",
  "description": "Kelley-Blue-Book style core return handling",
  "level": "location",
  "modules": [
    "enrollment.applications_approvals.application_panel",
    "enrollment.stores.enrolled_stores_control_panel"
  ]
}
```
```json
{
  "path": "programs/01a14367-dcfc-7215-bba7-b737e49d5a0e",
  "id": "01a14367-dcfc-7215-bba7-b737e49d5a0e",
  "name": "KBB", "slug": "kbb", "description": "Kelley-Blue-Book style core return handling",
  "level": "location", "active": true,
  "config_version": 1, "config_hash": "2ea056ce…",
  "create_time": "2026-10-16T06:30:42.684714", "update_time": "2026-10-16T06:30:42.684714",
  "state": "ACTIVE",
  "modules": [ …the manifest below… ]
}
```
Times are UTC without an offset, as everywhere in this API.

**The module manifest** — what the frontend uses to draw the program (`GET /programs/kbb/modules`). Parents come before
their children, so a client builds the tree in one pass:

| key | kind | source | handler_key | api_base |
|---|---|---|---|---|
| `overview` | leaf | **default** | overview | overview |
| `enrollment` | group | container | – | – |
| `enrollment.stores` | group | container | – | – |
| `enrollment.applications_approvals` | group | container | – | – |
| `enrollment.applications_approvals.application_panel` | leaf | **premade** | applications | `applications` |
| `enrollment.stores.enrolled_stores_control_panel` | leaf | **premade** | enrolled_stores | `enrollments` |

- **`source: premade`** → the frontend calls `api_base` (e.g. `/programs/kbb/applications`) for that screen.
- **`source: container`** → the admin chooses what the leaf **is**: a **Rule** (`mode: "rule"`) or **Custom settings**
  (`mode: "settings"`, then they add settings with a name and a type). Until they choose, `mode` is `null`. The leaf's own
  name ("Pre Enrollment") is fixed by the module, so nothing needs naming. See section 5.
- **`source: default`** → Overview, on every program automatically. The Danger zone page is `premade` and also on every program (`universal`).

**An error** (RFC 9457 style):
```json
{ "type": "UNPROCESSABLE_ENTITY", "status": 422, "title": "Unprocessable Entity",
  "detail": "slug \"Bad Slug\" must be 1-64 lowercase letters, digits and single hyphens", "instance": "26c49b90-…" }
```

## 4. How to use it properly

**Choosing modules at creation.** `modules` takes module **keys**. List them with `GET /module-catalog?full_tree=true`.
You get the **premade** modules (the admin selects these), plus any group or leaf:

| Premade module (key) | Screen |
|---|---|
| `enrollment.applications_approvals.application_panel` | Applications & Approvals |
| `enrollment.stores.enrolled_stores_control_panel` | Enrolled Stores |
| `enrollment.requirements.enrollment_requirements` | Enrollment Requirements |
| `catalog_pricing.store_discounts.control_panel` | Store Discounts |
| `document_agreement.documents.documents_panel` | Documents |

Rules the server applies for you: the **ancestors** of a picked module are added (so the tree is connected); picking a
**group** adds everything under it; **Overview** is always added; an unknown key (or `overview`) is rejected with `422`.

**Addressing a program.** `{program}` takes the **slug or the id**: `/programs/kbb` and `/programs/01a14367-…` are the same.
A slug is 1–64 lowercase letters, digits and single hyphens, is unique among live programs, **cannot be changed**, and
must not look like a UUID.

**Updating.** `PATCH /programs/kbb` with `name`, `description` (`null` clears it), `level`, `active`. Changing `level` or
`active` bumps `config_version`; a rename does not. A `slug` in the body is ignored.

**Deleting.** `DELETE` is a soft delete (`state: DELETED`, `204`); the slug becomes free. `GET /programs?show_deleted=true`
lists the deleted ones, and `POST /programs/{id}:undelete` brings one back (`409` if its slug was taken meanwhile).

**Paging.** `GET /programs?max_page_size=20` → `next_page_token`; send it back as `page_token`.

## 5. What the admin builds on a leaf

A configurable leaf (for example **Pre Enrollment**, **Warranty**) has a fixed name. The admin only chooses what it **is**:

- **`"mode": "rule"`**: the leaf is the rule engine. The backend defines no rule types: the frontend renders the rule-engine
  component, and the manifest gives it the leaf's `rule` summary.
- **`"mode": "settings"`**: the admin adds settings, each with a **name** and a **type**.

| Setting `type` | The frontend draws (`render`) | Options |
|---|---|---|
| `boolean` | a **Yes / No** switch (`yes_no`) | none |
| `number` | a number box (`number_input`) | none |
| `text` | a text box (`text_input`) | none |
| `radio` | radio buttons (`radio_group`) | the admin's own, **or** a library |
| `checkbox` | check boxes (`checkbox_group`) | the admin's own, **or** a library |
| `dropdown` | a dropdown (`dropdown`) | the admin's own, **or** a library |

For radio, checkbox and dropdown there are exactly two sources, never mixed: **`options`** (the values the admin types) **or**
**`library_id`** (a global-settings library, which is then the only source).

**Create** and **update** use the same `modules` entries:

```jsonc
POST /programs
{
  "name": "KBB", "slug": "kbb",
  "modules": [
    "enrollment.stores.enrolled_stores_control_panel",                      // a plain key still works
    { "key": "catalog_pricing.products_rules.pre_enrollment", "mode": "rule" },
    { "key": "catalog_pricing.products_rules.warranty", "mode": "settings",
      "settings": [
        { "name": "Warranty enabled", "type": "boolean", "default": true },
        { "name": "Max days",         "type": "number",  "required": true, "default": 90 },
        { "name": "Grade",            "type": "radio",   "options": [ {"label": "A"}, {"label": "B"} ] },
        { "name": "Coverage",         "type": "dropdown", "library_id": "<global-settings library id>" }
      ] }
  ]
}
```

**`PATCH /programs/{program}`** takes `modules` too, and applies it in one transaction:

| Entry | Effect |
|---|---|
| a key the program does not have | adds that module (with its parent groups) |
| `"mode": "rule"` / `"settings"` | sets what the leaf is |
| `"settings": [ ... ]` | **replaces** the module's settings: an item with an `id` is kept and updated (same key, same saved values), one without is created, the rest are removed; options are matched by `code` (a removed option is deactivated, never deleted) |
| `"settings": []` | removes them all |
| `"enabled": false` | switches the module off (its data is kept) |
| `settings` left out | leaves the settings alone |

The response includes the full manifest whenever `modules` was sent. Rules the server enforces: only configurable leaves take a
mode; a premade module can only use settings; a leaf that still has settings (or a rule with versions) cannot change mode; a
setting with saved values cannot change type; defaults must match the type; at most 50 settings per module and 100 options per setting.
A rejected request saves nothing.

**Library options** (`library_id`) are read from global-settings and kept in memory: a read costs about **2 ms** with the options
included. They stay current by refreshing in the background: a copy older than 30 seconds is still served, and re-read, so a new or renamed
option shows up within about 30 seconds (there is no webhook from global-settings). If global-settings is down, the page still loads and the affected setting reports
`"library_error": "unavailable"`. To make it work locally set `[global_settings] base_url` (default `http://localhost:8000`) and run
global-settings there. The Postman variable `library_id` must be a library that exists there.

### Saving the values

The settings above only DEFINE the fields. The values admins enter are saved separately, in one request per page:

```jsonc
POST /programs/kbb/setting-values:batchUpdate
{
  "version": 3,                                     // the config_version you last read (optional)
  "values": {
    "<setting id>": { "value": true },              // boolean / number / text
    "<setting id>": { "value": 90 },
    "<setting id>": { "option_ids": ["<option id>"] },   // radio / dropdown (one) / checkbox (a list)
    "<setting id>": null                            // clear
  }
}
```
→ `{ "version": 4, "changed": 3, "results": [ ... every saved value ... ] }`

- **One request, one transaction.** Every value is checked first and ALL the problems are reported together; if any value is wrong,
  nothing is saved.
- **Checks:** a boolean is true/false, a number a number, text text; a radio or dropdown takes ONE option and a checkbox a list; every
  chosen option must be one of the setting's own active options, or (library-backed) one of the library's active options; a required
  setting cannot be cleared; at most 200 values.
- **`version` stops two admins overwriting each other.** It is the program's `config_version`, returned by every read and every save.
  A stale one gives **409** and saves nothing; omit it to skip the check (last write wins).
- **Only what differs is written.** Saving a page nobody edited reports `"changed": 0`, does not bump the version and shows no
  "unpublished changes". A real change bumps `config_version` and writes an audit entry (with before and after) and an outbox event.
- **Reading:** the manifest shows each setting's `value` / `option_ids`; `GET /programs/{program}/setting-values[?module=<key>]` lists
  them with the version. A saved choice whose option was since deactivated or removed from its library is kept and listed in
  `missing_option_ids`, so the page can warn; it can no longer be chosen.

### Fees (a rule page that starts with a rule)

Folder **Fees (a rule by default)**. Fees (`order_transaction.core_returns.fees`, in Core Returns) is a Rule leaf as soon as a program selects it, with no
Settings or Rules choice. The folder shows it three ways (its own key, its group `order_transaction.core_returns`, and a later PATCH), then reads the
**starter rule** that is already saved: draft 1, valid, a GoRules decision graph (request, one decision table with one row that charges 0, response).
It then publishes that rule as it is, saves an edit (version 2), and shows that Fees cannot become a Settings page once it has versions (400).
`default_mode` appears on every module catalog item (`rule` for Fees, `null` for the rest).

### Module catalog (the catalog; what Create Program uses)

Read-only: the list of every page a program can have, as a tree. Folders **Module catalog** and **Module catalog - negative checks** (the first two in the collection).

- `GET /module-catalog`: the six main modules (Overview, Catalog & Pricing, Enrollment, Document & Agreements, Order & Transaction, Configuration & Oversight), in display order, with
  `total_size`. **Overview is `selectable: false`** (it is added to every program automatically), so Create Program shows the other five.
- `GET /module-catalog?parent_id={id}`: the direct children of one module (the right panel of Create Program). A leaf, or an unknown id, gives an empty list.
- `GET /module-catalog?full_tree=true`: the **whole tree in one call**, nested under `children` (71 modules). With `&parent_id={id}` it returns the tree below that module (unknown id: **404**).
  The simplest way to build Create Program is to load this once and render it.
- `GET /module-catalog/{id or key}`: one module, for example `/module-catalog/enrollment`.
- Paging uses `max_page_size` (default 50, capped at 200, **400** if below 1) and `page_token` (a UUID; **400** if it is not one).
- There is no write API: the catalog comes from seed migrations (`POST /module-catalog` is a 404).
- gRPC: `ListModuleCatalog` and `GetModuleCatalogItem`.

### Notifications (the premade page in Settings)

`configuration_oversight.settings.universal.notifications` is a **premade** page (`api_base: notifications`) that a program gets by picking it in Create program;
it is then in `GET /programs/{program}/modules`.

- `GET /programs/{program}/notifications` is the grid: `channels` (the columns), `triggers` (key, label, email, sms, push, in_app) and `version`. A new program starts with 8 triggers. **404** when the program did not pick the page.
- `PUT /programs/{program}/notifications` saves the **whole grid** at once (ticks, added and removed triggers): `{"version": 1, "triggers": [...]}`. A trigger missing from the list is removed. `version` is the `config_version` from the last read: stale -> **409**; omit it to skip the check. Bad or repeated key, empty label -> **400**. The same grid again changes nothing.
- Folder **Notifications (premade page, picked in Create program)** covers all of it. gRPC: `GetNotifications`, `ReplaceNotifications`.

### Danger zone (the premade page in Settings)

`configuration_oversight.settings.universal.danger_zone` is a **premade** page (`api_base: danger-zone`, not selectable). Every program
has it, with its parents (Settings > Universal), without anyone picking it: new programs get it on create, existing ones got it from
the migration. So when the Settings page loads, the manifest (`GET /programs/{p}/modules`) already lists it, and the frontend calls:

- `GET /programs/{program}/danger-zone` is the card: `archived`, `archive_time`, `can_archive`, `can_unarchive`, `can_delete`,
  `delete_blockers` (`enrollments`, `applications`, `publications`, `documents`) and `delete_blocked_reason`.
- `POST /programs/{program}/danger-zone:archive` and `:unarchive`: **Archive program** takes the program out of the working catalog
  (`GET /programs` leaves it out; `GET /programs?show_archived=true` lists only archived ones). Nothing is lost, the program stays
  readable, `state` stays ACTIVE. Both return the card after the change; archiving twice (or unarchiving a live one) is a **400**.
- `POST /programs/{program}/danger-zone:deletePermanently` (**Delete program**, **204**): removes the program and its modules, values and
  rules for good. Only with no operational history: **409** while it has enrollments, applications, publications or documents (the
  card says which in `delete_blockers`); archive it then. The audit entry stays. This is different from `DELETE /programs/{program}`,
  the reversible soft delete.
- Folder **Danger zone (premade page, on every program)** covers all of it. gRPC: `GetDangerZone`, `ArchiveProgram`,
  `UnarchiveProgram`, `DeleteProgramPermanently`.

### Overview and audit log

- `GET /programs/{program}/overview` is the default page every program has. One call returns the program, `stats`, the `funnel`
  (all six statuses, `0` when empty), `published_rules` (`state` per rule leaf: `never`, `drifted`, `in_sync`, plus
  `drift_detected`), `unpublished_changes` and the latest 10 changes in `recent_activity`.
- `GET /programs/{program}/audit-logs` is the full history, newest first: `max_page_size`, `page_token`, and
  `action=created,updated` to filter. Each entry has `actor`, `action`, `summary` and the `before` / `after` of the change.
- Folder **Overview and audit log** in the collection covers both, including paging, the filter, 404 and a bad token.
- gRPC: `programs.v1.ProgramsService/GetOverview` and `/ListAuditLogs` (same fields).

### Requirements

> **Parked.** These requests are in `enrollment-review.postman_collection.json` and need `APP__FEATURES__ENROLLMENT_REVIEW=true`.

What a store must satisfy before it can be approved. Types: `signup`, `card_on_file`, `credit_application`, `contract` (one of each
per program; a second gives **409**) and `custom_field` (as many as you like).

- `GET /programs/{program}/requirements` lists them in display order (never paged: a program has only a handful).
- `POST /programs/{program}/requirements` adds one: `{type, name, config?, position?}`; no `position` puts it last.
- `GET` / `PATCH` / `DELETE /programs/{program}/requirements/{requirement}`: `type` cannot change; `config` replaces the object;
  delete is for good (add it again instead). Saving the same values again changes nothing.
- `POST /programs/{program}/requirements:batchUpdate` with `{requirements:[{id, position}]}` reorders in one go and returns the new order.
- Every real change bumps the program's `config_version` (the Overview then shows unpublished changes) and writes an audit entry
  (`requirement_created`, `requirement_updated`, `requirements_reordered`, `requirement_deleted`) with before and after.
- gRPC: `ListRequirements`, `GetRequirement`, `CreateRequirement`, `UpdateRequirement`, `DeleteRequirement`, `BatchUpdateRequirements`.
- Collection folders: **Requirements** and **Requirements - negative checks**.

### Approval steps

> **Parked.** These requests are in `enrollment-review.postman_collection.json` and need `APP__FEATURES__ENROLLMENT_REVIEW=true`.

The sign-offs an application goes through, in order; each step is decided by one role (`approver_role`).

- `GET /programs/{program}/approval-steps` lists them in order (never paged).
- `POST /programs/{program}/approval-steps` adds one: `{name, approver_role, position?}`; no `position` puts it last.
- `GET` / `PATCH` / `DELETE /programs/{program}/approval-steps/{approval_step}`. Saving the same values again changes nothing.
- A step that already has signoffs **cannot be removed** (409): the approval history needs it. Rename or reorder it instead.
- `POST /programs/{program}/approval-steps:batchUpdate` with `{approval_steps:[{id, position}]}` reorders in one go.
- Every real change bumps `config_version` and writes an audit entry (`approval_step_created`, `_updated`, `approval_steps_reordered`, `_deleted`).
- gRPC: `ListApprovalSteps`, `GetApprovalStep`, `CreateApprovalStep`, `UpdateApprovalStep`, `DeleteApprovalStep`, `BatchUpdateApprovalSteps`.
- Collection folders: **Approval steps** and **Approval steps - negative checks**. The 409 for a step with signoffs is not in
  the collection (signoffs cannot be created yet); it was checked by hand against the database.
- The library-backed folders need global-settings running on port 8000.

### Applications

> **Parked.** These requests are in `enrollment-review.postman_collection.json` and need `APP__FEATURES__ENROLLMENT_REVIEW=true`.

A store's request to join a program (the Application Panel under Enrollment). Statuses: `draft`, `submitted`, `in_review`,
`approved`, `rejected`, `withdrawn`.

- `GET /programs/{program}/applications` is the page: newest first, `status=submitted` is a chip (several are comma-separated, omit it for
  All), `max_page_size` / `page_token` page it. Each row has the store, website, status, `current_step_name` and `actions`
  (what can still be done). Rows carry no answers; read one application for those.
- `POST /programs/{program}/applications` starts a draft: `{store_id, website_id, store_name?, website_name?, answers?}`. A store can
  have only one open (draft / submitted / in review) application per program and website (**409**); after a withdrawal it can apply again.
- `GET /programs/{program}/applications/{application}` returns it with `answers` and `requirements_snapshot`.
- `POST .../applications/{application}:submit` (draft -> submitted): freezes the program's requirements into
  `requirements_snapshot`, puts it at the first approval step and counts it as a pending approval.
- `POST .../applications/{application}:withdraw` (draft / submitted / in review -> withdrawn).
- Any move the status does not allow gives **400** (`cannot submit an application that is already submitted`).
- Each change updates the Overview counters and the funnel in the same transaction and shows up in the history as
  `Application created` / `submitted` / `withdrawn`. Applications are data, not configuration: they do **not** bump `config_version`.
- Approve and reject are Signoffs (below). Checking `answers` against the requirements is not built yet.
- gRPC: `ListApplications`, `GetApplication`, `CreateApplication`, `SubmitApplication`, `WithdrawApplication`.
- Collection folders: **Applications** and **Applications - negative checks**.

### Signoffs (approve and reject)

> **Parked.** These requests are in `enrollment-review.postman_collection.json` and need `APP__FEATURES__ENROLLMENT_REVIEW=true`.

An approver's decision on the approval step an application is waiting at. Recorded once, never changed.

- `POST /programs/{program}/applications/{application}/signoffs` with `{decision: "approved" | "rejected", comment?, approval_step_id?}`.
  - `approved` moves the application to the next step (status `in_review`); approving the **last** step approves the application.
  - `rejected` rejects it, at any step.
  - Only a `submitted` or `in_review` application that is waiting at a step can be decided (**400** otherwise: a draft, a decided or
    withdrawn one, or one whose program had no approval steps when it was submitted).
  - Send `approval_step_id` (the step you were looking at): if the application has moved on, the answer is **409** and nothing is
    written. This makes a double click, or two approvers at once, safe.
  - The approver is the `X-User-Id` header (`system` when absent). Checking that the approver has the step's `approver_role` waits for
    authentication; it is **not enforced yet**.
- `GET .../signoffs` lists the decisions in the order they were made; `GET .../signoffs/{signoff}` reads one.
- A decision that ends the application (approved or rejected) takes it out of "pending approvals" on the Overview, in the same
  transaction, and shows in the history as `Application step approved` / `approved` / `rejected`.
- The application's `actions` now include `approve` and `reject` while it waits at a step.
- A step that has signoffs can no longer be removed (409), which the Approval steps folder cannot show, but this one does.
- **Approving the last step also enrolls the store** (see Enrollments), in the same transaction.
- gRPC: `ListSignoffs`, `GetSignoff`, `CreateSignoff`.
- Collection folders: **Signoffs** and **Signoffs - negative checks**.

### Enrollments (Enrolled Stores)

A store enrolled in a program on one website: the Enrolled Stores page. Statuses: `pending`, `active`, `suspended`, `ended`.

- **How a store gets enrolled:** automatically when its application's **last** approval step is approved (an `active` enrollment
  linked to the application); by an admin with `POST /programs/{program}/enrollments` (no application needed); or in bulk with
  `POST /programs/{program}/enrollments:batchCreate`.
- `GET /programs/{program}/enrollments` is the page: newest first, filters `website_id` (All websites), `status` (chips, comma-separated),
  `store` (name prefix, any case); `max_page_size` / `page_token`. `total_size` is the "N STORES" count for the active filters.
  Each row has `actions`: the status changes allowed from there.
- `POST .../enrollments` body `{store_id, website_id, store_name?, website_name?, customer_group?, status?}`; `status` is `active`
  (default) or `pending`. Without `store_name` the row has `store_resolved: false` ("Unresolved store"). A store with a live
  (pending / active / suspended) enrollment on the website gets **409**; after `ended` it can be enrolled again.
- `PATCH .../enrollments/{enrollment}` body `{status?, customer_group?}`. Allowed: `pending -> active`, `active <-> suspended`,
  anything `-> ended`. Anything else is **409**; `ended` is final. Sending the current status is not a change. An empty
  `customer_group` clears it.
- `POST .../enrollments:batchCreate` body `{items:[...]}` (1 to 500): every item gets its own outcome in the order sent,
  `created` / `already_enrolled` / `failed`; one bad item never fails the rest.
- Every change moves the Overview's `active_enrollments` and `enrollments_total` in the same transaction and shows in the history as
  `Enrollment created` / `status changed` / `updated` / `batch created`. Enrollments do **not** bump `config_version`.
- Not built yet: the contracts of an enrollment (`contracts_count` stays 0), resolving store and website names from the storefront
  (names are supplied by the caller), and anything that reads enrollments to price (Store Discounts).
- gRPC: `ListEnrollments`, `GetEnrollment`, `CreateEnrollment`, `BatchCreateEnrollments`, `UpdateEnrollment`.
- Collection folders: **Enrollments** and **Enrollments - negative checks**.

### Store discounts

**Operate → Store discounts** (module `catalog_pricing.store_discounts.control_panel`, so a program must be created with that module in
`modules`, otherwise every call below is **400**). Each enrolled store (pending, active or suspended) shows its effective discount and
which level it came from.

**Three levels, lowest to highest priority: Program → Domain (website) → Store.** Each of the four fields (`enabled`, `type`
`percentage|fixed`, `amount`, `currency_code`) is inherited **on its own**: a store can set only the amount and still take `enabled` from
the program. With nothing set anywhere a store is disabled, a percentage, of 0.

- `GET /programs/{program}/store-discounts` (query `website_id`, `store` = name prefix, `max_page_size`, `page_token`): newest first;
  `total_size` counts the matches. Each row has `resolved` (the Resolved Discount column), `sources` (the badge next to each field:
  `program` / `domain` / `store`), `set_at` (the Set At column: the highest level that set anything), `store_override` (what the store
  sets itself) and `inherited` (what it would get if its override were reverted).
- `GET .../store-discounts/{enrollment}`: the same row, what the edit panel opens with.
- `PATCH .../store-discounts/{enrollment}` body `{"discount": {...}}`: a field you send is **set**; a field sent as `null` is **reverted**
  to the inherited value (the revert button); `{"discount": null}` is "Revert all to inherited". A body without `discount` is **400**.
  The value the store would end up with is checked: a percentage cannot pass 100 (**422**), an amount has at most 2 decimals,
  `currency_code` is 3 uppercase letters.
- `POST .../store-discounts:batchUpdate` body `{"ids": [...], "discount": {...} | null}` is **Mass edit**: 1 to 500 enrollment ids, one write
  statement, **one** version bump and **one** audit entry whatever the number. All ids must be enrolled stores (**404**, naming them);
  if the result is invalid for any store the **whole** batch is refused (**422**, naming the store). `updated` counts the stores that
  really changed.
- `GET .../store-discount-levels` returns the program default and every website that sets something. `PATCH .../store-discount-levels/program`
  and `PATCH .../store-discount-levels/domains/{website_id}` change those two levels with the same `{"discount": ...}` body.
- Every change bumps `config_version` (the module requires publish, so the Overview shows **unpublished changes**), writes an audit entry and an
  outbox event; a request that changes nothing writes nothing. **Publishing is not built yet**, so edits are saved but not live.
- gRPC has no `null`: use `revert_fields` (a list of field names) and `revert_all` inside `discount`. Methods: `ListStoreDiscounts`,
  `GetStoreDiscount`, `UpdateStoreDiscount`, `BatchUpdateStoreDiscounts`, `ListDiscountLevels`, `UpdateDiscountLevel` (unset `website_id` =
  the program level).
- The amount is a plain number and the currency a code: the **UI** formats the symbol (`£11.21 off`, `15% off`, `Disabled`).
- Collection folders: **Store discounts** and **Store discounts - negative checks**.

### Execute rule (run a rule on an input)

`POST /programs/{program}/rule-sets/{module_key}:execute` with `{"input": {...}}` runs the rule and returns `{program, module, type, version, status, result, duration_ms}`.
Optional: `"type"` (the rule family you expect), `"version"` (`"published"` by default, `"draft"`, or a number), `"trace": true`.
Each rule version says what it reads in `input_schema` (a list of `{name, type, required}`), saved with `POST .../versions`; **Publish refuses a rule that has none**.
Wrong input answers **422** with `code: INPUT_MISMATCH` and the list to send: `required_fields`, `optional_fields`, `missing`, `wrong_type`. Other codes: `INVALID_BODY` (400),
`TYPE_MISMATCH`, `RULE_INVALID`, `RULE_EXECUTION_FAILED` (with `node_id`) (422), `RULE_TIMEOUT` (504). Folder **Execute rule (run a rule on an input)** covers all of it. gRPC: `ExecuteRule`.

### Rule sets, versions and publishing

**Rule sets.** A module switched on with mode `rule` is a *rule leaf* (for example Catalog & Pricing → Products & Rules → Warranty). The rule
engine itself is a frontend component: the backend keeps what it saves, versioned, and publishes it. A rule set is addressed by its **module key**
(or its rule set id).

- `GET /programs/{program}/rule-sets` lists the program's rule leaves with their `state`: `empty` (no rules saved yet; nothing to publish, so not drift),
  `never` (saved, never published), `drifted` (a newer draft than what is live) or `in_sync`.
- `GET .../rule-sets/{module_key}` is what the rule builder opens with: the leaf, its `draft_version` / `published_version` and the newest version in
  full under `draft`. **400** if the module is not a rule leaf of this program.
- `POST .../rule-sets/{module_key}/versions` saves the builder's draft as a new **immutable version**: `{definition, compiled?, valid?, validation_errors?,
  rule_type?}`. `definition` is an object (up to 1 MB) that the backend keeps as it is. Saving what is already the newest version (same definition, same
  validity, even with the keys in another order) creates nothing and answers `created: false`. An **incomplete draft can be saved** with `valid: false`
  (and `validation_errors`), but it **blocks publishing**. Rules are configuration: a real save bumps `config_version`.
- `GET .../rule-sets/{module_key}/versions` (newest first, paged, no definitions) and `GET .../versions/{version}` (one, in full).
- A version is `draft` (the newest save), `published` (live) or `superseded`. A published version is never changed.

**Publishing.** `POST /programs/{program}/publications` publishes the program, in one transaction under the program lock so the snapshot is consistent:
every rule leaf's newest draft becomes its published version (the previous one is superseded), the **bundle** is built and stored, and an audit entry and
outbox event are written. The bundle holds: the program, the enabled modules with their settings and values, the requirements, the approval steps, the
**store → discount map** (the three levels worked out once, for active stores that really get a discount) and the published rules. The answer carries a
`summary` (counts and the rule versions), not the bundle.

- **400** when there is nothing new to publish (the program has not changed since the last publication), and **400** naming the rules when a rule's newest
  draft is invalid. A change that is not a rule (a requirement, a store discount, ...) can be published too.
- Publishing does not change `config_version`; it records the version it covers. The Overview then shows `unpublished_changes: false`,
  `published_config_version` equal to `config_version`, and the rule leaves `in_sync`.
- `GET .../publications` (newest first, paged, rows carry the `summary`) and `GET .../publications/{publication}` (with the whole bundle).
- **Store discounts become part of what is published**, so editing them shows "unpublished changes" until the next publish. Nothing reads the bundle yet.
- gRPC: `ListRuleSets`, `GetRuleSet`, `CreateRuleVersion`, `ListRuleVersions`, `GetRuleVersion`, `ListPublications`, `CreatePublication`, `GetPublication`.
- Collection folders: **Rules and publishing** and **Rules and publishing - negative checks**.

### Documents (the file manager)

**Configure → Documents** (module `document_agreement.documents.documents_panel`: a program must be created with it in `modules`, otherwise every call
below is **400**). **The file bytes never pass through this service**: the browser sends them straight to the object storage with a presigned URL, and
the service only signs the URL and later asks the storage what arrived.

**The upload, in three steps**
1. `POST /programs/{program}/documents` with `{name, content_type, size_bytes, checksum?, folder_id?}`. Size and type are checked first (**413** too big,
   **415** type not allowed; defaults: 25 MB, pdf / png / jpeg / docx / xlsx / csv / plain text). The row is created as `uploading` (hidden from every list, not
   counted) and the answer carries `upload: {url, method: "PUT", headers, expires_time}`.
2. `PUT` the file bytes to `upload.url` with **exactly** `upload.headers` (`content-type`, and `x-amz-checksum-sha256` when a checksum was declared). The URL is
   for one object key and expires in minutes; the type, size and checksum are signed into it, so the **storage itself refuses** anything else (403 / 400).
3. `POST .../documents/{document}:complete`. The server asks the storage what really arrived: object missing → **409**; size, type or SHA-256 different → **422**.
   Then the file is `pending_scan`, counted in the totals and its folder, and the outbox event the scanner listens to is written. Completing twice is harmless.

**The page**
- `GET .../documents:summary`: "N files · X KB", the locations (`all`, `recent`, `needs_review`, `quarantined`) and the folders with their counts, in one read
  of a counters row (never a `COUNT(*)`).
- `GET .../documents`: newest first, paged. `location=recent` (the newest 20, no next page), `needs_review`, `quarantined`; `folder_id`; `q` (name prefix, any case).
- `GET|PATCH|DELETE .../documents/{document}`: `PATCH {name?, folder_id?}` renames and/or moves (the folder counters follow); `DELETE` is a soft delete (the totals follow).
- `POST .../documents/{document}:download` returns a short-lived presigned GET (an attachment named after the file). **409** while the file is `quarantined` or `uploading`.
- Folders: `GET|POST .../document-folders`, `PATCH|DELETE .../document-folders/{folder}`. Every program has four **system folders** (General, Import artifacts, Program
  Documents, Store Enrollment Documents) that cannot be renamed or deleted (**409**); a folder with files or sub-folders cannot be deleted (**409**); a name is
  unique in its parent, ignoring case (**409**), and has no slashes (**422**).

**Scan and review**
- `POST /webhooks/document-scan` `{document_id, result: clean|suspicious|infected, scanner?}` (the scanner's callback, **204**): a file that is `pending_scan` becomes `ok`,
  `needs_review` or `quarantined`; a repeated callback, or one for a file that is not waiting, changes nothing. Nothing becomes `ok` until it was scanned. With
  `storage.scan_webhook_secret` set, the `X-Webhook-Secret` header must carry it.
- `POST .../documents/{document}:review` `{decision: approve|reject, note?}`: only a file that `needs_review` (**409** otherwise); approve → `ok`, reject → `quarantined`.

Files are data, not configuration: nothing here bumps `config_version`. Every change writes an audit entry and an outbox event. gRPC (no bytes): `GetDocumentSummary`,
`ListDocumentFolders`, `CreateDocumentFolder`, `UpdateDocumentFolder`, `DeleteDocumentFolder`, `ListDocuments`, `CreateDocument` (returns the upload instructions),
`CompleteDocument`, `GetDocument`, `UpdateDocument`, `DeleteDocument`, `DownloadDocument`, `ReviewDocument`. Collection folders: **Documents** and **Documents - negative checks**.

**Not built yet:** a job that removes uploads stuck in `uploading` (they are hidden and never counted, but their rows and any object stay), removing the object of a
deleted file (use a bucket retention rule), the scanner itself (only its callback).

## 6. gRPC in Postman

1. **New → gRPC**, URL `localhost:50061` (the `grpc_host` variable), **no TLS**.
2. **Select a method → Use server reflection** — Postman reads `programs.v1.ProgramsService` and lists the methods:
   `CreateProgram`, `GetProgram`, `ListPrograms`, `UpdateProgram`, `DeleteProgram`, `UndeleteProgram`,
   `ListProgramModules`, `ListModuleCatalog`, `GetModuleCatalogItem`, `ListHealthCheck`.
3. Pick a method, press **Use Example Message** (or paste a body), then **Invoke**. For the audit actor add **Metadata**
   `x-user-id: divyesh`.

Example bodies (gRPC JSON uses camelCase):

```jsonc
// CreateProgram
{ "name": "CPO", "slug": "cpo", "level": "organization",
  "modules": ["catalog_pricing.store_discounts.control_panel"] }

// CreateProgram with a rule leaf and a settings leaf (`modules` = plain keys, `moduleConfigs` = keys with a mode / settings)
{ "name": "KBB", "slug": "kbb",
  "modules": ["enrollment.stores.enrolled_stores_control_panel"],
  "moduleConfigs": [
    { "key": "catalog_pricing.products_rules.pre_enrollment", "mode": "rule" },
    { "key": "catalog_pricing.products_rules.warranty", "mode": "settings",
      "settings": { "items": [
        { "name": "Warranty enabled", "type": "boolean", "defaultValue": true },
        { "name": "Max days", "type": "number", "required": true, "defaultValue": 90 },
        { "name": "Grade", "type": "radio", "options": [ { "label": "A" }, { "label": "B" } ] },
        { "name": "Coverage", "type": "dropdown", "libraryId": "<global-settings library id>" } ] } } ] }

// UpdateProgram: replace a leaf's settings (omit `settings` to leave them alone; `{ "items": [] }` removes them all)
{ "program": "kbb", "path": "programs/kbb",
  "modules": [ { "key": "catalog_pricing.products_rules.warranty",
                 "settings": { "items": [ { "name": "Only one", "type": "text" } ] } } ] }

// GetProgram  (the `path` field is required for authorization only)
{ "program": "cpo", "path": "programs/cpo" }

// UpdateProgram  (an empty description clears it)
{ "program": "cpo", "level": "user", "description": "", "path": "programs/cpo" }

// ListProgramModules
{ "program": "cpo", "path": "programs/cpo" }

// ListModuleCatalog with the whole tree
{ "fullTree": true }
```
The same from a terminal: `grpcurl -plaintext -d '{"program":"cpo","path":"programs/cpo"}' localhost:50061 programs.v1.ProgramsService/GetProgram`.

## 7. Swagger UI (browse and try the HTTP API)

```bash
docker compose up -d swagger     # http://localhost:8082  (8081 is the currency project's Swagger); the drop-down at the top also lists the parked modules' spec

# "Try it out" calls the API from the browser, so allow this page as a CORS origin:
APP__CORS_ALLOWED_ORIGIN=http://localhost:8082 APP__HTTP_PORT=8090 cargo run --bin programs_http
```

- The spec is `./openapi.yaml`, mounted read-only: edit it and refresh the page. If a tool replaced the file instead of
  editing it in place, `docker compose restart swagger`.
- The first server in the dropdown is **Local** (`http://localhost:8090`); the dev gateway URL is second.
- CORS is **off unless `APP__CORS_ALLOWED_ORIGIN` is set**, and only that one origin is allowed, so a deployed service
  never answers cross-origin calls.
- Stop it with `docker compose down`.
- Heads-up: `openapi.yaml` still lists planned resources that are not built yet (applications, enrollments, documents,
  rules, ...); calling them returns 404 until they exist. Built today: **Programs**, **Program modules**, **Module
  definitions** and **Health**.

## 8. Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `Could not get response` | the server is not running, or the port differs; check `base_url` in the environment |
| `404` on every `/programs/...` | an old binary is running; rebuild and restart |
| `409` on create | the slug (or `?id=`) already exists on a live program |
| Module checks fail with a different count | the database was not migrated; apply all `migrations/*.up.sql` in order |
| gRPC: reflection lists nothing | the descriptor is stale; rebuild it (`buf build proto --path proto/programs/v1 -o src/adapters/generated/grpc_services/programs.binpb`) |
