# context-handoff.md — argocd-applicationsets-gitops

**Updated 2026-08-17.** Read PART 0 first: it is the current state and
supersedes the "how to fix" prose in Parts 1–2, which are RESOLVED and kept
only as institutional knowledge. PART 4 is the open work, headed by the
tactical map region selector.

## What this repository is

A production-grade **reference** showing how ArgoCD ApplicationSets replace the
App-of-Apps pattern. It is parked as a sibling folder to Broadcom's existing
`argocd-app-of-apps/`, handed over as a zip (no write access to their repo), and
pushed by their principal architect.

It is a **guide**, not a shipping product. Broadcom is the audience — not the
healthcare end client. Client-specific deltas belong in a README appendix, never
in the architecture.

The repository has three corners, and the whole argument depends on all three
being real:

```
apps/drone-convoy-tracker/                        the application (Rust + Containerfiles)
argocd-apps/cluster-apps/drone-convoy-tracker/    its Helm chart
argocd-apps/app-sets/templates/cluster-apps.yaml  the ApplicationSet that globs it up
```

Drop a chart directory into `cluster-apps/` and four Applications appear, one per
environment, with no per-app `Application.yaml` and no edit to any shared file.
That is the demo. Do not break it.

---

---

# PART 0 — READ THIS FIRST: STATE AS OF 2026-08-17

**Parts 1 and 2 below are RESOLVED.** They are kept verbatim as institutional
knowledge (the root-cause write-ups are the most valuable thing in this file),
but nothing in them is open work. Read Part 0 for what is true now and Part 4
for what is next. Do not "fix" anything Parts 1–2 describe as broken.

## 0.1 What exists now — two repositories

The app was split out of this repository into its own product repo:

    github.com/isgo-golgo13/argocd-applicationsets-gitops         <- THIS repo (Broadcom deliverable)
    github.com/isgo-golgo13/drone-convoy-attack-tracker-leptos-rs <- the app (Gumroad product)

This repo keeps `apps/drone-convoy-tracker/` as the workload the ApplicationSets
demonstrate; the standalone repo is where app feature work now happens. Sync
app changes across deliberately (delta zips), not by assumption. The standalone
repo was audited on 2026-08-14 after the rename: every workspace member, path
dep, `include_str!` asset path, `mod` declaration, Makefile/Containerfile/compose
reference and Trunk href resolves; `Cargo.toml` `repository` points at the new
URL; history squashed to one clean commit (~7 MB `.git`, screenshots slimmed to
~1.2 MB each).

## 0.2 The dashboard breathes — every div is live

Confirmed by screenshots 2026-08-13 22:30–22:35: leaderboard re-ranking with
target-roundel streaks, drone SVG cards with FUEL draining and ACC/WP tracking
per shot, status INGRESS→EGRESS→RTB, single feed scrollbar, live telemetry
chart and tooltip, four airframes in formation on the map, mission clock
ticking, ONLINE within ~2 s of page load. Zero seeds, zero stubs, zero
`rejected:` warnings in a clean simulator run, and the playground `leaderboard`
query matches the simulator's FINAL LEADERBOARD block exactly (that match is
the end-to-end integrity check — any delta = lost posts, each of which warned).

Delta zips shipped, in apply order (all superseded by the repo state, listed
for the record): server-live (15) → frontend-live (6) → build-fixes (4) →
sim-hardening → enum-pins → deterministic-ids → hud-polish (4) →
target-svg-asset → target-svg-fix → delivery (31, incl. app tweaks 1–3 + P4 +
P5) → cluster-secrets (3) → favicon → deploy-and-favicon (38) →
kind-setup-guide.

## 0.3 Root causes found this week — beyond Part 1.8, do not reintroduce

These are the ones discovered AFTER Part 1.8 was written. Each cost real time.

1. **Inflector digit bug (the big one).** async-graphql's
   `SCREAMING_SNAKE_CASE` rename runs through the Inflector crate, whose
   `char_is_uppercase` is `c == c.to_ascii_uppercase()` — TRUE for digits — so
   `Agm114Hellfire` registered as `AGM_114_HELLFIRE` and `Mq9Reaper` as
   `MQ_9_REAPER`. Every digit-bearing WeaponType/PlatformType value was
   rejected while every digit-free enum worked. Verified by porting
   Inflector 0.11.4 `to_case_snake_like` exactly. FIX: explicit
   `#[graphql(name = "...")]` pins on all nine digit variants. RULE: never
   trust `rename_items` for variants containing digits — pin them.
   (Part 1.8's claim that `Agm114Hellfire → AGM114_HELLFIRE` was wrong.)
2. **Leptos frozen-`<For>`-row bug.** `<For>` keeps a row's first-rendered
   view while its key exists. Cards keyed by `drone_id` alone froze
   FUEL/ACC/WP; leaderboard rows keyed by `drone_id` froze their numbers
   (which was the "sim final leaderboard ≠ UI" report). FIX: composite keys
   `(drone_id, updated_at)` on cards, `(drone_id, total_engagements,
   successful_hits, current_streak, rank)` on leaderboard. PATTERN: any
   `<For>` over polled data needs a content-bearing key.
3. **Bootstrap race.** The simulator's one-shot bootstrap fired before the
   API's release build finished under `make serve -j2` — connection refused,
   never retried; per-tick posts then landed without identity (all UNKNOWN,
   which also collapsed the map's per-callsign markers to one). FIX:
   `wait_for_api` (`{ health }` probe, ~3 min) before bootstrap, and
   identity (callsign/platformType/totalWaypoints) rides on EVERY per-tick
   `updateDroneState` (idempotent read-merge-write upsert → self-heals in one
   tick).
4. **Ghost drones.** Random v4 drone ids per sim start accumulated rows.
   FIX: `Uuid::new_v5(&Uuid::NAMESPACE_OID, callsign)` (+`v5` feature) —
   ALPHA-01 is `3c42d43b-fb70-55f0-b6f4-8a159226b5ee` on every machine, so
   restarts overwrite and playground examples are reproducible verbatim.
5. **XML `--` in comments.** `target-streak.svg`'s comment contained
   `var(--accent-primary)`; a double hyphen is illegal in XML comments, so
   the file was unparseable standalone (it rendered inlined only because
   `inline_svg` strips everything before `<svg>`). RULE: SVG assets validate
   via `xml.dom.minidom` before shipping; no `--` in comments (drone.svg's
   author already followed this — the precedent was there).
6. **Frontend compile gotchas.** E0525: passing whole `AppState` into the
   poll closure moved it and demoted the closure to `FnOnce` — pass the
   `Copy` `RwSignal` field instead (Rust 2021 disjoint capture). E0308:
   wasm_bindgen extern types have no own `Clone`; `.clone()` derefs to
   `JsValue` — capture by move. `threat_level_str` was missing
   `ThreatLevel::Unknown` (E0004; all seven `*_str` helpers audited, no `_`
   wildcards by design so the compiler catches drift).
7. **Charming re-render stacks instances.** `WasmRenderer::render()` calls
   echarts `init` every time; per-tick re-render stacked instances. FIX:
   render once, then `WasmRenderer::update` on the kept `Echarts` handle
   (verified in charming 0.4 source).
8. **Two dead-YAML finds in P4.** goldilocks prod declared `dashboard:` twice
   (last-key-wins had silently discarded the Ingress block); vault prod
   values were EMPTY (the handoff line about its Ingress was stale).

## 0.4 App additions this week (in the standalone repo)

- **Assets** (`assets/images/`, all `include_str!`'d, all validate standalone):
  `drone.svg` (map markers red-accent AND card icons green-accent, via CSS
  custom properties), `drone-favicon.svg` (same geometry, HUD-green fallback
  colors baked in — separate file so drone.svg's red defaults are untouched;
  browser tab via Trunk `copy-file` + `rel=icon`), `target-streak.svg`
  (currentColor streak roundel on leaderboard rows; the fire emoji is gone),
  `explosion.svg` (reserved for ENGAGE — see Part 4). Plane emojis are gone
  from the cards; the white 🛩️ favicon is gone.
- **HUD polish shipped:** `is_airborne()` (RTB counts as flying — "0/4
  airborne" at RTB fixed), telemetry averages rounded to 1 dp at the source
  (tooltip float dust), mission clock derives from the ticking 1 s `time`
  signal (was frozen at 00:00:00), single feed scrollbar.
- **Simulator:** `--api-url` also `DRONE_API_URL` env (clap `env` feature);
  `--insecure-tls` / `DRONE_INSECURE_TLS=1` (reqwest
  `danger_accept_invalid_certs`, loud warn, local-KinD only). Default
  `make run-simulator` unchanged.
- **READMEs:** app README filled (architecture: crate topology, three
  transports, enum-pin wire contract, ScyllaDB per-read-path schema design
  incl. leaderboard DELETE+INSERT-in-batch, Redis semantics, deterministic
  ids, resilience; prerequisites table; build/run incl. the FINAL-LEADERBOARD
  integrity check). Root README of THIS repo filled under the four verbatim
  headings (App-of-Apps vs ApplicationSets, blast radius +
  `--enable-policy-override`, ScyllaDB CRD-lifecycle vendor point, wave
  rationale, drop-in proofs, VKS/VCF appendix).

## 0.5 `deploy/` tree in the standalone repo (shipped 2026-08-17)

    deploy/
    ├── cluster/
    │   ├── kind-config.yaml        3 control-plane + 3 workers, no CNI, kube-proxy none,
    │   │                           extraPortMappings 80/443 -> 30080/30443 on cp-1 (label ingress-ready)
    │   ├── kind-bootstrap.sh       Gateway API CRDs FIRST -> Cilium (gatewayAPI.enabled) ->
    │   │                           cert-manager (enableGatewayAPI) + selfsigned ClusterIssuer ->
    │   │                           ESO + `fake` ClusterSecretStore -> KEDA, VPA (fairwinds),
    │   │                           metrics-server, scylla-operator. Idempotent.
    │   ├── kind-expose.sh          patches Cilium's per-Gateway svc to NodePort 30080/30443
    │   └── README-setup.md         for-dummies walkthrough, symptom-based troubleshooting
    ├── kubernetes/drone-convoy-attack-tracker/     Helm chart, screaming architecture
    │   ├── Chart.yaml              dependencies: [] BY DESIGN (see below)
    │   ├── values.yaml / values-nonprod.yaml / values-prod.yaml
    │   └── templates/  _helpers app app-config app-secrets(ESO+Certificate)
    │                   app-service(Gateway+HTTPRoute+301 redirect) app-storage(ScyllaCluster+
    │                   schema-init hook Job+Redis) app-scaling(HPA|KEDA+VPA Off) app-metrics
    ├── fly.io/fly.prod.toml        API on Fly (external ScyllaDB Cloud + Redis)
    ├── railway/railway.{prod,nonprod}.toml
    └── cloudflare/cloudflare-workers/wrangler.jsonc   dashboard as Workers Static Assets

Three decisions, each an answer to a question that WILL be asked again:

- **Chart.yaml has zero dependencies.** Cilium, cert-manager, ESO, KEDA, VPA,
  scylla-operator are cluster-scoped platforms; the app chart CONSUMES their
  CRDs. Vendoring them as subcharts is the anti-pattern (two apps fight over
  one cert-manager; `helm uninstall` rips out the CNI; Cilium cannot be a
  subchart of a workload it carries). Same split as this repo's waves. HPA is
  built into Kubernetes — no dependency exists.
- **No MetalLB.** Cilium implements Gateway API natively; but on macOS the
  KinD nodes live inside the container VM, so ANY announced LB IP is
  unreachable from the browser. `extraPortMappings` + NodePort is the honest
  local answer; `https://drone.localtest.me` resolves to 127.0.0.1 with no
  `/etc/hosts` edit. Real clusters get a real LB and none of this applies.
- **Let's Encrypt in prod, self-signed on KinD.** ACME HTTP-01 must reach the
  cluster; KinD is unreachable. `values-prod` → `letsencrypt-prod` via Gateway
  API; `values-nonprod` → `selfsigned-cluster-issuer` from the bootstrap.

Chart notes: env keys verified ⊆ `config.rs::from_env()`; schema-init hook
needs `make chart-sync` (Helm can't read outside the chart; copies
`schema/cql/` in, gitignored there — `kind-deploy` runs it); HPA and KEDA are
exclusive on the API (KEDA owns its HPA); VPA recommender-only, never Auto
beside an HPA. **No Helm binary existed in the authoring environment**: 26
mechanical checks pass (69 `.Values` refs, 11 helpers, 10 env keys, both
overlays, block balance, KinD topology) but `make chart-lint` and
`make chart-template` on the Mac are the first real render, and the first
`make kind-up` is the first real run of the bootstrap. Expect to patch
something small; the pinned operator versions are best-known-current.

Make front door additions: `kind-up`, `kind-load`, `kind-deploy`,
`kind-status`, `kind-down`; machinery `chart-sync`/`chart-lint`/
`chart-template`. Flow: `make kind-up` → `make kind-load` → `make kind-deploy`
→ open `https://drone.localtest.me` (accept self-signed once) →
`DRONE_API_URL=https://drone.localtest.me/graphql DRONE_INSECURE_TLS=1 make run-simulator`.

## 0.6 argocd-apps in THIS repo — delivery-complete

P4 done (Ingress→HTTPRoute wrapper templates for argocd + goldilocks parented
to a values-driven `platform-gateway`; cert-manager `ExperimentalGatewayAPISupport`
+ `master`→`control-plane` toleration; 10 chart pins refreshed — VERIFY with
`helm search repo` before first deploy, read Crossplane + Kargo release notes;
argo-cd `configs.cm`/`configs.rbac` migration + platform/app-team RBAC +
readonly default + IdP as a COMMENTED example; Crossplane Provider CRs
replacing `provider.packages`; sealed-secrets scope statement; ZERO `eql`
refs; cast-ai dropped — `git rm -r argocd-apps/cluster-addons/cast-ai` was a
manual step). P5 done. **cluster-secrets pattern completed**: a plain
Application (not an ApplicationSet — one instance, hub-only) at wave -3 syncs
`cluster-secrets/` with `directory.include: "sealed-*.yaml"` and
`prune: true`; `cluster-registration.template.yaml` carries
`argocd.argoproj.io/secret-type: cluster` + an `env:` label (inert now, stamped
so the list→cluster-generator refinement is a generator change, not a
re-seal); `spec-README.md` has the four-step per-spoke workflow. `helm
template argocd-apps/app-sets/` renders three objects (two ApplicationSets +
this Application). Delivery zip hygiene: build with `git archive` or from a
COPY with `.git` removed — never `rm -rf .git` on the working repo; slim
screenshots with `sips -Z 1800`.

---

# PART 4 — WHAT IS NEXT

## 4.1 NEXT FEATURE: tactical map region selector (spec'd, not started)

A **red dropdown selector on the top-right of the header navbar** that loads
a different tactical theater on the map. Requirements as stated:

- **Web-native styling** via the tactical select trait — it must NOT look like
  a macOS-native `<select>`. Same HUD language as the rest of the header
  (JetBrains Mono, accent red for this control since it's a mode switch, dark
  panel dropdown, keyboard-navigable).
- **Six theaters**, first is the default: **Afghanistan (Kandahar — current
  view)**, Syria, Libya, Pakistan, Iran (Tehran), Iraq (Baghdad).
- Choosing a region **dynamically re-centers the Leaflet map on that theater
  and drops that region's red waypoint pins** — the same visual grammar as
  today's Kandahar route, one route set per region.

Design guidance for whoever cuts it (this is where the pieces already are):

- `map.rs` today flies a hardcoded `ROUTE` const (Kandahar). The clean cut is
  a `regions` module: `struct Theater { id, label, center: (lat, lon), zoom,
  route: &'static [(f64, f64)] }` and a `THEATERS: [Theater; 6]` table; the
  selected theater is an `RwSignal<TheaterId>` in `AppState`
  (`selected_theater`, default Afghanistan). `map.rs` reads it in an Effect:
  on change, `map.setView(center, zoom)`, clear and re-add the waypoint
  markers + polyline for that route, and reset the flight loop's progress so
  the airframes restart at that route's IP.
- The **selector component** lives in `header.rs` (right side, beside ONLINE)
  and only writes the signal — it knows nothing about Leaflet. That keeps the
  "tactical select trait" reusable for the next dropdown.
- **Backend is unaffected**: theaters are a client-side view concern for now.
  The `waypoints` query is real server-side and this feature is the natural
  home for the deferred "waypoints → map ROUTE swap" — but do it as a second
  step, after the selector works against the six static route tables.
  Persisting the selection (query param or `window.storage`) is a polish item.
- Route data: six short waypoint arrays (10–26 points each, like Kandahar's
  26). Sensible centers/zooms: Kandahar 31.6/65.7 z8, Syria (Aleppo–Raqqa
  corridor) 36.0/38.0 z7, Libya (Sirte–Benghazi) 31.5/18.5 z7, Pakistan
  (Quetta–Peshawar) 31.5/69.5 z7, Iran (Tehran) 35.7/51.4 z8, Iraq (Baghdad)
  33.3/44.4 z8. Keep the map tiles as they are (CartoDB); no new tile
  provider.

## 4.2 App wishlist (unchanged, in priority order)

1. **ENGAGE button + explosion.svg impact animation** — `record_engagement`
   in `api.rs` still has zero callers; the asset is already shipped.
2. **Real waypoints → map** (pairs with 4.1 as its second step).
3. **Poll → subscription** over the live `/graphql/ws` — additive; polling
   stays as the fallback either way.
4. **Same-origin API_URL** — `api.rs` has `const API_URL =
   "http://localhost:8080/graphql"`, correct for `make serve`, wrong
   in-cluster; the Helm chart already routes `/graphql` same-origin through
   the Gateway, so this is a one-line change to a relative path plus a
   `Trunk.toml` proxy for local dev.
5. **Native-DB run path** — ScyllaDB + Redis are brew-installed on the M2
   (Redis running as a brew service); the Makefile's `deps-up` assumes it owns
   the containers. A `NATIVE_DBS=1` bypass is a small change; the config
   already reads `SCYLLA_HOSTS`/`REDIS_URL` from env.

## 4.3 First-run verifications still owed (nothing is broken; nothing is proven)

- `make chart-lint && make chart-template` — first real Helm render.
- `make kind-up` — first real run of the bootstrap; then `kind-load`,
  `kind-deploy`, browser, simulator.
- `helm dependency build` on the addon wrappers in THIS repo (validates the
  ten refreshed pins); `helm template argocd-apps/app-sets/` (three objects,
  literal `{{.path.basename}}`-style strings must survive the escaping).

---

# PART 1 — THE DASHBOARD DOES NOT BREATHE

**This is the top priority and the reason for this handoff.**

The Leptos dashboard renders beautifully and is almost entirely inert. The drones
fly (client-side animation), the leaderboard polls a real query, and every other
number on screen is frozen. The owner is selling this as a $100 Gumroad
Rust/Leptos course, so "looks live" is not good enough — a buyer will open
`lib.rs` on about page three.

## 1.1 What is actually live today

Exactly one path reaches storage end to end:

```
drone-simulator  --recordEngagement-->  mutation.rs
                 -->  ScyllaLeaderboardRepository::update_entry()  -->  ScyllaDB
                 -->  leaderboard query  -->  frontend poll (2s)  -->  leaderboard panel
```

Everything else is seeded in the frontend by `seed_static_panels()` in
`crates/drone-frontend/src/lib.rs`, or returned as hardcoded literals by stub
resolvers.

## 1.2 Why nothing else can be live

`crates/drone-graphql-api/src/context.rs` — `ApiContext` carries **only**
`leaderboard_repo`, plus the Scylla client, the cache, and five broadcast senders.
The resolvers therefore have no handle on any other repository, regardless of what
exists in `drone-persistence`.

**Twelve stubs in `resolvers/query.rs`** (lines ~76, 122, 154, 206, 210, 229, 267,
312, 370, 391, 412, 443) and **eight in `resolvers/mutation.rs`** (~49, 133, 178,
237, 283, 318, 355, 377). The ones that matter:

| Resolver | Current behaviour |
|---|---|
| `drones` | returns ONE hardcoded `REAPER-01` built inline, with a fresh `Uuid::new_v4()` each call |
| `engagements` | returns an empty `Connection` |
| `convoyStats` | `average_fuel_pct: 75.0` and `airborne_count: entries.len()` as literals |
| `latestTelemetry`, `telemetryHistory` | `// TODO`, empty |
| `waypoints` | `// TODO`, empty |
| `activeConvoys` | one hardcoded convoy, id `550e8400-e29b-41d4-a716-446655440000` |
| `convoy` | `// TODO` |

## 1.3 The repository layer is half-built — know which half

`crates/drone-persistence/src/repository/scylla_impl.rs`. **Write paths are real;
most read paths are not.** Do not assume "the repos exist" means "the repos work".

| Method | State |
|---|---|
| `ScyllaLeaderboardRepository::get_leaderboard` | **REAL** — full SELECT and row parsing |
| `ScyllaLeaderboardRepository::update_entry` | **REAL** — UPDATE plus counter maths |
| `ScyllaLeaderboardRepository::get_drone_entry` | partial — falls through to `Ok(None)` |
| `ScyllaEngagementRepository::record` | **REAL** — INSERT |
| `ScyllaEngagementRepository::get_by_drone` | **STUB** — `Ok(Vec::new())`, args underscored |
| `ScyllaTelemetryRepository::record` | **REAL** — INSERT |
| `ScyllaTelemetryRepository::get_latest` | **STUB** — `Ok(None)` |
| `ScyllaConvoyRepository::create` | **REAL** |
| `ScyllaConvoyRepository::get` | **STUB** — `Ok(None)` |
| `ScyllaWaypointRepository::get_waypoints` | **STUB** — `Ok(Vec::new())` |

Every stub carries the same comment: *"TODO: Implement full parsing of complex
X type"*. The CQL tables all exist in `schema/cql/001_core_schema.cql`: `convoys`,
`drones`, `waypoints`, `telemetry`, `engagements`, `engagements_by_drone`,
`accuracy_counters`, `leaderboard`, `command_log`, `alerts`.

So the work is **row deserialisation**, not schema design. That is the single
largest ticket in this repo.

## 1.4 The simulator only writes engagements

`crates/drone-simulator/src/main.rs` calls `convoy.generate_telemetry()` every
tick, logs the count, and **throws it away**. There is no `post_telemetry`, and no
drone-state call either.

So even after the read paths are implemented, `telemetry` and `drones` return
empty until the simulator posts them. Both mutations exist in the schema already:
`recordTelemetry` and `updateDroneState` (themselves stubs, mutation.rs ~283 and
~237).

## 1.5 Recommended order of work

Sequential. Each step is independently verifiable in the GraphQL playground at
`http://localhost:8080/graphql`, which is the fastest way to tell a backend
problem from a frontend one.

1. **Widen `ApiContext`.** Add `engagement_repo`, `telemetry_repo`, `convoy_repo`,
   `waypoint_repo` alongside `leaderboard_repo`, and construct them in
   `ApiContextBuilder`. Nothing else can proceed without this.
2. **Implement the read-path stubs in `scylla_impl.rs`** — row parsing for
   `Engagement`, `Telemetry`, `Convoy`, `Waypoint`. Copy the pattern from
   `get_leaderboard`, which already does this correctly for `LeaderboardEntry`.
3. **Implement `recordTelemetry` and `updateDroneState`** in `mutation.rs` against
   the now-available repos, and **have the simulator post telemetry and drone
   state** each tick, not just engagements.
4. **Point the query resolvers at the repos**: `drones`, `engagements`,
   `convoyStats`, `latestTelemetry`, `telemetryHistory`, `waypoints`, `convoy`,
   `activeConvoys`. Delete every inline hardcoded struct as you go.
5. **Frontend**: extend `start_live_feed()` in `lib.rs` to poll the new queries and
   drive `state.drones`, `state.engagements` and the stats panels. Then delete
   `seed_static_panels()` entirely — while it exists, a regression looks like
   working software.
6. **Swap polling for subscriptions.** See 1.6.
7. **Feed the map from real waypoints.** `map.rs` has a `ROUTE` constant of 14
   hardcoded points. Once `waypoints` returns rows, replace the constant with the
   query result; the animation code needs no other change.

## 1.6 The WebSocket route is commented out

`crates/drone-graphql-api/src/lib.rs:122`:

```rust
// TODO: WebSocket subscriptions disabled until async-graphql-axum supports axum 0.8
// .route("/graphql/ws", any(GraphQLSubscription::new(schema)))
```

Meanwhile `main.rs` still logs *"WebSocket subscriptions at ws://…/graphql/ws"*,
and the frontend's `services/websocket.rs` (`use_websocket`, never called)
connects there and gets a 404.

The subscription resolvers are already wired to the broadcast channels in
`ApiContext`, and were changed to return `async_graphql::Result<impl Stream<...>>`
so a missing context propagates with `?` instead of panicking the task.
**Re-check the axum / async-graphql-axum version constraint** before assuming this
is still blocked — the comment may predate a release that fixes it.

## 1.7 Frontend polish, low effort

- **An empty leaderboard collapses to a bare header.** When the poll returns
  `entries: []` the panel renders nothing at all. Add an explicit empty state
  ("NO ENGAGEMENTS RECORDED") so an empty result is distinguishable from a broken
  one — this has already caused confusion once.
- **`ws_connected` takes ~20s to flip on load**, because `start_live_feed()`
  awaits `fetch_active_convoys()` before the first leaderboard poll.
- **Hardcoded API URL.** `services/api.rs` has
  `const API_URL: &str = "http://localhost:8080/graphql"`, and `websocket.rs` the
  matching `ws://`. This **cannot work in-cluster**. Move to same-origin relative
  paths (`/graphql`, `/graphql/ws`); the Helm chart's HTTPRoute already routes UI
  and API through one hostname for exactly this reason, which also removes CORS.
- **ENGAGE button.** Requested, not built. `record_engagement` exists in
  `services/api.rs` with zero callers and the mutation works. Wire a control to it
  and play `assets/images/explosion.svg` as a temporary Leaflet marker for ~1.5s
  (self-animating SMIL: add it, wait, remove it).

## 1.8 Bugs already found and fixed — do not reintroduce

Each of these cost hours, and every one presented as something other than its
cause.

- **GraphQL errors arrive in a 200 OK body.** The simulator checked only
  `response.status().is_success()`, so every rejected mutation logged a cheerful
  HIT line and wrote nothing. Now parses `errors[]`. **Any new HTTP caller must do
  the same.**
- **Enum drift.** Simulator `TargetType` had `Artillery`/`Aircraft`; the schema has
  `AIR_DEFENSE`/`SUPPLY`. Silent coercion failures. async-graphql serialises
  variants as SCREAMING_SNAKE (`Agm114Hellfire` → `AGM114_HELLFIRE`).
- **Convoy id mismatch.** `activeConvoys` returns a fixed UUID; the simulator
  generated a random one. Both processes healthy, every panel empty. The simulator
  now pins to `ConvoySimulator::DEMO_CONVOY_ID`, overridable via
  `DRONE_CONVOY_ID`. **When `activeConvoys` becomes real, keep them agreeing.**
- **GraphQL variables not camelCased.** `fetch_leaderboard`'s `Variables` struct
  lacked `#[serde(rename_all = "camelCase")]` while the query declared `$convoyId`.
  Rejected on every poll, which read as a connectivity failure. Check this on every
  new query.
- **CSS class collision.** `main.css:245` defines `.drone-marker` (green card
  badge). Map markers use `.drone-air-marker`; keep them separate.
- **`var()` in SVG presentation attributes is unreliable.** Custom properties are
  substituted on CSS declarations. `assets/images/*.svg` use hex attributes with a
  `<style>` block re-theming via classes.
- **Container builds:** base was `rust:1.83` against `edition = "2024"` (needs
  ≥1.85), and the API Containerfile copied `drone-graphql-api` when the `[[bin]]`
  is `drone-api`. Both fixed; `ARG RUST_VERSION` is now parameterised.
- **`-C lto` via RUSTFLAGS is incompatible with `-C embed-bitcode=no`**, which
  Cargo passes for proc-macros — it fails every proc-macro dependency at once.
  Codegen settings live in `[profile.release]` in the root `Cargo.toml`, never in
  RUSTFLAGS.
- **ScyllaDB has no `/docker-entrypoint-initdb.d` hook** — that is a Postgres/MySQL
  convention. Schema is applied explicitly by `deps-up` and by the `scylla-init`
  compose service.

## 1.9 How to run

```
make build          # touch sweep + full build
make serve          # API + trunk dev server NATIVELY; podman only for Scylla/Redis
make run-simulator  # second terminal
```

Dashboard `http://localhost:3000`, playground `http://localhost:8080/graphql`.
`make stack-up` is the all-in-containers path and needs podman-compose.

`make build` runs `touch` first, because overlaying a zip restores the archive's
mtimes and Cargo then skips rebuilding files that look older than their artifacts.
`TOUCH=0` opts out.

---

# PART 2 — ARGOCD: P4 AND P5

Both corners of the ApplicationSet story are built and **unverified by Helm** —
there was no Helm binary in the authoring environment. Run `helm template` on both
charts before anything else.

```
helm template argocd-apps/app-sets/
helm template argocd-apps/cluster-apps/drone-convoy-tracker \
  -f argocd-apps/cluster-apps/drone-convoy-tracker/env/prod/values-prod.yaml
```

For `app-sets`, confirm the rendered `cluster-addons` ApplicationSet has **three**
generators and that `{{.path.basename}}`-style placeholders survive verbatim into
the output. That escaping is the one thing reading cannot prove.

## 2.0 Already done

- `app-sets/` rewritten: generator paths corrected to repo-root-relative,
  `goTemplate: true` + `goTemplateOptions: ["missingkey=error"]`, all placeholders
  dotted, `repo.url` driven from `values.yaml`, ArgoCD-side escaping confined to
  `templates/_helpers.tpl` behind backticks.
- **Sync waves without losing the glob.** Each wave group is its own matrix
  generator scoped to explicit directories and stamped via the git generator's
  `values.wave`. A catch-all globs `cluster-addons/*` with `exclude: true` on
  everything already claimed, so a new addon needs no edit and sorts into
  `addonDefaultWave`. Wave -2: cert-manager, external-secrets, sealed-secrets.
  Wave -1: kyverno, crossplane, keda, scylla-operator. Catch-all: argocd, cast-ai,
  external-dns, goldilocks, kargo, vault. Apps: wave 10.
- **Asymmetric deletion posture.** cluster-addons carries
  `applicationsSync: create-update`, `preserveResourcesOnDeletion: true` and the
  `resources-finalizer.argocd.argoproj.io` finalizer; cluster-apps has normal
  semantics so removing a chart removes the Application. **Note:**
  per-ApplicationSet `applicationsSync` is ignored unless the controller runs with
  `--enable-policy-override`, set in `cluster-addons/argocd/values.yaml`. Without
  it the setting looks configured and silently does nothing.
- AppProjects fixed: `sourceRepos` was `https://github.com/eql/*`, which would have
  rejected every generated Application; now driven from values. `cluster-apps` has
  `clusterResourceWhitelist: []`.
- Addon list: `cnpg` → `scylla-operator`; `cluster-api` and `kured` deleted;
  `external-dns` added (source `gateway-httproute`, provider deliberately unset).
- `cluster-apps/drone-convoy-tracker/` chart built with the full
  screaming-architecture set: `app.yaml`, `app-service.yaml` (Cilium Gateway +
  HTTPRoute), `app-config.yaml`, `app-storage.yaml` (ScyllaCluster CR),
  `app-secrets.yaml` (ExternalSecret + cert-manager Certificate),
  `app-scaling.yaml` (HPA + VPA recommender + KEDA), `app-metrics.yaml`.

## 2.1 P4 — addon values hygiene

**The repo currently argues with itself. This is what a careful reader catches.**

1. **Ingress contradicts the Gateway API decision.** `argocd`, `goldilocks` and
   `vault` prod values all carry `ingress.enabled: true`,
   `ingressClassName: nginx` and `nginx.ingress.kubernetes.io/*` annotations —
   pointing at a controller this repo never installs. Replace each with an
   HTTPRoute in that wrapper chart's own `templates/`, parented to a shared
   Gateway. **Highest-priority P4 item.**
2. **cert-manager feature gate.** Prod sets only
   `AdditionalCertificateOutputFormats=true`. Issuing certificates from Gateway
   `listeners[].tls` also needs `ExperimentalGatewayAPISupport=true`, or the
   Gateway migration has no certificate path.
3. **Dead toleration.** cert-manager tolerates `node-role.kubernetes.io/master`,
   removed in Kubernetes 1.25. Use `control-plane`.
4. **Stale chart pins, all of them.** argo-cd 5.46.7 (current ~10.x),
   cert-manager v1.13.2, crossplane 1.14.1, kyverno 3.0.5, keda 2.12.0,
   external-secrets 0.9.9, sealed-secrets 2.13.3, kargo 0.3.0, vault 0.25.0,
   goldilocks 6.7.0. Read release notes for **Crossplane and Kargo** specifically —
   both have had major transitions. Re-pin scylla-operator (1.21.0) and
   external-dns (1.21.1) against `helm search repo --versions` as well.
5. **argo-cd values migration**, which falls out of item 4:
   `server.config` → `configs.cm`, `server.rbacConfig` → `configs.rbac`. Then add
   an RBAC policy mapping a platform group to the `cluster-addons` project and an
   app-team group to `cluster-apps`, with `policy.default: role:readonly`. Keep the
   IdP connector a **commented example** — a reference must not hardcode anyone's
   identity provider. Do NOT hand-roll `argocd-rbac-cm` in wrapper templates; the
   subchart creates it and you will end up with two resources of one name.
6. **Crossplane provider install.** `provider.packages: [aws, kubernetes]` is the
   old path. Current Crossplane uses `Provider` CRs, and the monolithic AWS
   provider split into per-service families.
7. **Secrets story.** Decision made: Vault as backing store, External Secrets
   Operator as in-cluster sync, sealed-secrets kept **explicitly scoped to
   bootstrap-only** secrets that must exist before ESO runs. Implement it and state
   it in the README — three overlapping secret systems with no stated primary is
   the first thing a reviewer asks about.
8. **Brand sweep.** `eql.cloud` hostnames and `eql-org` still appear in addon env
   values; only the AppProject `sourceRepos` was fixed. Separately, `CHANGEME-ORG`
   appears in `app-sets/values.yaml` (`repo.url`) and
   `cluster-addons/argocd/values.yaml` (`projects.sourceRepos`) — **these two must
   agree**, or Applications generate and then refuse to sync with a project
   permission error, which is a confusing failure to debug.
9. **`cast-ai`** is still an addon: third-party SaaS agent needing an API key,
   cost-optimisation rather than platform. Candidate for the same treatment as
   cluster-api and kured.
10. **`cluster-secrets/` is spec-only** — it holds just a README. Cluster
    registration secrets (`argocd.argoproj.io/secret-type: cluster`) must exist for
    the list generator destinations to resolve. Document the bootstrap ordering:
    ESO and sealed-secrets are themselves addons, so the first cluster secret
    cannot be encrypted by a controller that is not yet running. Standard answer is
    one manually-applied bootstrap secret for the hub, everything after that
    via ESO.

**Post-delivery refinement, not a blocker:** once `cluster-secrets/` is populated,
swap the hardcoded list generator for the **cluster generator** with label
selectors (`env: prod`), so adding a cluster becomes a Secret rather than a repo
edit.

## 2.2 P5 — the README

For a guide this is arguably the highest-value artifact. Lead with the three things
App-of-Apps cannot do cleanly:

1. No per-child `Application.yaml` to write or maintain.
2. Environment promotion by **directory**, not by branch.
3. RBAC boundaries via AppProject, not everything in `default`.

Then cover:

- **Blast radius.** Deleting an ApplicationSet deletes every Application it
  generated and prunes their resources. Explain the asymmetric posture (protective
  for addons, normal for apps) and the `--enable-policy-override` dependency. This
  is the first objection an App-of-Apps holdout raises — answer it before it is
  asked.
- **The ScyllaDB point.** Upstream ScyllaDB documentation states Helm only creates
  CRDs on first install and never updates them, and recommends their GitOps
  manifest path over Helm for that reason. ArgoCD renders with `--include-crds` and
  re-applies every sync, so the ApplicationSet path closes a gap the vendor
  documents as a Helm limitation. A vendor describing the exact problem your
  delivery solves is the strongest paragraph available.
- **Worked drop-in examples.** `external-dns` was added with no edit to any shared
  file (catch-all glob). `drone-convoy-tracker` generates four Applications from
  one directory. Show both.
- **The wave example.** cert-manager (wave -2) must reconcile before
  scylla-operator (wave -1) because it issues the cert for the operator's webhook
  server. Concrete beats abstract.
- The existing `docs/*.png` diagrams already carry most of the comparison:
  `argocd-app-of-apps-v-appsets.png`, `appsets-structure.png`, and the three
  `argocd-*-cp-cluster-*.png` hub-and-spoke topologies.
- A clearly separated appendix for VKS/VCF-specific configuration, skippable
  without losing the architecture.

## 2.3 Verification — KinD on Apple Silicon

Planned, not yet done. Prerequisites, none of them one-liners:

- Create the cluster with the default CNI **disabled**, then install Cilium with
  `gatewayAPI.enabled=true`.
- Install the **Gateway API CRDs** before Cilium.
- `cloud-provider-kind` or MetalLB — KinD has no LoadBalancer, so the Gateway never
  gets an address without one.
- The chart's ScyllaCluster will not come up as written: `developerMode: false`
  assumes tuned nodes, and even the nonprod overlay's single 4Gi member is heavy
  for a KinD node. **Add a `scylla.developerMode` value** and drop nonprod
  resources — five minutes of work that should be decided, not discovered at 4pm.

A KinD run proves ApplicationSet mechanics, chart rendering and sync-wave ordering.
It will not prove the Vault/ESO path without standing up Vault; skip that.

---

# PART 3 — CONVENTIONS

- **Deliverable zips carry only the delta** — files changed since the previous zip,
  plus a `DELTA.md` naming what changed and why. Never re-ship unchanged files.
  Deletions must be called out explicitly in prose; an overlay cannot remove files.
- **Make targets stay a small front door** of bare verbs. The eight in `make help`
  are the interface; everything else is machinery under `make help-all`.
- **Rust standards:** no bare `unwrap()`/`expect()` in non-test code, `map_err` and
  `?` on `Result` returns, `thiserror` per domain crate with `anyhow` at the binary,
  lifetimes over `Arc<Mutex<T>>`, no reflexive `clone()`. The workspace already
  sets `clippy::pedantic` + `nursery` and `unsafe_code = "forbid"`, and there is no
  `Arc<Mutex<T>>` anywhere in the tree — keep it that way. Roughly 38 `unwrap()`s
  remain: mostly tests, WASM JS-interop in `map.rs`, and four
  `Normal::new(...).unwrap()` in the simulator marked `TODO(unwrap-sweep)` —
  infallible by construction but still fallible constructors.
- **Screaming architecture** for chart templates: a file's presence means that
  concern is present. Do not ship an empty `app-storage.yaml`.
- Nothing secret is templated into this repository — only references to where
  values live.
