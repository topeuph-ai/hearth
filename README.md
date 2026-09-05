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

## Build and run

No nix required. Binaries come straight from the Holochain 0.7.0 GitHub release
(the `holochain/binaries` repo publishes them per platform, MPL-2.0):

```bash
for a in hc holochain lair-keystore kitsune2-bootstrap-srv; do
  gh release download holochain-0.7.0 --repo holochain/holochain \
     --pattern "${a}-x86_64-pc-windows-msvc.exe" -O "bin/${a}.exe"
done
```

`bin/` is gitignored. Verified versions: hc 0.7.0, holochain 0.7.0,
lair-keystore 0.7.1, kitsune2-bootstrap-srv 0.5.0.

```bash
# 1. compile the zomes
cargo build --target wasm32-unknown-unknown --release

# 2. bundle
./bin/hc.exe dna pack dnas/aboutme/workdir
./bin/hc.exe app pack workdir

# 3. run two agents locally with NO network at all
./bin/hc.exe sandbox --piped generate workdir/aboutme.happ --run=8888 -n 2 network mem
```

`network mem` uses the in-memory transport: two agents, one machine, no
internet, no bootstrap server. This is the demo in its smallest form. Use
`network quic` with `--bootstrap` for real machines.

Note `hc sandbox generate network` also exposes `--target-arc-factor` directly,
which is how you would create a participating non-storing node.

## Gotchas already paid for

Five things cost time. They're fixed here; don't rediscover them.

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
4. **Manifests are `manifest_version: "0"`, not `"1"`.** Despite `"0"` looking
   like a placeholder.
5. **Manifests use `path:`, not `bundled:`.** Most tutorials online say
   `bundled`. That was an older format and `hc` 0.7 rejects it outright.

`.cargo/config.toml` **must be committed.** A stock `.gitignore` containing
`.cargo/` will silently exclude it and the project then fails to build for
anyone else, with an error that points at `getrandom` rather than at the missing
file.

**The wider lesson, which cost the most time today:** every one of these was
found by reading the crate source in the local cargo registry, or the Holochain
repo at tag `holochain-0.7.0`. None of them were in documentation or tutorials,
and several tutorials state the opposite. Holochain's published material lags its
releases. **Read the source at the tag.**

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

### Android — a first-party route exists

`holochain/android-service-runtime` pins `holochain = "0.7.0"` and
`holochain_serialized_bytes = "0.0.57"` — the same versions as this project. It
runs a **system-wide Holochain conductor as an Android Foreground Service**,
which in their words

> can run persistently, even when the app is closed, ensuring that you can be a
> reliable contributor to the peer-to-peer networks of your apps

That is the answer to "phones suspend background apps", and it makes a family
member's Android phone a reliable always-on peer — which is the mitigation for
the joining problem above, without needing a laptop left switched on. One
conductor is shared across hApps rather than each bundling its own.

Young: 4 stars, no licence file. But first-party and on our version.

### Other org repos worth knowing

- `holochain/binaries` — per-platform binaries, MPL-2.0. No nix needed.
- `holochain/hc-spin` v0.700.0 — run hApps in dev mode.
- `holochain/scaffolding` v0.700.0 — `hc scaffold` supports 0.7. It would have
  generated this structure. Hand-rolling cost time.
- `holochain/ai-tools` — a first-party Claude Code skill for Holochain, written
  because LLMs reproduce obsolete alpha APIs. **It targets HDK 0.6.1-rc.5 /
  HDI 0.7.1-rc.5, so it is a version behind this project** and would give wrong
  pins here.
- Tests use **`sweettest`**, not tryorama.
- `holochain/hc-http-gw` — HTTP gateway from web2 into Holochain. Irrelevant now;
  relevant if an NHS system ever needs to read a circle.
- `holochain/peerkit` — the Foundation's own experiment in a different direction:
  *"no Rust dependency, and a lighter footprint where deep validation is layered
  on top rather than built in from the start"*, described as *"one of what will be
  several experiments the foundation supports over time."* Not a reason to move —
  validation built into the substrate is exactly what the no-operator argument
  rests on — but worth knowing the Foundation is hedging its own architecture.
- The release also ships `holochain-unstable-*` binaries: the build with
  countersigning, warrants and sharding enabled behind the compile-time flag.
  Not needed here.

## Build order

Four Holochain capabilities this design wants and does not yet use:

1. **Membrane proof** — how a circle controls who joins. The DNA currently has no
   membrane, so "the person decides who is in" is a claim in the DPIA that the
   code does not yet back. **Highest priority, because it is the gap between the
   document and the software.**
2. **Capability grants** — a professional's access as a revocable, assignable
   grant rather than plain membership. Gives real revocation, which speaks
   directly to the open erasure question.
3. **Cloned cells** — `clone_limit: 1000` is set but nothing creates a clone. One
   circle per person is the architecture and it is not implemented.
4. **Remote signals** — "someone read your record", without polling and without a
   server.

Not needed: countersigning (nothing here requires atomic multi-party agreement)
and warrants (automatic).

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

## Design principle: offline is not a failure state

Everyone being offline mostly means everyone is busy living, not that anything is
broken — and if nobody is online, nobody is trying to read it either. The concern
largely cancels itself out.

This has a consequence that is easy to lose by accident, because every UI
convention we have inherited comes from cloud software, where offline genuinely
does mean broken. Those defaults are all anxiety: reconnecting banners, sync
spinners, staleness warnings, notifications nagging you back.

**None of that applies, and none of it should be built.** A member's copy is
complete. When they open the app it is simply there. There is nothing to
reconnect to.

For these users this is not a nicety. A carer who is already exhausted does not
need software implying she has fallen behind on something.

**Rule: no interface element may suggest that being away is a problem.** No sync
status, no "you are offline" bar, no last-updated warnings. Information that
arrived while someone was away is shown as new, never as a backlog they are late
on.

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
