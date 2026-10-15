# Programs service — implementation plan

Eight parts, all **for review**:
- **Part A (sections 1–6): Program Settings** — the first settings modules, **Configuration & Oversight →
  Settings → Universal** (Program Settings, Shopping Cart, Vault Payment Methods, Program Level, About this
  Program, Danger Zone, and the other universal cards).
- **Part B (section 7): Applications & Approvals** — **Enrollment → Applications & Approvals → Application
  Panel**, built for the lowest latency.
- **Part C (section 8): Enrolled Stores** — **Enrollment → Stores → Enrolled Stores Control Panel**, built for the
  lowest latency.
- **Part D (section 9): Store Discounts** — **Catalog & Pricing → Store Discounts → Store Discounts Control Panel**,
  built for the lowest latency.
- **Part E (section 10): Imports** — **Order & Transaction → Imports → Upload Consignment Orders + Import History**
  (the same import-jobs API also serves Price Import).
- **Part F (section 11): Products & Rules (rule engine)** — **Catalog & Pricing → Products & Rules → Pre-Enrollment,
  Post-Enrollment, Warranty** (+ Live Results): rule builder, Custom CSV, test cases, publish, runtime evaluation.
- **Part G (section 12): Enrollment Requirements** — **Enrollment → Requirements → Enrollment Requirements**: the
  ordered list that *is* the program's application form (Sign-up, Card on file, Credit application, Custom field, Contract).
- **Part H (section 13): Documents** — **Document & Agreements → Documents → Documents Panel**: a file manager (locations,
  folders, search, upload, storage), files kept in object storage with presigned upload / download.

Every change ships on HTTP, gRPC, `openapi.yaml` and Postman together.

Status: nothing in sections 3–5 or 7–13 is built yet unless marked *done*. The module registry is **seeded and live**
(70 modules, all five main modules, HTTP + gRPC).

---

## 1. The idea in one paragraph

Each settings card is described by a **library in `global-settings`**: the library's attributes are
the card's fields (`bool`, `text`, `number`, or `ref_single` / `ref_multi` pointing at a choice
library such as Payment Methods). The programs service reads those definitions (cached), stores
each program's **chosen values in its own database**, and serves the whole page in **one call**.
The frontend never talks to `global-settings`.

---

## 2. Normal flow (who calls what)

| # | Step | Who | Call |
|---|------|-----|------|
| 1 | Admin opens a program | Frontend → Programs API | `GET /programs/{program}` |
| 2 | Menu / tabs drawn | Frontend → Programs API | `GET /programs/{program}/modules` |
| 3 | Settings tab opened | Frontend → Programs API | `GET /programs/{program}/settings?include_options=true` |
| 3a | …cards, fields, saved values, options assembled | Programs API | in-memory definitions + 1 DB read + Redis options; `global-settings` only on a cache miss |
| 4 | Admin edits several cards; "N unsaved changes" bar fills | Frontend | no call |
| 5 | Admin clicks **Save changes** | Frontend → Programs API | `PATCH /programs/{program}/settings` (changed fields only) |
| 5a | Validate, then one transaction: values + version bump + snapshot + audit + outbox | Programs API | — |
| 6 | Option added / changed in `global-settings` | `global-settings` → Programs API | `POST /webhooks/global-settings` (clears that library's cache) |
| 7 | Unpublished changes badge; someone publishes | Frontend | later phase (Publish) |

Access: calls carry the admin's session through the gateway; `permission_key` on each module decides
who may edit it. Service-to-service calls to `global-settings` use `X-Client-ID` / `X-Client-Secret`.

---

## 3. What we make

**Already done**
- Module tree (`module_definitions`): 51 modules seeded, `GET` / `LIST` / full tree on HTTP + gRPC.
- All 38 tables migrated (programs, modules, field values, outbox, audit log, …).
- HTTP + gRPC servers, health, OpenAPI file, Postman collection.

**To build**

| Piece | What |
|---|---|
| Program resource | create, get, list, update, archive/delete (needed: no program exists yet) |
| `programs.level` | new column for the Program Level card (User / Organization / Location) |
| Card → library link | each `field_form` module knows which `global-settings` library describes it |
| `global-settings` client | `reqwest`, service headers, timeout, config section `[global_settings]` |
| Cache | in-memory definitions + Redis options; cleared by the webhook |
| Settings read | assemble cards, values, options in one response |
| Settings save | validate + one transaction (below) |
| `program_settings_snapshot` | one JSON row per program, written in the same transaction; the page read is one primary-key lookup |
| Webhook endpoint | receives `lib.*` events, clears cache, flags programs using a removed option |

**Save transaction (one commit)**
1. Upsert changed rows in `program_field_values`.
2. Bump `programs.config_version` and `config_hash`.
3. Rewrite the program's snapshot row.
4. Insert `audit_log` (before/after) and `outbox`.

**Validation on save:** field exists and applies to this program · value matches type · every option id belongs to that
field's library and is active · single-select has exactly one option · derived fields rejected ·
`version` matches (else 409).

---

## 4. API list

Errors everywhere: `400` bad input · `404` not found · `409` version conflict / duplicate · `422` bad format · `500`.
All lists use `max_page_size`, `page_token`, `next_page_token`, `total_size`.

### 4.1 Programs

| Method | Path | Request | Response |
|---|---|---|---|
| `GET` | `/programs` | `max_page_size`, `page_token`, `show_deleted` | `{results:[Program], next_page_token, total_size}` |
| `POST` | `/programs?id=` | `{name, slug, description?, blueprint_id?}` | `Program` |
| `GET` | `/programs/{program}` | – | `Program` |
| `PATCH` | `/programs/{program}` | `{name?, description?, active?, level?}` | `Program` |
| `DELETE` | `/programs/{program}` | – | `204` (soft delete; reversible) |
| `POST` | `/programs/{program}:undelete` | – | `Program` |

`Program`: `{path, id, name, slug, description, active, level, blueprint_id, config_version, config_hash, state, create_time, update_time}`

### 4.2 Modules of a program

| Method | Path | Request | Response |
|---|---|---|---|
| `GET` | `/programs/{program}/modules` | `parent_id?`, `full_tree?` | enabled modules only (same shape as module definitions) |
| `GET` | `/module-definitions` | `parent_id?`, `full_tree?`, paging | *done* |
| `GET` | `/module-definitions/{id-or-key}` | – | *done* |

### 4.3 Settings (the new core)

**`GET /programs/{program}/settings?include_options=true&module=`** (`module` optional: one card only)

```json
{
  "version": 14,
  "cards": [
    {
      "module": "configuration_oversight.settings.universal.program_settings",
      "name": "Program Settings",
      "fields": [
        { "key": "products_associated", "label": "Products Associated", "type": "bool", "value": true },
        { "key": "associated_payment_methods", "label": "Associated Payment Methods",
          "type": "ref_multi", "selected": ["opt-1","opt-2"],
          "options": [{"id":"opt-1","name":"Apple Pay","code":"apple_pay"}],
          "visible": true },
        { "key": "owner_code", "label": "Owner Code", "type": "ref_single",
          "selected": ["opt-9"], "options": [{"id":"opt-9","name":"MobileSentrix"}] },
        { "key": "enrollment_required", "label": "Enrollment Required",
          "type": "derived", "value": true, "source": "enrollment requirements" }
      ]
    }
  ]
}
```
A selected option that no longer exists comes back as `{id, deleted:true}` (warning on the field).

**`PATCH /programs/{program}/settings`** — only changed fields

```json
{
  "version": 14,
  "values": {
    "program_settings": {
      "owner_code": { "option_ids": ["opt-3"] },
      "products_associated": { "value": false }
    },
    "shopping_cart": { "separate_cart_group": { "value": true } }
  }
}
```
Response: the same body as `GET` (fresh `version`, so the page can re-render).
`409` if `version` is stale · `422` if an option does not belong to the field's library.

### 4.4 Supporting

| Method | Path | Request | Response |
|---|---|---|---|
| `GET` | `/programs/{program}/audit-logs` | paging | `{results:[{actor, action, target, before, after, create_time}]}` (History / View changes) |
| `POST` | `/webhooks/global-settings` | `{event, library_id, revision?}` | `204` (internal) |
| `GET` | `/health` | – | *done* |

### 4.5 gRPC (same names, `programs.v1.ProgramsService`)
`ListPrograms`, `CreateProgram`, `GetProgram`, `UpdateProgram`, `DeleteProgram`, `UndeleteProgram`,
`ListProgramModules`, `GetProgramSettings`, `UpdateProgramSettings`, `ListAuditLogs`, plus the
existing `ListModuleDefinitions`, `GetModuleDefinition`, `ListHealthCheck`.

---

## 5. Build order (each phase ends with HTTP + gRPC + OpenAPI + Postman tested)

1. **Programs** — table column `level`, program CRUD, list.
2. **Program modules** — create a program copies its blueprint's modules into `program_modules`; `GET .../modules`.
3. **`global-settings` client + cache + webhook** — fetch a library, cache it, clear on event.
4. **Settings read** — `GET .../settings`.
5. **Settings save** — `PATCH .../settings`, snapshot row, audit, outbox.
6. **History** — `GET .../audit-logs`.

---

## 6. Decisions needed from review

1. **Card ↔ library:** one library **per card** (`program_settings`, `shopping_cart`, `vault_payment`, …) or one
   `program_settings` library for the whole page plus a card mapping? *Recommended: one per card.*
2. **Extras that a library attribute cannot hold** (default value, tooltip, show-only-when rule, "applies to
   KBB only"): a small `module_field_extras` table keyed by attribute id — needed now, or later?
3. **Toggles** (Yes/No): stored as plain booleans (no library). Confirm.
4. **Archive vs delete** (Danger Zone): is *Archived* the same as soft-deleted, or a separate state?
5. **Program Level** saves on its own button today; keep it a separate `PATCH /programs/{program}` call?
6. **Programs belong to a business unit?** If yes, `programs` needs `business_unit_id` and `slug` becomes unique per unit.
7. **`global-settings` speed-ups** (batch get, revision / ETag, revision in webhook): do them first, or start without?

---

# Part B — Applications & Approvals

## 7. Applications & Approvals

Screen: Operate → Applications. Columns **Store, Website, Status, Step, Submitted, Actions**; filter chips
**All, Submitted, In review, Approved, Rejected, Draft, Withdrawn** with a count each.

### 7.1 Tables

| Table | Used for |
|---|---|
| `applications` | One row per store application: store, website, status, submitted time, frozen `requirements_snapshot`. **The list screen reads only this table.** |
| `approval_steps` | The program's approval chain (name, order, approver role). Tiny; kept in memory. |
| `signoffs` | One approve / reject decision per application per step. Unique on `(application_id, approval_step_id)`. |
| `enrollments` | Created when the last step approves; also feeds the pricing pods' store → programs map. |
| `agreement_contracts` | Created with the enrollment, pinned to an agreement template version. |
| `requirements` | What a store must provide; copied into `applications.requirements_snapshot` at submit. |
| `audit_log`, `outbox` | History row and event to send, both written in the approve / reject transaction. |

**New (not migrated yet):** on `applications` add `store_name`, `website_name` (copied at submit; external data, so the
list never calls the storefront), `current_step_id`, `current_step_name`, `decided_time`; index
`(program_id, status, id)` replaces the single partial `submitted` index.

### 7.2 API list

Errors: `400` · `404` · `409` (already decided / duplicate) · `422` · `500`. Lists use `max_page_size`, `page_token`.

| Method | Path | Request | Response |
|---|---|---|---|
| `GET` | `/programs/{p}/applications` | `status?`, `max_page_size`, `page_token` | `{results:[ApplicationRow], counts:{all, submitted, in_review, approved, rejected, draft, withdrawn}, next_page_token}` |
| `POST` | `/programs/{p}/applications` | `{store_id, website_id}` | `Application` (draft) |
| `GET` | `/programs/{p}/applications/{a}` | – | `Application` + `requirements_snapshot` + `signoffs[]` |
| `POST` | `/programs/{p}/applications/{a}:submit` | – | `Application` (status `submitted`, requirements frozen) |
| `POST` | `/programs/{p}/applications/{a}/signoffs` | `{approval_step_id, decision: approved\|rejected, comment?}` | `Signoff` (+ updated `Application`; on the last step an `Enrollment` is created) |
| `GET` | `/programs/{p}/applications/{a}/signoffs` | paging | `{results:[Signoff]}` |
| `GET` `POST` `PATCH` `DELETE` | `/programs/{p}/approval-steps[/{step}]` | `{name, position, approver_role}` | `ApprovalStep` / list |

`ApplicationRow` carries everything the list shows, so the frontend makes no other call:
`{id, store_id, store_name, website_id, website_name, status, current_step_name, submitted_time, actions:["approve","reject"]}`
(`actions` computed on the server from the caller's role and the current step).

gRPC (same names): `ListApplications`, `CreateApplication`, `GetApplication`, `SubmitApplication`,
`CreateSignoff`, `ListSignoffs`, `ListApprovalSteps`, `CreateApprovalStep`, `UpdateApprovalStep`, `DeleteApprovalStep`.

### 7.3 Lowest-latency design

**Read (list)**
- One query, one table, no joins: names and the current step are copied onto the row.
- Chips: `(program_id, status, id)` serves each filter as one index range, with keyset paging.
- List selects only displayed columns, so the large `requirements_snapshot` is never read.
- Counts: one grouped query on the `(program_id, status)` index (index-only); switch to per-program counter rows
  only if a program gets very large. `total_size` is optional, never a full table count.
- Detail (requirements + signoffs) is a separate call; the list never carries it.

**Write (approve / reject), one short transaction**
1. `SELECT … FOR UPDATE` the application row (two approvers cannot collide).
2. Insert the signoff (unique key blocks duplicates).
3. Update the application's status and current step.
4. Last step → create the enrollment (and agreement contract) here.
5. Write `audit_log` and `outbox`.

- No outside call inside the transaction; emails and the enrollment event go out through the outbox after commit.
- Approval steps are read from memory, not the database.
- Idempotency key on approve / reject so a retry cannot create a second decision.
- Status changes only a few times per application, so keeping it indexed is cheap; `fillfactor 85` keeps updates in place.

**Targets (estimates, to be measured):** list page 2–5 ms of DB time; approval under 15 ms, on thousands of applications.

### 7.4 Build order
1. Migration: new `applications` columns and index.
2. Approval steps CRUD.
3. Applications create, get, list (with counts), submit.
4. Signoff create (the transaction above), enrollment creation, outbox event.
5. Load test the list and approve paths.

### 7.5 Decisions for review
1. **Roles:** which role may approve each step? (`approver_role` on the step.)
2. **Statuses:** are Draft, Submitted, In review, Approved, Rejected, Withdrawn the full set? Is **Withdrawn** a store action?
3. **Last step:** does approval always create the enrollment immediately, or wait for the agreement to be signed?
4. **Counts:** live grouped query (default) or counter rows from day one?

---

# Part C — Enrolled Stores

## 8. Enrolled Stores

Screen: Operate → Stores. Header: **All websites** filter, **"N STORES"** count, **Bulk enroll**, **Enroll store**.
Columns: **Store, Website, Customer Group, Status, Enrolled, Last Contract, Contracts, Actions** (View + status
dropdown). Statuses seen: **Pending, Active, Suspended**.

### 8.1 Tables

| Table | Used for |
|---|---|
| `enrollments` | One row per store enrolled in a program on a website. **The list screen reads only this table.** |
| `agreement_contracts` | A store's contracts; only read by the *View* call (the list uses counters on `enrollments`). |
| `applications` | Link back to the application that produced the enrollment (`application_id`), when there was one. |
| `audit_log`, `outbox` | History row and event, written in the same transaction as every status change. |

**New (not migrated yet) on `enrollments`:**

| Column | Why |
|---|---|
| `store_name`, `website_name` | Copied at enroll time; the list never calls the storefront. |
| `store_resolved BOOLEAN` | `false` = show *"Unresolved store"*; a background retry fills the name in later. |
| `customer_group TEXT` | Per enrollment (rows on the same website differ: standard, tier-1, tier-2). |
| `enrolled_time TIMESTAMPTZ` | The *Enrolled* date (differs from `create_time` while a row is still Pending). |
| `contracts_count INT`, `last_contract_time TIMESTAMPTZ` | The last two columns, kept on the row. |

**Index changes**
- Add `(program_id, website_id, id)` — serves the *All websites* filter.
- Change the "one live enrollment" unique index `(program_id, store_id, website_id)` to cover
  `status IN ('pending','active','suspended')` — today a suspended store could be enrolled a second time.
- Keep `(program_id, id)`, the partial `active` index `(store_id, program_id) INCLUDE (website_id)` (pricing pods' map),
  and `application_id`.

### 8.2 API list

Errors: `400` · `404` · `409` (already enrolled / illegal status change) · `422` · `500`.

| Method | Path | Request | Response |
|---|---|---|---|
| `GET` | `/programs/{p}/enrollments` | `website_id?`, `status?`, `max_page_size`, `page_token` | `{results:[EnrollmentRow], next_page_token, total_size}` |
| `POST` | `/programs/{p}/enrollments` | `{store_id, website_id, customer_group?}` | `Enrollment` |
| `POST` | `/programs/{p}/enrollments:batchCreate` | `{items:[{store_id, website_id, customer_group?}]}` | `{results:[{store_id, website_id, outcome: created\|already_enrolled\|failed, enrollment?, error?}]}` |
| `GET` | `/programs/{p}/enrollments/{e}` | – | `Enrollment` + `contracts[]` (*View*) |
| `PATCH` | `/programs/{p}/enrollments/{e}` | `{status?, customer_group?}` | `Enrollment` |

`EnrollmentRow`: `{id, store_id, store_name (null = unresolved), website_id, website_name, customer_group, status,
enrolled_time, last_contract_time, contracts_count, actions:[allowed status changes]}`.

Allowed status changes: `pending → active`, `active ⇄ suspended`, any → `ended`. Anything else is `409`.

gRPC (same names): `ListEnrollments`, `CreateEnrollment`, `BatchCreateEnrollments`, `GetEnrollment`, `UpdateEnrollment`.

### 8.3 Lowest-latency design

**Read (list)**
- One query, one table, no joins: `WHERE program_id = ? [AND website_id = ?]`, keyset order; every shown value is on the row.
- *"N STORES"*: one index-only count for the active filter.
- *View* is a separate call (enrollment + contracts); the list never carries contracts.

**Write**
- **Enroll one:** one insert; store and website names resolved from the storefront *before* the insert. If the storefront
  is unreachable, insert anyway with `store_resolved = false` (retried in the background). Never wait on it when listing.
- **Bulk enroll:** one `INSERT … SELECT unnest(…) ON CONFLICT DO NOTHING` per chunk of a few hundred, one batched outbox
  write; the response reports each item (created / already enrolled / failed) so one bad row cannot fail the batch.
- **Status change:** one `UPDATE` guarded by the allowed-transition check, in one transaction with `audit_log` and
  `outbox`. The outbox event updates the pricing pods' store → programs map, so a suspended store drops out of program
  pricing within moments.
- **Contract counters:** `contracts_count` / `last_contract_time` updated in the same transaction that creates or signs
  a contract (so the list never joins `agreement_contracts`).

**Targets (estimates, to be measured):** list page 2–5 ms of DB time; status change under 10 ms.

### 8.4 Build order
1. Migration: new `enrollments` columns and indexes.
2. List and get (with contracts).
3. Enroll one, then status change with audit and outbox.
4. Bulk enroll.
5. Contract counters (when the agreement flow creates contracts).
6. Load test the list and bulk paths.

### 8.5 Decisions for review
1. **Customer group:** read from the store's group in the storefront at enroll time, or chosen by the admin?
2. **Store names:** which service resolves store id → name, and how fast / reliable is it?
3. **Statuses:** are Pending, Active, Suspended the full set? Is there *Ended* / *Removed*?
4. **Enroll store vs applications:** does an admin enrolling a store skip the application and approval flow?
5. **Last Contract:** the signed date or the created date?

---

# Part D — Store Discounts

## 9. Store Discounts

Screen: Operate → Store discounts. *"Each store's effective discount, and which level it came from. Programs is the
source of truth for per-store discounts."* Header: **All websites** filter, **Search stores**, **Mass edit** (enabled
when rows are ticked). Columns: **Store, Website, Resolved Discount, Set At, Edit**.

**Three levels, lowest to highest priority** (the *Set At* badge says which one won):

| Badge | Meaning |
|---|---|
| **From program** | Inherited from the program default; no domain or store override. |
| **From domain** | Set at the domain level; the store inherits it. |
| **From this store** | Overridden on the store; wins over domain and program. |

*Resolved Discount* is the winner of those three, e.g. `Disabled` or `5%`.

### 9.1 Tables (no new table)

| Table | Used for |
|---|---|
| `enrollments` | The list of stores (store, website, names). A row on this screen = one enrolled store. |
| `program_overrides` | The **domain** and **store** discounts. `scope_type` = `domain` / `store`, `scope_id` = website id / store id, `config` = the discount. Unique on `(program_id, scope_type, scope_id)`, so every lookup is one index probe. |
| `program_modules.config` | The **program-level** default discount (the Store Discounts module's own config). |
| `programs` | `config_version` / `config_hash` bumped on every edit (the *unpublished changes* badge). |
| `audit_log`, `outbox` | History row and event in the same transaction as each edit. |

`config` shape (all three levels): `{ "enabled": true, "discount_pct": 5 }`; `enabled:false` (or no row) = *Disabled*.

**New (not migrated yet):** index `(program_id, lower(store_name) text_pattern_ops)` on `enrollments` for prefix search
(Part C adds `store_name`). Nothing else.

### 9.2 API list

Errors: `400` · `404` · `409` (version conflict) · `422` (bad discount value) · `500`.

| Method | Path | Request | Response |
|---|---|---|---|
| `GET` | `/programs/{p}/store-discounts` | `website_id?`, `q?` (store name prefix), `max_page_size`, `page_token` | `{results:[StoreDiscountRow], next_page_token, total_size}` |
| `PATCH` | `/programs/{p}/store-discounts/{enrollment}` | `{discount: {enabled, discount_pct}}` or `{discount: null}` (reset to inherited) | `StoreDiscountRow` |
| `POST` | `/programs/{p}/store-discounts:batchUpdate` | `{ids:[enrollment ids], discount: {…} \| null}` | `{updated: N, results:[StoreDiscountRow]}` (*Mass edit*) |

`StoreDiscountRow`: `{id (enrollment id), store_id, store_name, website_id, website_name, resolved: {enabled, discount_pct},
set_at: "program" | "domain" | "store"}`.

The domain-level and program-level values are edited through the existing `program-overrides` and `program-modules`
APIs; this screen only reads them (through `set_at`) and edits the **store** level.

gRPC (same names): `ListStoreDiscounts`, `UpdateStoreDiscount`, `BatchUpdateStoreDiscounts`.

### 9.3 Lowest-latency design

**Read (list)**
- Resolve **for the page only** (50 rows): `enrollments` row + one probe each into `program_overrides` for the store and
  domain level + the program default read once per request. Three indexed lookups per row, no table scan, no write
  amplification, and it is always consistent with the source rows.
- Keyset paging on `enrollments (program_id, id)`; *All websites* uses `(program_id, website_id, id)`; search is a
  prefix match on the new index (a contains-match would need a trigram index; add only if needed).

**Write**
- **Edit one:** upsert (or delete, for *reset to inherited*) one `program_overrides` row, bump `config_version` /
  `config_hash`, write `audit_log` and `outbox`: one transaction.
- **Mass edit:** one statement for all selected ids
  (`INSERT … SELECT unnest(…) ON CONFLICT (program_id, scope_type, scope_id) DO UPDATE`, or one `DELETE` for reset),
  **one** version bump, **one** audit row, **one** outbox event, whatever the number of rows.
- Validation: `discount_pct` between 0 and 100; stores must be enrolled in this program.

**Publish (pricing never reads this screen's data)**
This leaf has `requires_publish = true`: edits show *unpublished changes* until Publish. Publish resolves the three
levels once into a `store_id → discount` map inside the program's bundle, so pricing does an in-memory lookup with no
database call.

**Targets (estimates, to be measured):** list page 2–5 ms DB time; one edit under 10 ms; mass edit of 500 stores
under 50 ms.

### 9.4 Build order
1. Migration: prefix-search index (after Part C columns).
2. List with resolution and `set_at`.
3. Edit one (upsert / reset) with audit and outbox.
4. Mass edit (batch update).
5. Publish: write the resolved `store → discount` map into the bundle.
6. Load test the list and mass-edit paths.

### 9.5 Decisions for review
1. **Discount shape:** percent only, or also fixed amount / tiers? What exactly does *Disabled* mean: no discount, or the feature off?
2. **Where are the program-level and domain-level discounts edited** (which screens), so the *Set At* levels have a source?
3. **Store override scope:** per store, or per (store, website)? If a store is enrolled on two websites, should one override cover both?
4. **Which stores are listed:** only stores enrolled in the program, or every store on the program's websites?
5. **Mass edit:** can it also *reset to inherited* for the selected rows (my proposal: yes)?
6. **Publish:** confirm store discounts are not live until published.

---

# Part E — Imports (Upload Consignment Orders + Import History)

## 10. Imports

Screen: Operate → Imports. One page, two modules (two leaves) in the registry:

- **Upload Consignment Orders** — *"Upload a CSV with store and product details to create consignment orders."*
  **Download template** button; drop zone; `.csv` up to **10 MB**; columns **Order ref, Store ref (optional), SKU (optional),
  Quantity (optional), Amount (optional), Order date (optional)**.
- **Import History** — *"Recent imports for this program, with row-level results from the import-jobs API."* Each upload
  lands here with its row-level results.

The same `import-jobs` API and tables serve **Price Import** (Catalog & Pricing), selected by `kind`.

### 10.1 Tables (already migrated)

| Table | Used for |
|---|---|
| `import_jobs` | One row per upload: `kind` (`consignment_upload`, `price_import`), `status`, counters. History reads only this table. |
| `import_job_rows` | One row per CSV line with its result (`ok` / `failed`, `data`, `error`). Insert-only, minimal indexes. |
| `audit_log`, `outbox` | Who uploaded what, and the "import finished" event. |

**New (not migrated yet) on `import_jobs`:** `file_name`, `created_by`, `succeeded_rows`, `started_time`, `finished_time`,
`error` (job-level failure message). The original file is **not** stored (see decision 3); only parsed rows are.

### 10.2 API list

Errors: `400` (bad header / not CSV) · `404` · `409` (job already finished) · `413` (over 10 MB) · `422` (bad row values) · `500`.

| Method | Path | Request | Response |
|---|---|---|---|
| `GET` | `/programs/{p}/import-jobs:template` | `kind` | the CSV template (header row only), `text/csv` |
| `POST` | `/programs/{p}/import-jobs` | `multipart/form-data`: `file`, `kind` | `Operation` (returns as soon as the file is accepted); poll it |
| `GET` | `/programs/{p}/import-jobs` | `kind?`, `status?`, `max_page_size`, `page_token` | `{results:[ImportJob], next_page_token, total_size}` (*Import history*) |
| `GET` | `/programs/{p}/import-jobs/{j}` | – | `ImportJob` |
| `GET` | `/programs/{p}/import-jobs/{j}/rows` | `status?` (`failed`), paging | `{results:[{row_number, status, data, error}], next_page_token}` |
| `POST` | `/programs/{p}/import-jobs/{j}:cancel` | – | `ImportJob` |
| `GET` | `/operations/{op}` | – | `Operation` (`done`, `response`, `error`) |

`ImportJob`: `{id, kind, file_name, status, total_rows, succeeded_rows, failed_rows, created_by, create_time, started_time, finished_time, error?}`.
Statuses: `pending → running → succeeded | failed | cancelled`. A job with some bad rows **still ends `succeeded`** with
`failed_rows > 0`; only a job-level failure (bad header, unreadable file) ends `failed`.

gRPC (same names): `GetImportTemplate`, `CreateImportJob` (the file as a `bytes` field, same 10 MB cap), `ListImportJobs`,
`GetImportJob`, `ListImportJobRows`, `CancelImportJob`, plus `GetOperation`.

### 10.3 Lowest-latency design

**Upload returns fast; the work is asynchronous**
1. Check size (≤ 10 MB), content type, and the **header row** immediately. Wrong header fails in milliseconds with a clear `400`.
2. Insert the `import_jobs` row (`pending`) and return the `Operation`. The user sees the job in history at once.
3. A worker streams the CSV: **no whole-file load into memory**, rows validated as they are read.

**Bulk insert, never row by row**
- Rows go in batches of ~1,000 with one `INSERT … SELECT unnest(…)` per batch; counters on `import_jobs` update **once per
  batch**, not per row.
- `import_job_rows` keeps only its primary key and `(import_job_id, id)` plus the partial *failed* index, so inserts stay cheap.
- One job per program runs at a time (advisory lock) so two uploads cannot fight over the same orders.

**Reads**
- Import history: one query on `import_jobs`, `(program_id, id)` keyset; no counts query, the counters are on the row.
- Row results: keyset on `(import_job_id, id)`; the *failed* tab uses the partial index, so it never scans good rows.
- Template download is static text: no database call.

**Targets (estimates, to be measured):** accept an upload in under 50 ms plus transfer time; process ~10,000 rows in about
1–2 s; history page 2–5 ms.

### 10.4 Build order
1. Migration: new `import_jobs` columns.
2. Template download and history list / get.
3. Upload endpoint: header check, job row, `Operation`.
4. Worker: streaming parse, validation, batched row inserts, counters, finish.
5. Row results endpoint and cancel.
6. Load test with a 10 MB / 50,000-row file.

### 10.5 Decisions for review
1. **What does a consignment import produce?** Orders in which system or table (this service has no orders table)? Does the
   worker call an order service, and is that call per row or batched?
2. **Duplicates:** what makes a row a duplicate (same *Order ref*)? Skip and report, or update?
3. **Keep the original CSV?** My proposal: no, keep only parsed rows (smaller, simpler). Keep it only if audit needs the file.
4. **Upload style:** direct multipart (≤ 10 MB, my proposal) or a presigned-URL upload?
5. **Limits:** 10 MB is on the screen; set a row cap (for example 50,000) too?
6. **Row rules:** which columns are really required (*Order ref* only, per the screen)? Date format? Currency for *Amount*?
7. **Who may upload** (`permission_key` for this module)?
8. **Retention:** how long to keep jobs and their rows?

---

---

# Part F — Products & Rules (rule engine)

## 11. Products & Rules

Screen: Catalog & Pricing → Products & Rules (one page, three rule areas + a side panel).

| Area | Text on screen | Rule engine? |
|---|---|---|
| **Pre-Enrollment Products** | *"Products visible before users enroll."* Product Selection rules (`INCLUDE`, conditions, `incomplete` badge), **Add product rule**, a **Rules / Custom CSV** switch, and the setting **Pricing Visible Pre Enrollment** (checkbox). | **Yes** (`pre_enrollment`) |
| **Post-Enrollment Products** | *"Products available after users enroll."* Same builder as above. | **Yes** (`post_enrollment`) |
| **Warranty** | *"Match product attributes to warranty durations in days."* Warranty rules: `IF <Field> <Value>` (+ **Add OR condition**, **AND group**) `THEN Warranty <N> days`; **Add warranty rule**; **Replicate Above**; Rules / Custom CSV. | **Yes** (`warranty`) |
| **Live results** (side panel) | *Auto-updated*; *"Add a complete rule to see matched products."* + **Add product manually**. | Uses the same engine (preview) |

Registry modules (already seeded): `catalog_pricing.products_rules.pre_enrollment`, `.post_enrollment`, `.warranty`, `.live_results`.

### 11.1 The key idea: a guided rule model, compiled to the engine

Admins **never edit a raw decision graph**. The screen edits a small, **typed rule model** (JSON). The server validates it and
**compiles it to a GoRules JDM** that the embedded `zen-engine` (the company fork already in `Cargo.toml`) evaluates.

- Source of truth = the rule model (what the builder draws). JDM = generated output, never hand-edited, never shown.
- A type mistake is impossible to *enter* (field and operator lists come from the server) and impossible to *publish*
  (server validation + test cases). This is the guardrail answer for non-technical users.
- The fork's **database / HTTP / function nodes are never generated**, so rules cannot query a database or run code.

**Rule model (examples)**

Product Selection (pre / post):
```json
{
  "mode": "rules",
  "rules": [
    { "id": "r1", "effect": "include",
      "when": { "or": [
        { "and": [ { "field": "brand", "op": "eq", "value": "Apple" },
                   { "field": "category", "op": "in", "value": ["Screens", "Batteries"] } ] },
        { "field": "sku", "op": "starts_with", "value": "IP15" } ] } }
  ]
}
```
Warranty:
```json
{
  "mode": "rules",
  "rules": [
    { "id": "w1", "when": { "field": "category", "op": "eq", "value": "Screens" }, "days": 90 },
    { "id": "w2", "when": { "field": "brand", "op": "eq", "value": "Samsung" }, "days": 180 }
  ]
}
```
`when` is a tree: a condition `{field, op, value}`, or `{and:[…]}` / `{or:[…]}`. `incomplete` on the screen = a rule that fails
validation (no field, no value, wrong type, or empty group).

**Custom CSV mode** (the *Custom CSV* switch): the admin uploads a CSV instead of building rules.
- Pre / Post: one column `sku` (a list of products). Warranty: columns `field value → days` (e.g. `brand,Apple,365`).
- Parsed rows are stored (not the file); at publish they are loaded into an in-memory lookup (hash set / hash map).

### 11.2 Contract per rule family (typed input / output)

| Family | Input (one product) | Output | Fail-closed default |
|---|---|---|---|
| `pre_enrollment` | product attributes (see field catalog) + `program_id` | `{ "visible": bool }` | not visible |
| `post_enrollment` | product attributes + `program_id` | `{ "available": bool }` | not available |
| `warranty` | product attributes | `{ "days": int ≥ 0 }` | no warranty (0 days) |

**Field catalog** = the list behind the **Field…** dropdown. Each field has `key`, `label`, `type` (`text`, `number`, `bool`,
`enum`), the allowed operators for that type (`eq`, `ne`, `in`, `starts_with`, `gt`, `lt`, …) and, for enum fields, the
allowed values (from a `global-settings` library where one exists). Served by `GET /rule-fields`.

**Combination semantics (proposed, confirm in 11.9):** Pre / Post — a product is included if **any** `include` rule matches
(plus manual includes) and no manual exclude applies. Warranty — rules are **ordered; the first match wins**.

### 11.3 Tables

| Table | Used for | Status |
|---|---|---|
| `rule_sets` | One row per `(program, family)`: `draft_version`, `published_version`; add `settings JSONB` (holds *Pricing Visible Pre Enrollment*). | exists; add column |
| `rule_versions` | Immutable versions. Add: `mode` (`rules` / `csv`), `definition JSONB` (the rule model), `compiled JSONB` (the generated JDM), `valid BOOLEAN`, `validation_errors JSONB`, `created_by`, `size_bytes`. Keep `content_hash` (hash of `definition`, so an unchanged save creates no new version). | exists; add columns |
| `rule_version_entries` | Parsed **Custom CSV** rows: `(rule_version_id, key, value)`, PK `(rule_version_id, key)`. | **new** |
| `rule_manual_products` | **Add product manually**: `(rule_set_id, sku, effect: include/exclude, created_by)`, PK `(rule_set_id, sku)`. | **new** |
| `rule_test_cases` | Expected results: `(id, rule_set_id, name, input JSONB, expected JSONB)`. Run before every publish. | **new** |
| `publications` | The published bundle includes each family's compiled JDM + entries + manual products. | exists |
| `program_modules.config`, `audit_log`, `outbox` | module on/off, history, events. | exist |

**Storage decision:** the rule model and compiled JDM live **in Postgres (JSONB)** in the version row, **in the same
transaction** as the version, audit and outbox (no object storage, no orphan files). Limits: warn at 256 KB, reject at
1 MB per version; CSV entries capped at 50,000 rows. Version rows are **append-only**; nothing updates a published version.

**Seeded module changes (data only):** set `ui_component = 'rule_builder'` on `pre_enrollment`, `post_enrollment` and
`warranty` (today `rule_playground` / `field_form`), and `rule_family = 'warranty'` on the warranty leaf.

### 11.4 API list

Errors: `400` · `404` · `409` (version conflict) · `413` (CSV over limit) · `422` (invalid rule model / unknown field or
operator) · `500`. `{family}` ∈ `pre_enrollment`, `post_enrollment`, `warranty`.

| Method | Path | Request | Response |
|---|---|---|---|
| `GET` | `/rule-fields` | `family` | `{results:[{key, label, type, operators:[…], values?:[…]}]}` (feeds **Field…**) |
| `GET` | `/programs/{p}/rule-sets` | – | `{results:[{family, draft_version, published_version, rule_count, valid, has_unpublished_changes}]}` |
| `GET` | `/programs/{p}/rule-sets/{family}` | – | `{family, settings, draft: RuleVersion, published_version, manual_products_count}` |
| `PATCH` | `/programs/{p}/rule-sets/{family}` | `{settings:{pricing_visible_pre_enrollment:true}}` | `RuleSet` |
| `POST` | `/programs/{p}/rule-sets/{family}/versions` | `{mode:"rules", definition:{…}}` | `RuleVersion` (a new draft; `valid` + `validation_errors[]` always returned, saved even if incomplete) |
| `GET` | `/programs/{p}/rule-sets/{family}/versions` | paging | `{results:[RuleVersion]}` (history / rollback) |
| `GET` | `/programs/{p}/rule-sets/{family}/versions/{v}` | – | `RuleVersion` (+ `definition`) |
| `POST` | `/programs/{p}/rule-sets/{family}/csv` | `multipart`: `file` | `RuleVersion` (draft, `mode:"csv"`) + `{rows, errors:[{row, message}]}` |
| `GET` | `/programs/{p}/rule-sets/{family}/csv:template` | – | CSV header (`text/csv`) |
| `POST` | `/programs/{p}/rule-sets/{family}:simulate` | `{definition?\|version?, input:{…product…}}` | `{output:{…}, matched_rule_id?, trace?}` |
| `POST` | `/programs/{p}/rule-sets/{family}:preview` | `{definition?\|version?, max_results:100}` | `{matched:[products], match_count, truncated}` (**Live results**) |
| `GET` `POST` `DELETE` | `/programs/{p}/rule-sets/{family}/manual-products[/{sku}]` | `{sku, effect}` | `ManualProduct` / list (**Add product manually**) |
| `GET` `POST` `PATCH` `DELETE` | `/programs/{p}/rule-sets/{family}/test-cases[/{t}]` | `{name, input, expected}` | `TestCase` / list |
| `POST` | `/programs/{p}/rule-sets/{family}/test-cases:run` | `{definition?\|version?}` | `{passed, failed, results:[{test_case, passed, actual}]}` |
| `POST` | `/programs/{p}/publications` | – | `Operation`; **fails with `422` if any family is invalid or a test case fails** |

`RuleVersion`: `{id, version, status: draft|published|superseded, mode, definition, valid, validation_errors:[{rule_id, field, message}], size_bytes, created_by, create_time}`.

gRPC (same names): `ListRuleFields`, `ListRuleSets`, `GetRuleSet`, `UpdateRuleSet`, `CreateRuleVersion`, `ListRuleVersions`,
`GetRuleVersion`, `UploadRuleCsv`, `GetRuleCsvTemplate`, `SimulateRuleSet`, `PreviewRuleSet`, `ListManualProducts`,
`CreateManualProduct`, `DeleteManualProduct`, `ListRuleTestCases`, `CreateRuleTestCase`, `UpdateRuleTestCase`,
`DeleteRuleTestCase`, `RunRuleTestCases`, plus `CreatePublication`.

### 11.5 Normal flow

| # | Step | Call |
|---|---|---|
| 1 | Page opens | `GET /programs/{p}/rule-sets` and `GET /programs/{p}/rule-sets/{family}` ×3 (or one list call carrying the drafts) |
| 2 | **Field…** dropdown | `GET /rule-fields?family=…` (cached by the frontend) |
| 3 | Admin edits a rule | no call (builder state); **Live results** panel calls `:preview` (debounced) |
| 4 | Admin saves | `POST …/versions` → returns validity; incomplete rules are flagged, never published |
| 5 | Admin tests one product | `POST …:simulate` |
| 6 | Admin publishes | `POST /programs/{p}/publications` → validate all families → run test cases → write bundle (one transaction) |
| 7 | Runtime | pricing / catalog loads the bundle, compiles to in-memory engines, evaluates per product |

### 11.6 Safeguards (non-technical users)

1. **Typed contract:** fields, operators and value types come from the server; every save is validated against them (11.2).
2. **No free graphs:** JDM is generated from the rule model; database / HTTP / function nodes are never produced.
3. **Test cases gate publish:** one failing case blocks the publication and names the case.
4. **Draft ≠ live:** saves, simulate and preview touch drafts only; only **Publish** changes live results.
5. **Immutable versions + rollback:** a rollback is "create a new version from version N".
6. **Fail closed at runtime:** a rule error, timeout or out-of-contract output returns the safe default in 11.2 and raises an
   alert; it never throws and never returns a wrong visible/price result.
7. **Limits:** rule count, condition depth, value length, 1 MB per version, 50,000 CSV rows.
8. **Permissions:** `permission_key` per module; publish restricted to a smaller group.

### 11.7 Lowest-latency design

**Runtime (the hot path) — no database**
- The published bundle holds, per family, the compiled JDM + CSV entries + manual products. At load, each family is compiled
  **once** into an in-memory `DecisionEngine` (memory loader, keyed by content hash); CSV entries become a hash set / map.
- Per product: manual exclude → manual include → CSV lookup → engine evaluation. All in memory; **microseconds**, zero I/O.
- Optional LRU memo keyed `(program, version, product)`; invalidated automatically when the version changes.
- A new publication is swapped in atomically (build the new engines first, then replace the pointer): no request ever sees a
  half-loaded rule set.

**Admin**
- Page load: 1–4 indexed reads (`rule_sets`, latest `rule_versions` by `(rule_set_id, id DESC)`); the definition is one JSONB
  value read by primary key. Field catalog is static, served from memory.
- Save: **one transaction** — insert version (validated + compiled), update `rule_sets.draft_version`, `programs.config_version`
  / `config_hash`, `audit_log`, `outbox`. Unchanged definition (same `content_hash`) = no new version, no writes.
- **Live results (preview):** evaluates the draft against a **bounded** slice of the catalog (first page of matches, capped at
  100 returned and an estimated count up to 10,000), so cost is fixed regardless of catalog size. Response cached for ~30 s
  keyed by the definition hash; the frontend debounces keystrokes.
- CSV upload: stream-parse, validate, bulk insert with one `unnest` statement per ~1,000 rows; one version row.
- Publish: validation + test cases run in memory on the drafts, then one transaction writes the bundle + publication + outbox.

**Targets (estimates, to be measured):** product evaluation < 50 µs in memory; save under 15 ms; page load 3–6 ms DB time;
preview under 150 ms; publish (3 families, 50 test cases) under 300 ms.

### 11.8 Build order

1. **Contract and model:** rule model types, field catalog (`GET /rule-fields`), validation, unit tests for every operator and type.
2. **Compiler:** rule model → JDM, evaluated by the fork's `zen-engine`; golden tests (model in, JDM + results out).
3. **Migration:** new columns and tables from 11.3; update the three seeded modules.
4. **Rule sets and versions:** get, save (validated), list, rollback.
5. **Simulate** and **test cases** (CRUD + run).
6. **Manual products** and **Custom CSV** (upload, template, entries).
7. **Preview** (Live results) against the catalog source.
8. **Publish integration:** bundle + gate on validity and test cases.
9. **Runtime evaluator:** loader, atomic swap, fail-closed defaults, benchmarks.
10. Load test and measure the targets.

Each step ships HTTP + gRPC + `openapi.yaml` + Postman and is tested live before the next.

### 11.9 Decisions for review

1. **Field catalog source:** which product attributes can a rule use (brand, category, model, sku, price, …) and **where do products
   live** (a catalog service, a search index, StarRocks)? This also feeds **Live results**.
2. **Combination rules:** confirm "any include matches → included; manual exclude wins" for Pre / Post, and "ordered, first match
   wins" for Warranty. Is there an `EXCLUDE` effect (the screen shows only `INCLUDE`)?
3. **Warranty default:** when no rule matches, 0 days, or a program default?
4. **Who evaluates at runtime:** embed the engine in the pricing / catalog service (fastest, proposed) or call this service?
5. **Pricing Visible Pre Enrollment:** confirm it is a flag stored on the Pre-Enrollment rule set and applied by the same bundle.
6. **Custom CSV columns:** exact columns for Pre / Post and for Warranty; replace-all on each upload (proposed) or merge?
7. **Replicate Above** (Warranty): copies the previous rule row on screen only (no API) — confirm.
8. **Rule limits:** max rules per family, max condition depth, CSV row cap (proposed 50,000).
9. **Who may edit and who may publish** (`permission_key` for these three modules)?
10. **Database node:** confirm it stays **off** for these rules (proposed; it adds latency and injection risk).

---

# Part G — Enrollment Requirements

## 12. Enrollment Requirements

Screen: Configure → Enrollment Requirements. *"What a store must satisfy to enrol. This ordered list IS the program's
application form — custom fields author its inputs, and a Contract requirement is what makes the program demand a signed
agreement."* Edited as a draft with **Discard** and **Save requirements**; the whole list is saved at once.

**Add a requirement:** `+ Sign-up`, `+ Card on file`, `+ Contract`, `+ Credit application`, `+ Custom field`. A type that is
already in the list is greyed out, **except Custom field, which can be added repeatedly**. Each row has move-up / move-down
and delete. Every row reads *"Required to complete enrollment."*

| Type | Settings on screen |
|---|---|
| Sign-up | none |
| Card on file | none |
| Credit application | none |
| Custom field | **Field label** (required — *"A field label is required."*) and **Validation (optional)**, placeholder `e.g. email, function:leadTime, regex:^\d+$` |
| Contract | **Agreement template id (optional)** — *"Link a UUID now, or attach an agreement later"* |

A banner on a Contract row: *"Saving adds the **Agreements** section, where you pick the e-Sign template to send."*

Registry module (already seeded): `enrollment.requirements.enrollment_requirements` (`custom:enrollment_requirements`).

### 12.1 Tables

| Table | Used for | Status |
|---|---|---|
| `requirements` | One row per requirement: `program_id`, `type`, `name`, `config JSONB`, `position`. | exists; add index |
| `program_modules` | Turned on / off for the three Agreements modules when a Contract requirement is saved / removed. | exists |
| `agreement_templates` | Validates (and lists, for the picker) the template a Contract requirement points to. | exists |
| `applications` | The store's answers to the form: **add `answers JSONB`** keyed by requirement id (validated at submit against the frozen `requirements_snapshot`). | exists; add column |
| `programs` | `config_version` / `config_hash` bumped on every save (also the optimistic-concurrency `version`). | exists |
| `audit_log`, `outbox` | History row and event in the same transaction. | exist |

**Index change:** partial unique `(program_id, type) WHERE type <> 'custom_field'` — the database itself enforces "one of each
fixed type" (the greyed-out buttons). `position` is **not** unique (a reorder rewrites many positions in one statement).

**`config` shapes**
```json
{ "type": "signup" }                    // no config
{ "type": "card_on_file" }              // no config
{ "type": "credit_application" }        // no config
{ "type": "contract",     "config": { "agreement_template_id": "uuid-or-null" } }
{ "type": "custom_field", "config": { "label": "Reseller Tax ID",
                                     "validation": { "kind": "regex", "arg": "^\\d+$" } } }
```
`validation.kind` ∈ `none`, `email`, `number`, `regex`, `function`.

### 12.2 API list

Errors: `400` · `404` · `409` (stale `version`) · `422` (item errors, listed per item) · `500`.

| Method | Path | Request | Response |
|---|---|---|---|
| `GET` | `/requirement-types` | – | `{results:[{type, label, description, once_per_program, config_schema}]}` (builds the **Add a requirement** buttons) |
| `GET` | `/programs/{p}/requirements` | – | `{version, results:[Requirement]}` in order |
| `PUT` | `/programs/{p}/requirements` | `{version, items:[{id?, type, config?}]}` (the **whole** ordered list) | `{version, results:[Requirement]}` — or `422` with `details[]` per item; `409` if `version` is stale |

`Requirement`: `{id, type, name, description, required: true, position, config}`.
`description` is the fixed text of the type ("Required to complete enrollment."); for `custom_field`, `name` is the label.

**`PUT` semantics (AEP-137 apply):** items **with** an `id` are kept (config updated, position set from their order); items
**without** an `id` are created; stored items **missing** from the request are deleted. One transaction.
**Item errors** use the standard `details` array so the screen can place them next to the row, for example
`{"field": "items[3].config.label", "description": "A field label is required."}`.

The text form typed into **Validation** (`email`, `regex:^\d+$`, `function:leadTime`) is accepted by the API and converted to
the typed `{kind, arg}` form on save; reads return the typed form (plus the original `text` for display).

gRPC (same names): `ListRequirementTypes`, `GetRequirements`, `ReplaceRequirements`.

### 12.3 Normal flow

| # | Step | Call |
|---|---|---|
| 1 | Page opens | `GET /requirement-types` (cached) and `GET /programs/{p}/requirements` |
| 2 | Admin adds, reorders, deletes, edits | no call (draft in the browser; **Discard** drops it) |
| 3 | **Save requirements** | `PUT /programs/{p}/requirements` with the full list |
| 3a | Server validates, diffs, then one transaction: requirements, Agreements modules on/off, version bump, `audit_log`, `outbox` | — |
| 4 | Storefront shows the form | reads the same list (cached in memory per program) |
| 5 | A store submits an application | the list is frozen into `requirements_snapshot`; answers are stored in `answers` |

### 12.4 Rules and safeguards

1. **Fixed types only:** `signup`, `card_on_file`, `credit_application`, `contract`, `custom_field` (unknown type = `422`).
2. **One of each fixed type** (index, plus a friendly `422`); custom fields repeatable, **capped** (proposed 15) with a **label
   length cap** (proposed 80).
3. **Typed validation, not free text:** `email` / `number` come from a fixed list; **`regex` is compiled on save** and rejected
   if invalid or longer than the cap (proposed 200 characters); Rust's `regex` engine has no catastrophic backtracking.
   **`function:<name>` may only name a function from a registered list.** Unknown name = `422`.
4. **Contract template must exist** and be active; it stays optional (can be attached later in Agreements).
5. **Saving a Contract turns the Agreements section on:** the three Agreements modules (Enrollment Agreements, Agreement
   Settings, Completion E-mail) are set `enabled` in `program_modules` in the same transaction. **Removing the Contract
   switches them off but keeps their data.**
6. **Applications are protected:** the list is frozen into `requirements_snapshot` at submit, so editing requirements never
   changes an application already in review.
7. **Optimistic concurrency:** `version` must match, so two admins cannot overwrite each other.

### 12.5 Lowest-latency design

- The list is 5–20 rows: **one indexed read** `(program_id, position, id)`, kept **in memory per program** and replaced when
  `config_version` changes (no explicit invalidation).
- **Save is one request and one transaction:** validate in memory first, then a diff (`DELETE` missing ids, one `UPDATE …
  FROM unnest(…)` for kept rows' config and positions, one `INSERT … unnest` for new rows), the Agreements toggle,
  `programs` version bump, `audit_log`, `outbox`. No per-row round trips; a no-change save writes nothing.
- Derived values other modules need ("Enrollment Required", "Agreement Required" in Program Settings) are `EXISTS` checks on
  the `(program_id, type)` index, or computed from the in-memory list.
- No Publish step: requirements are not a pricing input; they are read live and protected by the snapshot.
- Type catalog (`GET /requirement-types`) is static, served from memory.

**Targets (estimates, to be measured):** read under 1 ms from memory (2 ms cold DB); save under 15 ms.

### 12.6 Build order

1. Migration: partial unique index; `applications.answers`.
2. Requirement-type catalog and the typed validation (email / number / regex / function), with unit tests.
3. `GET` list.
4. `PUT` replace-all: diff, per-item errors, version check, audit, outbox.
5. Contract → Agreements modules toggle.
6. Frozen snapshot hook for applications (with Part B).
7. Load test (concurrent saves on one program).

Each step ships HTTP + gRPC + `openapi.yaml` + Postman and is tested live before the next.

### 12.7 Decisions for review

1. **`function:leadTime`:** what are these functions, and where are they defined (a registered list in code, or per program)?
2. **Regex:** confirm the length cap and that only the `^…$` style is allowed.
3. **Credit application:** the row has no settings, so where do its form fields come from (a fixed form, or another module)?
4. **Removing a Contract:** switch the Agreements modules off immediately even if agreements already exist (data kept)?
5. **Custom field:** always a text input (proposed), or other input types later?
6. **Contract template picker:** a dropdown of `agreement_templates` (proposed) rather than pasting a UUID?
7. **Caps:** maximum custom fields (proposed 15), label length (80)?
8. **Who may edit** requirements (`permission_key` for this module)?

---

# Part H — Documents

## 13. Documents

Screen: Configure → Documents. A file manager for the program.

- **Header:** *All files · N files · X KB*, **Search**, **Upload**, list / grid toggle (screen only).
- **Locations:** All files, Recent, **Needs review**, **Quarantined**, each with a count.
- **Folders:** General, Import artifacts, Program Documents, Store Enrollment Documents, each with a count.
- **Footer:** *Storage — X KB used*. Empty state: *"No files yet. Nothing has been uploaded here yet."* with **Upload files**.

Registry module (already seeded): `document_agreement.documents.documents_panel` (`custom:documents`).

### 13.1 Why object storage here (and not Postgres)

Rule graphs and requirements are small JSON, so they live in Postgres. **Documents are arbitrary files (PDF, images, office
files) up to tens of MB**, so the **bytes go to object storage (S3)** and Postgres keeps only the metadata. The upload and
download URLs are **presigned**: the browser talks to S3 directly, so file bytes never pass through the programs service.

### 13.2 Tables

| Table | Used for | Status |
|---|---|---|
| `documents` | One row per file: name, folder, type, size, `file_key`, status, source. | exists; add columns |
| `document_folders` | Folders, incl. the four system folders per program. | exists; add columns |
| `program_storage` | **One small row per program** with the counters the screen shows: `file_count`, `total_bytes`, `needs_review_count`, `quarantined_count`. PK `program_id`. | **new** |
| `audit_log`, `outbox` | Upload / review / delete history; scan and review events. | exist |

**`documents` — new columns**

| Column | Why |
|---|---|
| `status` | `uploading` → `pending_scan` → `ok` \| `needs_review` \| `quarantined`. Drives **Needs review** / **Quarantined**. |
| `source_type`, `source_id` | Who created it: `import_job`, `enrollment`, `application`, `agreement`, or none (manual upload). Feeds **Import artifacts** / **Store Enrollment Documents**. |
| `uploaded_by`, `checksum` (SHA-256), `scan_time` | Audit and integrity. |
| `review_note`, `reviewed_by`, `reviewed_time` | Result of a manual review. |
| `state` | `ACTIVE` / `DELETED` (soft delete, like the other resources). |

**`document_folders` — new columns:** `key` (`general`, `import_artifacts`, `program_documents`, `store_enrollment_documents`,
NULL for custom), `system BOOLEAN` (system folders cannot be renamed or deleted), `document_count INT`.
The four system folders are **created with each program**.

**Indexes**
- `documents (program_id, id DESC) WHERE state = 'ACTIVE'` — *All files* and *Recent* (UUIDv7 ids are in time order).
- Partial `documents (program_id, id) WHERE status = 'needs_review'` and `… WHERE status = 'quarantined'` — those two
  locations scan almost nothing.
- Keep `(program_id, folder_id, id)` for folder views.
- `(program_id, lower(name) text_pattern_ops)` for prefix search (a trigram index only if contains-search is needed).
- Unique `(program_id, key) WHERE key IS NOT NULL` on folders; unique `(program_id, parent_id, lower(name))`.

**Counters** (`program_storage`, `document_folders.document_count`) are updated **in the same transaction** as every
upload-complete, delete, move and status change, so the screen never runs `COUNT(*)`.

### 13.3 API list

Errors: `400` · `404` · `409` (folder not empty / system folder / wrong status) · `413` (over size limit) · `415` (type not
allowed) · `422` · `500`.

| Method | Path | Request | Response |
|---|---|---|---|
| `GET` | `/programs/{p}/documents:summary` | – | `{files, bytes, locations:{all, recent, needs_review, quarantined}, folders:[{id, key, name, system, document_count}]}` — header, left panel and storage in **one call** |
| `GET` `POST` `PATCH` `DELETE` | `/programs/{p}/document-folders[/{f}]` | `{name, parent_id?}` | `Folder` / list (system folders: no rename / delete; non-empty: `409`) |
| `GET` | `/programs/{p}/documents` | `folder_id?`, `location?` (`recent`\|`needs_review`\|`quarantined`), `q?` (name prefix), `max_page_size`, `page_token` | `{results:[Document], next_page_token}` |
| `POST` | `/programs/{p}/documents` | `{name, folder_id?, content_type, size_bytes, checksum?}` | `{document (status uploading), upload:{url, method:"PUT", headers:{…}, expires_time}}` |
| `POST` | `/programs/{p}/documents/{d}:complete` | – | `Document` (status `pending_scan`; the server checks the object exists, size and checksum match) |
| `GET` | `/programs/{p}/documents/{d}` | – | `Document` |
| `PATCH` | `/programs/{p}/documents/{d}` | `{name?, folder_id?}` (rename / move) | `Document` |
| `DELETE` | `/programs/{p}/documents/{d}` | – | `204` (soft delete) |
| `POST` | `/programs/{p}/documents/{d}:download` | – | `{url, expires_time}` (presigned, short-lived; not allowed while `quarantined`) |
| `POST` | `/programs/{p}/documents/{d}:review` | `{decision: "approve"\|"reject", note?}` | `Document` (`ok` or `quarantined`) |
| `POST` | `/webhooks/document-scan` | `{document_id, result: "clean"\|"suspicious"\|"infected", scanner}` | `204` (internal; the scanner's callback) |

`Document`: `{id, name, folder_id, content_type, size_bytes, status, source_type, source_id, uploaded_by, create_time, update_time}`.

gRPC (same names): `GetDocumentSummary`, `ListDocumentFolders`, `CreateDocumentFolder`, `UpdateDocumentFolder`,
`DeleteDocumentFolder`, `ListDocuments`, `CreateDocument` (returns the upload instructions), `CompleteDocument`,
`GetDocument`, `UpdateDocument`, `DeleteDocument`, `DownloadDocument`, `ReviewDocument`. File bytes are **never** sent over
gRPC or REST — only through the presigned URLs.

### 13.4 Normal flow

| # | Step | Call |
|---|---|---|
| 1 | Page opens | `GET …/documents:summary` and `GET …/documents` (default: *All files*) |
| 2 | Admin picks a folder / location, types in Search | `GET …/documents?folder_id=…` / `?location=…` / `?q=…` |
| 3 | Admin uploads | `POST …/documents` → browser `PUT`s the file to the returned S3 URL → `POST …/documents/{d}:complete` |
| 4 | Scanner finishes | `POST /webhooks/document-scan` → status `ok` / `needs_review` / `quarantined`; counters updated |
| 5 | Admin reviews | `POST …/documents/{d}:review` |
| 6 | Admin downloads | `POST …/documents/{d}:download` → open the returned URL |
| 7 | Rename, move, delete | `PATCH` / `DELETE` |

### 13.5 Safeguards

1. **Row first, file second, visible last:** the row exists as `uploading` before any byte is sent and is hidden from every
   list until `:complete`. A file can therefore never be a row pointing at nothing, and every stray file belongs to a row we
   can find. A job removes rows stuck in `uploading` (proposed: older than 1 hour) and their objects.
2. **Presigned URLs are narrow:** one exact object key, the declared size and content type, expiry about 10 minutes.
3. **Object keys are generated by the server** (`programs/{program}/documents/{document_id}`); user-supplied names are only
   labels and are sanitized, so a name can never choose a path.
4. **Integrity:** SHA-256 checked at `:complete`; size and content type must match what was declared.
5. **Quarantine:** nothing becomes `ok` until scanned; `quarantined` files cannot be downloaded.
6. **Limits:** maximum file size (proposed **25 MB**), allowed types (proposed pdf, png, jpg, docx, xlsx, csv, txt), maximum
   files per folder / per program quota.
7. **Bucket:** private, encrypted at rest, versioning on, CORS limited to the admin origin, lifecycle rule that removes
   objects of soft-deleted documents after the retention period.

### 13.6 Lowest-latency design

- **Left panel + header = one call:** one primary-key read of `program_storage` plus the folders list (a handful of rows,
  counts already on them). **No `COUNT(*)` anywhere.**
- **File lists:** keyset on the indexes above; *Recent* is `ORDER BY id DESC LIMIT 20` on the `(program_id, id DESC)` index;
  *Needs review* and *Quarantined* scan tiny partial indexes.
- **Upload and download cost the service nothing:** the bytes go browser ↔ S3. The API only does small metadata writes:
  create (one insert + counters untouched until complete), complete (one update + counter increments in one transaction).
- **Scan results** arrive by webhook and are one `UPDATE` plus counter adjustments.
- Search is an index prefix match; list / grid is a frontend toggle.

**Targets (estimates, to be measured):** summary 1–2 ms DB time; list page 2–5 ms; create / complete under 10 ms each.

### 13.7 Build order

1. Migration: new `documents` and `document_folders` columns, `program_storage`, indexes; create the system folders with each
   program (and backfill existing ones).
2. Summary and list endpoints (with counters).
3. Folder CRUD with system-folder rules.
4. Upload: create → presigned PUT → complete (checksum / size check), counters, stuck-upload cleanup job.
5. Download (presigned GET) and soft delete.
6. Scan webhook, status transitions, review.
7. Cross-module sources (import artifacts, enrollment documents).
8. Load test summary and list on a program with 50,000 documents.

Each step ships HTTP + gRPC + `openapi.yaml` + Postman and is tested live before the next.

### 13.8 Decisions for review

1. **Scanning:** is there an existing virus scanner, and what exactly produces **Quarantined** (infected only, or also suspicious)?
2. **Needs review:** what puts a file there (every store-uploaded file, a suspicious scan, a rule)?
3. **Import artifacts:** should an uploaded import CSV be kept here as a document? **This reverses decision 3 in section 10.5**
   (which proposed not keeping the original file); decide both together.
4. **Folders:** flat as on screen, or nested (`parent_id` supports it)? Are the four system folders the full default set?
5. **Limits:** maximum file size, allowed types, and per-program storage quota. Is *"Storage — X KB used"* a quota or only usage?
6. **Quarantined files:** can an admin release one (a `:release` action), or only delete it?
7. **Visibility:** does *All files* include `needs_review` and `pending_scan` files (proposed: yes, with a badge) and exclude
   `quarantined` (proposed: yes)?
8. **Retention:** how long soft-deleted files are kept before the object is removed.
9. **Permissions:** who can upload, review, download and delete (`permission_key` for this module)?
10. **Object storage:** which bucket / prefix, and does an existing service already own document storage we should reuse?

## 14. Out of scope here
Discount Rules and Markup (rule screens in Pricing), Price Check / Price Impact, the Publish *screen* itself (only its
validation gate is covered in 11), agreements (Enrollment Agreements, Agreement Settings, Completion E-mail) and Reports. Their modules are seeded but not wired.
