# Testing the Programs API in Postman

What exists today: **Programs** (create, get, list, update, delete, undelete), the **module manifest**
(`GET /programs/{program}/modules`) and the **module registry** (`/module-definitions`). Everything is on **HTTP** and
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

## 2. Import into Postman (HTTP)

1. **Import** → both files from the [postman/](../postman/) folder:
   - `programs.postman_collection.json`
   - `programs.local.postman_environment.json`
2. Top right, choose the environment **Programs local**. Its variables: `base_url` = `http://localhost:8090`,
   `grpc_host` = `localhost:50061`.
3. Run the collection (**Run** → **Run Programs API (local)**), or send requests one by one, **top to bottom**.

The collection generates a fresh slug (`pm-<time>`) on the first run, keeps it in the `slug` and `program_id`
variables, and **deletes its test program at the end**, so you can run it again and again. Expected result:
**91 requests, 186 checks, 0 failed.**

| Folder | What it shows |
|---|---|
| **Programs** | create → list → get by slug / by id → module manifest → update → delete → list deleted → undelete |
| **Settings and rules** | build a program with a **rule** leaf and a **settings** leaf (every type), read it back, replace its settings, change modes, switch modules off, add a new module |
| **Setting values** | save the values admins enter (every type, in one request), read them back, the version check, clearing, and options that disappear |
| **Webhook (global-settings -> programs)** | the event global-settings sends when a library changes |
| **Programs - negative checks** | duplicate slug 409, bad slug / level / module / name 422, unknown program 404, undelete of a live program 400 |
| **Settings and rules - negative checks** | 21 rejected payloads (bad type, library + own options, unknown library, duplicate names, wrong default, ...) |
| **Setting values - negative checks** | 14 rejected saves (wrong type, wrong option, required cleared, unknown setting, ...) |

The collection covers the **Programs resource only**. (The module registry, `/module-definitions`, is not part of it.)

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
- **`source: default`** → Overview, on every program automatically.

**An error** (RFC 9457 style):
```json
{ "type": "UNPROCESSABLE_ENTITY", "status": 422, "title": "Unprocessable Entity",
  "detail": "slug \"Bad Slug\" must be 1-64 lowercase letters, digits and single hyphens", "instance": "26c49b90-…" }
```

## 4. How to use it properly

**Choosing modules at creation.** `modules` takes module **keys**. List them with `GET /module-definitions?full_tree=true`.
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
included. They stay current: global-settings calls `POST /webhooks/global-settings` when a library, attribute or option changes and
the cache is cleared and re-read, so a new or renamed option shows up on the very next read; anything missed is refreshed in the
background every 30 seconds. If global-settings is down, the page still loads and the affected setting reports
`"library_error": "unavailable"`. To make it work locally set `[global_settings] base_url` (default `http://localhost:8000`) and
give global-settings `APP__ENABLE_EVENT=true` and a webhook URL of `http://localhost:8090/webhooks/global-settings` (a library must also
have `event_enabled`). The Postman variable `library_id` must be a library that exists there.

### Saving the values

The settings above only DEFINE the fields. The values admins enter are saved separately, in one request per page:

```jsonc
POST /programs/kbb/field-values:batchUpdate
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
- **Reading:** the manifest shows each setting's `value` / `option_ids`; `GET /programs/{program}/field-values[?module=<key>]` lists
  them with the version. A saved choice whose option was since deactivated or removed from its library is kept and listed in
  `missing_option_ids`, so the page can warn; it can no longer be chosen.

### Overview and audit log

- `GET /programs/{program}/overview` is the default page every program has. One call returns the program, `stats`, the `funnel`
  (all six statuses, `0` when empty), `published_rules` (`state` per rule leaf: `never`, `drifted`, `in_sync`, plus
  `drift_detected`), `unpublished_changes` and the latest 10 changes in `recent_activity`.
- `GET /programs/{program}/audit-logs` is the full history, newest first: `max_page_size`, `page_token`, and
  `action=created,updated` to filter. Each entry has `actor`, `action`, `summary` and the `before` / `after` of the change.
- Folder **Overview and audit log** in the collection covers both, including paging, the filter, 404 and a bad token.
- gRPC: `programs.v1.ProgramsService/GetOverview` and `/ListAuditLogs` (same fields).

### Requirements

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
- Approve / reject (signoffs) and answers checking are not built yet.
- gRPC: `ListApplications`, `GetApplication`, `CreateApplication`, `SubmitApplication`, `WithdrawApplication`.
- Collection folders: **Applications** and **Applications - negative checks**.

## 6. gRPC in Postman

1. **New → gRPC**, URL `localhost:50061` (the `grpc_host` variable), **no TLS**.
2. **Select a method → Use server reflection** — Postman reads `programs.v1.ProgramsService` and lists the methods:
   `CreateProgram`, `GetProgram`, `ListPrograms`, `UpdateProgram`, `DeleteProgram`, `UndeleteProgram`,
   `ListProgramModules`, `ListModuleDefinitions`, `GetModuleDefinition`, `ListHealthCheck`.
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

// ListModuleDefinitions with the whole tree
{ "fullTree": true }
```
The same from a terminal: `grpcurl -plaintext -d '{"program":"cpo","path":"programs/cpo"}' localhost:50061 programs.v1.ProgramsService/GetProgram`.

## 7. Swagger UI (browse and try the HTTP API)

```bash
docker compose up -d swagger     # http://localhost:8082  (8081 is the currency project's Swagger)

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
