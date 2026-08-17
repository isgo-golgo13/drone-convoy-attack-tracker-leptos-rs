# Selling the Drone Convoy Tracker on Gumroad — packaging & launch, step by step

Product: **Rust full-stack tactical drone tracking system** (Leptos/WASM + GraphQL
+ ScyllaDB + Redis + Cilium Gateway API) delivered as a **source zip + ~85-page
tutorial PDF**, with versioned updates. Price **$150** single seat, **$375**
team (5 seats). This document is the runbook from "private repo" to "live
product with the first sale."

Nothing here is legal advice — the license text is a normal paid-code license
of the kind used by Tailwind UI / Cruip / ThemeForest authors; have a lawyer
glance at it once if you want.

---

## Phase 0 — Decisions (already made; recorded so nothing drifts)

| Decision | Value | Why |
|---|---|---|
| Delivery | Zip (source) + PDF, no repo access | Repo access = free product |
| Price | $150 single / $375 team (5 seats) | Compared against $300–500 courses that ship a TODO app; this ships a system |
| Tutorial length | ~85 pages, versioned (v1.0, v1.1 …) | Depth per page beats page count; updates are the anti-piracy moat |
| Watermark | Gumroad PDF stamping (buyer email on every page) | Zero-effort deterrent |
| License keys | ON | Revocable per-buyer if a copy surfaces |
| Repo | **Private** | Non-negotiable — see Phase 1 |
| Updates | Through Gumroad's "post/update to buyers" only | Pirated copies go stale; buyers' don't |

---

## Phase 1 — Repository hygiene (do this first, before anything else)

### 1.1 Make the repo private
GitHub → repo → **Settings → General → Danger Zone → Change visibility →
Make private.** If it was ever public: assume every commit up to that moment is
in the wild. That's fine — the product is the *maintained* version, and future
commits are private.

### 1.2 Replace the license (yes, you can change it any time)
A GitHub license is just the `LICENSE` file — nothing is registered anywhere.
Delete the MIT `LICENSE`, add the commercial one below, commit. GitHub's
"license" badge re-reads the file. Anyone who cloned under MIT keeps MIT for
*that copy*; everything after is under the new terms.

`LICENSE` (root of repo AND root of the zip):

```
DRONE CONVOY TRACKER — COMMERCIAL SOURCE LICENSE v1.0
Copyright (c) 2026 Tomas Wallick / LuckyDrone.io. All rights reserved.

This software and accompanying tutorial ("the Product") are licensed, not
sold, to the purchaser ("you") on the following terms:

1. GRANT. You receive a perpetual, worldwide, non-exclusive, non-transferable
   license to use, copy, modify and compile the source code, and to deploy
   applications built from it, for any purpose including commercial products
   and client work, without royalty. Single-seat: one named developer. Team:
   up to five named developers within one organisation.

2. RESTRICTIONS. You may not (a) redistribute, sublicense, sell, publish or
   make available the source code or the tutorial, in whole or in substantial
   part, in source or readable form — including in public repositories,
   package registries, or other tutorials/courses/templates; (b) use the
   Product to create a product whose primary value is the Product itself
   (a competing template, starter kit, or course); (c) remove this notice.
   Deploying a compiled application built from the Product is never a
   redistribution.

3. UPDATES. Updates are delivered through the purchase platform to license
   holders. There is no obligation to provide updates or support.

4. WARRANTY. Provided "as is" without warranty of any kind. In no event shall
   the author be liable for any claim or damages arising from the Product.

5. TERMINATION. Breach of section 2 terminates the license. Sections 4–5
   survive.
```

Also change `Cargo.toml` `[workspace.package] license = "MIT"` →
`license-file = "LICENSE"` (Cargo requires one or the other; a custom license
uses `license-file`).

### 1.3 Add the product files to the repo root
```
LICENSE            (above)
VERSION            e.g. 1.0.0  — matches the PDF version on its cover
CHANGELOG.md       Keep-a-Changelog format; v1.0.0 = "Initial release"
```
And confirm `.gitignore` covers `target/`, `dist/`, `**/*.wasm`, `.DS_Store`,
`*.zip`, `*.pdf` (the tutorial is a *product file*, not repo content — keep it
out of git or in a private docs repo).

### 1.4 Tag the release
```shell
git tag -a v1.0.0 -m "Drone Convoy Tracker v1.0.0 — first Gumroad release"
git push origin v1.0.0
```
Every zip you ever ship is built from a tag, never from a working tree.

---

## Phase 2 — Build the deliverable zip (reproducibly, from the tag)

**Never zip a filesystem copy.** `git archive` ships tracked files at the tag
only — no `.git`, no `target/`, no `.DS_Store`, byte-identical every time.

```shell
cd drone-convoy-attack-tracker-leptos-rs
git archive --format=zip \
  --prefix=drone-convoy-attack-tracker-leptos-rs/ \
  -o ../drone-convoy-attack-tracker-v1.0.0.zip v1.0.0

# sanity — nothing fat, nothing forbidden
unzip -l ../drone-convoy-attack-tracker-v1.0.0.zip | sort -k1 -n | tail -8
unzip -l ../drone-convoy-attack-tracker-v1.0.0.zip | grep -E '\.git/|target/|dist/|\.DS_Store' && echo "LEAK" || echo "clean"
```
Expected size ~2–3 MB (source ~750 KB + slimmed screenshots). Then **the
buyer's-eye test**: unzip into a fresh temp dir and run the README's
`make build && make serve && make run-simulator` exactly as written. If it
doesn't fly from the zip alone, it doesn't ship.

Zip layout the buyer sees:
```
drone-convoy-attack-tracker-leptos-rs/
├── LICENSE  VERSION  CHANGELOG.md  README.md
├── Cargo.toml  Cargo.lock  Makefile
├── assets/  config/  containers/  crates/  deploy/  docs/  schema/
└── (no .git, no build output)
```

---

## Phase 3 — The tutorial PDF (~85 pages, versioned)

Deliverable: `Drone-Convoy-Tracker-Tutorial-v1.0.0.pdf`, US Letter (8.5×11),
version + date on the cover and in the footer. Build the PDF outside the
repo (Typst/LaTeX/Google Docs → PDF); do NOT commit the PDF.

Chapter plan (reveal first, derivation after — the reader must see it fly in
the first 20 pages):

1. **It flies in 15 minutes** — prerequisites (Rust 1.85+, wasm target, Trunk,
   Podman, Make), `make serve`, `make run-simulator`, what you're looking at.
   Anatomy diagram of the five crates. *(~8 pp)*
2. **The thesis: one type system across the wire** — `drone-domain` consumed by
   simulator, API, persistence AND the WASM frontend; the compiler enforces the
   contract. The Leptos/React/Next comparison table — concrete, same feature
   three ways (leaderboard row, drone card, poll loop), honest about where TS
   wins (ecosystem, hiring, hot-reload, learning curve). *(~10 pp)*
3. **ScyllaDB design by read path** — partition-per-query, telemetry hour
   buckets, engagements dual-write batch, the leaderboard clustering-key
   DELETE+INSERT story, UDTs ↔ Rust structs. *(~12 pp)*
4. **Redis as accelerator, never truth** — leaderboard cache, invalidate-on-
   write, degrade to Scylla. *(~4 pp)*
5. **GraphQL API in axum + async-graphql** — resolvers, dataloaders, the enum
   wire contract (the Inflector digit bug — war story #1), errors[] in a 200
   body, subscriptions on /graphql/ws. *(~10 pp)*
6. **Rust the way this codebase does it** — the quick sections you asked for:
   lifetimes over `Arc<Mutex<T>>`, `thiserror` per domain crate + `anyhow` at
   the binary, `?` and `map_err` discipline, no bare unwrap, `clippy::pedantic`
   as the floor. Short, concrete, drawn from real functions. *(~6 pp)*
7. **Leptos frontend** — signals and `<For>` (war story #2: the frozen-row
   composite key), the poll loop, wasm_bindgen + Leaflet, charts (render-once/
   update — war story #3), the TacticalSelect component, panes and layering for
   the impact bursts, the server-anchored flight loop + GPS readout. *(~14 pp)*
8. **The simulator** — deterministic UUIDv5 ids, wait_for_api + identity-per-
   tick self-healing (war story #4), theater routes, engagement gating. *(~5 pp)*
9. **Deploy: KinD + Cilium Gateway API + cert-manager + ESO** — the chart,
   why it has zero dependencies, why no MetalLB, why self-signed on KinD and
   Let's Encrypt in prod. *(~10 pp)*
10. **Appendices** — Fly.io / Cloudflare Workers / Railway configs (short);
    field notes (the root-cause write-ups); how to apply an update; license.
    *(~6 pp)*

Sidebars throughout: the Rust snippet + the same thing in TypeScript
React/Next/Node. Keep them short (10–25 lines each) — the point is the
contrast, not completeness.

---

## Phase 4 — Gumroad setup (one sitting, ~1 hour)

1. **Product** → New product → type **Digital product**. Name:
   *"Drone Convoy Tracker — Full-Stack Rust (Leptos/WASM · GraphQL · ScyllaDB · Redis · Kubernetes) — Source + 85-page Tutorial."*
2. **Files**: upload the zip AND the PDF as two files on the same product.
3. **Versions/tiers** (Gumroad "Versions" or "Variants"):
   - *Single seat* — $150
   - *Team (5 seats)* — $375
   Same files on both; only the license scope differs (say so in each variant's
   description).
4. **License keys**: Product settings → **Generate a unique license key per
   sale** → ON. You'll see keys in the sales list; a key can be disabled if a
   copy leaks.
5. **PDF stamping (watermark)**: on the PDF file → **Stamp PDF** → ON. Gumroad
   overlays the buyer's email on every page at delivery. This is the whole
   watermarking step.
6. **Content/description** (this is your sales page — write it as the buyer):
   - 30-second screen recording of the map with bursts as the **cover
     media** (record with QuickTime; this is your entire marketing).
   - What ships (repo layout, the PDF chapter list, the deploy targets).
   - **Who it's for**: mid/senior devs with React/Next/Node history evaluating
     Rust for the full stack. Say who it's *not* for (Rust beginners).
   - The proof points: every panel is DB-backed, ScyllaDB clustering-key
     ordering, real Gateway API/cert-manager/ESO chart, one type system across
     the wire, root-cause field notes.
   - Prerequisites (Mac/Linux, Rust 1.85+, Podman/Docker, 8 GB for KinD).
   - Refund policy: "14 days if it doesn't build following the README" —
     specific, generous, and refunds are rare because chapter 1 makes it fly.
   - Updates policy: "free updates for v1.x delivered through Gumroad."
7. **Settings**: enable **"Allow customers to pay what they want"** → OFF (fixed
   price); **discount code** `LAUNCH` = $25 off for the first two weeks (list
   at $150, launch at $125 — anchors the value); **affiliate program** ON at
   20–30% (Rust newsletter authors will promote for that).
8. **Preview**: use Gumroad's preview link and buy it yourself with a 100%
   discount code to test the full flow: key issued, PDF stamped, zip downloads,
   zip builds. Delete the test sale after.

---

## Phase 5 — Launch week checklist

- [ ] Repo private, LICENSE swapped, `license-file` in Cargo.toml, tag v1.0.0
- [ ] Zip built from the tag; buyer's-eye test passed from a clean unzip
- [ ] PDF v1.0.0 exported, cover shows version + date, checked on a phone
- [ ] Gumroad product live with both files, two tiers, keys ON, stamping ON
- [ ] 30-second recording uploaded as cover media
- [ ] Test purchase completed and deleted
- [ ] `LAUNCH` code created; expiry set (14 days)
- [ ] Announce: r/rust ("Show and tell"), Leptos Discord #showcase, X/LinkedIn
      with the recording, This Week in Rust "call for participation" (they list
      commercial products sparingly — lead with the technical write-up, link
      the product second), your LinkedIn banner already covers four accounts
- [ ] Reply to every buyer question the first week; each answer becomes a
      FAQ line on the sales page

---

## Phase 6 — Updates (the moat)

For every update:
1. Bump `VERSION` and `CHANGELOG.md`, tag `v1.x.y`, `git archive` from the tag.
2. Re-export the PDF with matching version.
3. Gumroad → product → **replace files** (keeps all past buyers entitled) →
   **Send update email to customers** with the changelog. Buyers re-download;
   pirated copies stay at v1.0.

Planned v1.x line: KinD first-run verification notes; ENGAGE button; real
waypoints from the API; poll → subscription. **Dioxus edition** is a *separate*
product later (desktop-first pitch), sold at its own price with a bundle
discount for existing buyers.

---

## Reference: what NOT to do
- Don't link a repo, public or private, from the product. Ever.
- Don't ship from a working tree; always `git archive` a tag.
- Don't put the PDF in the repo (it's a product file with its own lifecycle).
- Don't promise support; promise updates. Answer questions anyway.
- Don't watermark by hand — Gumroad stamping is per-buyer and automatic.
- Don't undercut the price to "test the market"; the recording and chapter 1
  are the test. Discount codes, not list price, are the lever.
