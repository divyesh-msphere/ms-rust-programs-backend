You are a senior backend engineer and a patient teacher. I am learning a Rust backend service called "Programs". I will give you everything about ONE module of it: the first one, "Module catalog". Teach it to me.

Rules for you:
- Use ONLY the facts below. If something is not covered, say "this is not in what you gave me" instead of guessing.
- Explain in plain, simple language. Assume I am smart but new to this codebase. Define every technical word the first time you use it.
- Use small examples from the real data below, not made-up ones.

=====================================================================
1. THE SYSTEM (just enough context)
=====================================================================
The service manages "programs" (offerings a store can join, such as KBB, CPO, Credit Line). An admin builds a program by choosing which PAGES it has. Every page of the admin screen is a "module". A program only has the modules its admin picked.

The admin screen of one program has three tabs: Overview, Configure, Operate. Pages sit under them (for example Operate > Applications, Configure > Documents).

Tech: Rust, actix-web (HTTP), connectrpc (gRPC), Postgres (sqlx, hand-written SQL), AEP-style REST (resource paths, "?parent_id=", ":custom" methods). Layered code: route -> mappers -> use case -> domain -> repository -> database.

=====================================================================
2. THIS MODULE: "Module catalog" (the catalog)
=====================================================================
Purpose: the read-only catalog of EVERY page that can exist, arranged as a tree of groups and leaves. The frontend and the program builder use it to know what pages exist and how each behaves (which code handles it, which API backs it, whether the admin can pick it). It is data, not code: adding a page means adding a row in a migration, not deploying new code. There is NO write API: the rows come from seed migrations.

Two kinds of node:
- group = a menu heading that has children (for example "Enrollment")
- leaf  = a real page (for example "Application Panel")

Three kinds of source:
- default   = on every program, the admin cannot pick it (only "Overview")
- premade   = a page written in code; the admin only switches it on
- container = an empty page; the admin decides what it becomes (a Rule page or a Settings page)

=====================================================================
3. THE TABLE (exact definition from the live database)
=====================================================================
Table public.module_definitions

  id                uuid         NOT NULL   primary key
  key               text         NOT NULL   UNIQUE
  name              text         NOT NULL
  kind              text         NOT NULL   ('group' or 'leaf')
  parent_id         uuid         NULL       foreign key -> module_definitions(id) ON DELETE RESTRICT
  ancestry          ltree        NOT NULL   (Postgres ltree path)
  position          integer      NOT NULL   default 0
  ui_component      text         NULL
  rule_family       text         NULL
  universal         boolean      NOT NULL   default false
  requires_publish  boolean      NOT NULL   default false
  permission_key    text         NULL
  source            text         NOT NULL   default 'container'   ('default' | 'premade' | 'container')
  handler_key       text         NULL
  api_base          text         NULL
  selectable        boolean      NOT NULL   default true
  allow_components  boolean      NOT NULL   default false
  default_mode      text         NULL       'rule' or 'settings': the mode a program gets on selecting the page (only Fees has it)
  default_rule      jsonb        NULL       a starter rule saved as draft 1 for a rule page (only Fees has it)
  create_time       timestamptz  NOT NULL   default now()
  update_time       timestamptz  NOT NULL   default now()

Indexes:
  module_definitions_pkey                   btree (id)
  module_definitions_key_key                UNIQUE btree (key)
  idx_module_definitions_ancestry_btree     btree (ancestry)
  idx_module_definitions_ancestry_gist      gist  (ancestry)
  idx_module_definitions_parent_position_id btree (parent_id, position, id)

Referenced by:
  module_definitions.parent_id   (the tree points at itself)
  program_modules.module_id      (a program's chosen modules) ON DELETE RESTRICT, so a module that any program uses cannot be deleted

What the columns mean:
  key              the module's unique dotted name, for example "enrollment.applications_approvals.application_panel". Programs refer to modules by this.
  name             the label shown to the admin ("Application Panel")
  kind             group or leaf
  parent_id        the parent node (NULL for the six top-level modules)
  ancestry         the same path as the key, stored as an ltree, so "everything below this node" is ONE indexed query (operator <@)
  position         display order among siblings
  source           default / premade / container (see above)
  handler_key      which code handler a premade page uses (for example "applications")
  api_base         which API backs the page (for example "applications")
  ui_component     which frontend component renders it (for example "custom:applications")
  rule_family      for rule pages: the family of rules (for example "pre_enrollment", "discount")
  requires_publish true when edits to this page only go live after "Publish"
  selectable       false only for Overview (it is added automatically, the admin cannot pick it)
  allow_components true when the leaf can host admin-added components (settings or rules)
  universal        true only for Overview
  permission_key   reserved for authorization

=====================================================================
4. THE REAL DATA
=====================================================================
71 rows. 6 top-level modules. Maximum depth: 4 levels.
  container group: 19     container leaf: 46     default leaf: 1     premade leaf: 5

Top-level modules (position, key, name, kind, source):
  0  overview                Overview                   leaf   default
  1  catalog_pricing         Catalog & Pricing          group  container
  2  enrollment              Enrollment                 group  container
  3  document_agreement      Document & Agreements      group  container
  4  order_transaction       Order & Transaction        group  container
  5  configuration_oversight Configuration & Oversight  group  container

The 5 premade leaves: Enrollment Requirements (handler "requirements"), Enrolled Stores Control Panel ("enrolled_stores"), Application Panel ("applications"), Store Discounts Control Panel ("store_discounts"), Documents Panel ("documents").

The whole "Enrollment" branch as the API returns it (full tree):
  Enrollment  [group, container]
    Requirements  [group, container]
      Enrollment Requirements  [leaf, premade, handler=requirements]
    Stores  [group, container]
      Enrolled Stores Control Panel  [leaf, premade, handler=enrolled_stores]
    Applications & Approvals  [group, container]
      Application Panel  [leaf, premade, handler=applications]

One row as JSON (the Overview, the "default" module):
  {
    "path": "module-catalog/0239b5f9-9a53-8a52-284f-38e02d62a0ab",
    "id": "0239b5f9-9a53-8a52-284f-38e02d62a0ab",
    "key": "overview", "name": "Overview", "kind": "leaf",
    "parent_id": null, "ancestry": "overview", "position": 0,
    "ui_component": "custom:overview", "rule_family": null,
    "universal": true, "requires_publish": false, "permission_key": null,
    "source": "default", "handler_key": "overview", "api_base": "overview",
    "selectable": false, "allow_components": false
  }

One row as JSON (a premade leaf):
  {
    "path": "module-catalog/90576b1c-18a7-a4fb-e6ce-cecfe0ef04fc",
    "id": "90576b1c-18a7-a4fb-e6ce-cecfe0ef04fc",
    "key": "enrollment.applications_approvals.application_panel",
    "name": "Application Panel", "kind": "leaf",
    "parent_id": "1cc6521a-31bd-ca47-e044-5a2396880535",
    "ancestry": "enrollment.applications_approvals.application_panel", "position": 1,
    "ui_component": "custom:applications", "rule_family": null,
    "universal": false, "requires_publish": false, "permission_key": null,
    "source": "premade", "handler_key": "applications", "api_base": "applications",
    "selectable": true, "allow_components": true
  }

=====================================================================
5. THE APIs
=====================================================================
HTTP (2 endpoints, both read-only):

  GET /module-catalog
     query parameters (all optional):
       parent_id       uuid   list the DIRECT children of this module; omitted = the 6 top-level modules
       max_page_size   int    default 50, at most 200, must be >= 1
       page_token      uuid   the last item of the previous page
       full_tree       bool   true = ignore paging and return the whole tree (or, with parent_id, everything beneath that module), nested under "children"
     response: { "results": [ ...modules... ], "next_page_token": null or uuid, "total_size": number }
     Example: with no parameters it returns the 6 top-level modules and total_size = 6.

  GET /module-catalog/{module_catalog_item}
     {module_catalog_item} is the id (UUID) OR the key, for example /module-catalog/enrollment
     response: one module (the JSON above). If it is a UUID the SQL looks up by id, otherwise by key.

gRPC (same data, same rules; the service is "ProgramsService"):
  ListModuleCatalog(parent_id?, max_page_size?, page_token?, full_tree?) -> results, next_page_token, total_size (and the nested tree when full_tree is true)
  GetModuleCatalogItem(module_catalog_item = id or key, path) -> the module
  (Both are tagged for authorization as permission "programs.module_definitions", action "reader".)

Errors seen live:
  GET /module-catalog/no.such.module   -> 404 {"type":"NOT_FOUND","detail":"Module catalog item with key no.such.module not found"}
  GET /module-catalog?parent_id=nope   -> 400 {"type":"INVALID_ARGUMENT","detail":"Validation error: parent_id must be a UUID"}
  GET /module-catalog?max_page_size=0  -> 400 {"type":"INVALID_ARGUMENT","detail":"Validation error: max_page_size must be at least 1"}

=====================================================================
6. HOW IT WORKS INSIDE (the real SQL)
=====================================================================
Get one (by id or by key):
  SELECT <columns> FROM module_definitions WHERE id  = $1     -- when the identifier is a UUID
  SELECT <columns> FROM module_definitions WHERE key = $1     -- otherwise
  no row -> 404

List one level (keyset paging on (position, id), not OFFSET):
  SELECT COUNT(*) FROM module_definitions WHERE parent_id IS NOT DISTINCT FROM $1     -- total_size
  SELECT <columns> FROM module_definitions
   WHERE parent_id IS NOT DISTINCT FROM $1
     AND ((position, id) > (SELECT position, id FROM module_definitions WHERE id = $2))   -- only when a page_token is given
   ORDER BY position, id LIMIT $3                                                         -- $3 = page size + 1, to know if there is a next page
  ("IS NOT DISTINCT FROM" makes NULL match NULL, so one query covers both the top level and a given parent.)

Full tree (ONE query, then nested in memory):
  SELECT <columns> FROM module_definitions ORDER BY position, id                       -- everything
  SELECT <columns> FROM module_definitions
   WHERE ancestry <@ (SELECT ancestry FROM module_definitions WHERE id = $1)
   ORDER BY position, id                                                                 -- everything below one node (<@ = "descendant of", served by the gist index)
  The rows come back flat; a function (build_tree) groups them under their parents into nested "children".

Code layout (about 600 lines in total):
  src/domain/module_catalog_item.rs                                   the ModuleCatalogItem type, the repository TRAIT (get, list, list_descendants), build_tree
  src/adapters/services/postgres/module_catalog_item/                 the repository (all SQL), a model (database row) and a mapper (row -> domain)
  src/application/usecases/module_catalog_item/                       list_module_definitions.rs, get_module_catalog_item.rs, get_module_catalog_tree.rs
  src/adapters/inputs/http_apis/module_catalog_item/                  the HTTP controller and mappers
  src/adapters/inputs/grpc_apis/module_catalog_item/                  the gRPC service and mappers
  Seed data (these migrations create the 71 rows): 20261014100000_seed_root_modules, 20261014110000_seed_catalog_pricing_modules, 20261014120000_seed_configuration_oversight_modules, 20261015100000_seed_document_agreement_modules, 20261015110000_seed_enrollment_modules, 20261015120000_seed_order_transaction_modules, 20261016100000_seed_module_classification (marks the premade leaves, their handler_key and api_base)

=====================================================================
7. HOW THE OTHER MODULES USE IT
=====================================================================
- When an admin creates a program they send module keys, for example  "modules": ["enrollment.stores.enrolled_stores_control_panel"].
  Step 1, check_keys: every key must exist and be selectable (Overview is not selectable, but every program already has it, so it is allowed). Otherwise the answer is 422 "unknown or non-selectable modules: <keys>".
    SELECT md.key FROM module_definitions md WHERE md.key = ANY($keys)
       AND (md.selectable OR EXISTS (SELECT 1 FROM program_modules pm WHERE pm.program_id = $program AND pm.module_id = md.id))
  Step 2, ensure_modules: one INSERT adds, into program_modules, Overview (source = 'default'), every picked module, ALL of their ancestors (so the tree stays connected) and everything under a picked group. The two ltree checks (s.ancestry <@ md.ancestry and md.ancestry <@ s.ancestry) are what find "ancestors" and "descendants" in one statement:
    INSERT INTO program_modules (program_id, module_id, enabled, position)
    SELECT $program, md.id, true, md.position FROM module_definitions md
     WHERE md.source = 'default'
        OR EXISTS (SELECT 1 FROM module_definitions s WHERE s.key = ANY($keys)
                   AND (s.ancestry <@ md.ancestry OR md.ancestry <@ s.ancestry))
    ON CONFLICT (program_id, module_id) DO NOTHING        -- modules the program already has are left exactly as they are
  That is why the tree (parent_id and ancestry) matters.
- The program's "manifest" (GET /programs/{program}/modules) is a JOIN of program_modules with module_definitions: it tells the frontend which pages the program has, which handler renders each (handler_key, ui_component) and which API backs it (api_base).
- Because program_modules.module_id is ON DELETE RESTRICT, a module catalog item that a program uses can never be deleted.
- Store Discounts and Documents check that the program has THEIR module switched on, by looking up their key in this catalog (for example "catalog_pricing.store_discounts.control_panel" and "document_agreement.documents.documents_panel").

=====================================================================
8. WHAT I WANT FROM YOU
=====================================================================
1. In plain language: what is this module for, and why is it a database table and not code?
2. Walk through EVERY column with a real example value from the data above. Group them (identity, tree, behavior, flags, timestamps).
3. Explain "ltree" and why the table has BOTH parent_id and ancestry. What does each one make easy? Why two indexes on ancestry (btree and gist)?
4. Explain keyset paging versus OFFSET paging using the list query above, and why "page size + 1" is fetched.
5. Take each API call (list roots, list children, full tree, get by key, get by id) and tell me, step by step, what SQL runs and what comes back, using the Enrollment branch as the example.
6. Draw an ASCII diagram of the layers (route -> use case -> domain -> repository -> database) for GET /module-catalog/{key}.
7. Explain the three "source" kinds and how they decide what the admin can do with a page.
8. Point out design trade-offs or risks you notice (for example: no write API, ON DELETE RESTRICT, seed-only data) and say what you would check next.
9. Finish with 8 quiz questions about this module (do NOT show the answers yet). After I answer, grade me and explain what I missed.
10. Before we move on, ask me which part is still unclear.
