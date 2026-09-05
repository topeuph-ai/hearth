# About Me, held by the person

A person-centred record that spans organisations which will never share a system,
with **no server and no operator**.

Scoped deliberately to the PRSB **About Me** standard — the things a person wants
professionals to know: how to communicate with them, what matters to them, how to
put them at ease. The standard explicitly says it is *not* a clinical record and
excludes medications and diagnoses. That boundary is load-bearing: it keeps
clinical safety certification (DCB0129), clinician liability and the heaviest data
protection questions out of scope.

**Do not add clinical fields without understanding what they drag in with them.**

## Why this shape

The blocker on every previous attempt at a shared record around one person is that
spanning organisational boundaries means *becoming the operator* — the legally
responsible party holding clinical information about a vulnerable person,
contributed by seven organisations, who must still exist in twenty years. Microsoft
HealthVault wasn't; it shut in 2019 and took the data with it.

With no operator, a professional isn't depositing into somebody's system. They are
sharing from their own, where their record stays authoritative and under their own
employer's governance. That's a different act legally, not just technically.

## What's here

- `dnas/aboutme/zomes/integrity/aboutme` — entry types and validation rules
- `dnas/aboutme/zomes/coordinator/aboutme` — the callable functions
- `workdir/happ.yaml` — `clone_limit: 1000`, because the architecture is one
  cloned cell (one isolated network) per person's circle

The whole professional workflow is **one tap**: an acknowledgement that they read
a specific version. Cheap for them; it's the thing families currently have no way
of knowing.

Validation rules enforced by every peer independently:
- an About Me must have a display name
- only the original author may revise an About Me
- you cannot acknowledge your own About Me
- an acknowledgement must reference a real About Me entry
- acknowledgements cannot be edited

## Versions (verified, not assumed)

| | |
|---|---|
| Holochain | **0.7.0** (released 30 July 2026) |
| hdk | **0.7.0** |
| hdi | **0.8.0** |
| holochain_serialized_bytes | **=0.0.57** |

Read from `crates/hdk/Cargo.toml` and `crates/hdi/Cargo.toml` at tag
`holochain-0.7.0`, not from documentation.

## Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

Requires the `wasm32-unknown-unknown` target. Both zomes compile clean.

Packaging into a `.dna` / `.happ` and running it needs the `hc` CLI, which is
**not installed on this machine**. Get it from the Holochain 0.7.0 release, or
via nix.

## Gotchas already paid for

Three things cost time. They're fixed here; don't rediscover them.

1. **`getrandom` refuses to build for wasm32.** hdk registers its own
   `__getrandom_v03_custom` backend that asks the host conductor for randomness,
   so the app must set `--cfg getrandom_backend="custom"`. Without it you get an
   error that looks completely unrelated to Holochain. See `.cargo/config.toml`;
   the flags are copied from how Holochain builds its own test zomes.
2. **`holochain_serialized_bytes` must be a direct dependency** even though
   nothing references it in the source. The `hdk_entry_helper` macro expands to
   code that names the crate at the caller's root.
3. **The hdi 0.8 flat-op API changed.** `FlatOp::StoreEntry` is gone. Creates are
   `FlatOp::CreateEntry(OpEntry::CreateEntry { .. })` and updates are a separate
   `FlatOp::Update(OpUpdate::Entry { .. })`. `get_links` now takes
   `(LinkQuery, GetStrategy)` rather than a built `GetLinksInput`, and `author`
   on a `TypedAction` is a method, not a field.

## Packaging

Use **`holochain/kangaroo-electron`** — first-party, pinned to Holochain 0.7.0,
updated within a day of the 0.7.0 release. Desktop only (Windows, macOS, Linux).

Do *not* depend on `p2p-shipyard`: pinned to 0.6, last commit to main 15 May 2026,
and the whole darksoil studio has been quiet since mid-July. Holochain has also
**paused Launcher development** and now recommends standalone apps instead.

Worth reading but not building on: **Moss / The Weave** (lightningrodlabs). Their
group-management DNA gives each group *and each tool within a group* its own
private peer-to-peer network — architecturally, this is the care circle. But it's
alpha, pinned to Holochain 0.6, and **has no licence file**, so all rights are
reserved by default.

The pattern: first-party tooling tracks core within a day; third-party runtimes run
about one minor version behind and move slowly. Take no downstream dependency you
don't control.

## Open questions

### Availability — resolved, and better than expected

An earlier draft treated this as the question that decides whether the project is
viable. That was wrong, and the correction is worth keeping.

Every agent in Holochain 0.7.0 joins with a **full storage arc**: in
`holochain_p2p`, an agent joining a space is constructed with `DhtArc::FULL`, and
the conductor config documents `target_arc_factor`'s default of 1 as normal
operation. **Every member of a circle holds a complete copy of that circle.** Six
members means six complete copies.

So there is no redundancy problem — no scenario where data is thinly spread or
partly lost. What remains is *liveness*: if nobody is online, nobody answers.
True of any peer-to-peer system, and a much narrower claim.

**Sharding is irrelevant here and is not needed.** It is planned for 0.9.x (an
epic at 0%, behind a compile-time flag since 0.4.x) and exists for large networks
where holding everything becomes burdensome. At the scale of a family circle arcs
stay full regardless. **This works on 0.7.0 as shipped, with no dependency on an
unreleased feature** — which is a far stronger position for a funding application
than "viable in a future release."

Residual risk is narrower still, because reading is local. A member already holds
a complete copy: they read from their own device, needing no network and no other
member online. Writing is local too. So an existing member is never blocked by
anyone else being offline.

The only case needing a live peer is *joining* — a newly invited member, or an
existing member on a replacement phone, has no copy yet and someone must hand them
one. The realistic worst case is a professional invited into the circle at 3am who
cannot receive anything because no existing member is reachable.

Mitigation is one always-on device per circle — a home laptop or a tablet on
charge — whose job is specifically to make joining possible at any hour. Note that such a device
holds a full readable copy like any other member, so contents should be encrypted
to circle members at application level regardless.

Design note for later: `target_arc_factor: 0` gives a node that participates
without storing. Wrong for a home device, right for a phone — full arc on a
machine at home, zero arc on the phone in a pocket.

### Mobile

Not a blocker any more, but not free either.

- **iOS**: Holochain 0.7.0 added wasmer's `wasmi` interpreted backend, which
  complies with Apple's ban on hot-loaded binaries — the App Store barrier is
  gone, and they demonstrated a Holochain app on an iPhone via Tauri. Remaining:
  wasmi is slower (irrelevant at this data size), a Lair keystore loading bug
  fixed in 0.7.1, and no ready-made packaging template.
- **Android**: proven since 0.3 (Volla ship a phone with a Holochain app), but the
  route is p2p-shipyard, which is stalled on 0.6.

The blocker moved from physics to packaging. The desktop demo needs neither.

## The demo

Two laptops and a third device. A person, their daughter, a nurse.

The daughter edits the About Me; it appears on the nurse's screen. **Turn off the
router** — it keeps working. Take a device out of the room, change something,
bring it back — it reconciles.

> There's no account, no server, and no company. If I'm hit by a bus tomorrow,
> this carries on working.
