# About Me, held by the person

[![tests](https://github.com/topeuph-ai/hearth/actions/workflows/tests.yml/badge.svg)](https://github.com/topeuph-ai/hearth/actions/workflows/tests.yml)

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
- **only the person may write or revise their own About Me** — membership lets
  you read a circle and acknowledge it, never author somebody else's account of
  themselves
- an About Me must have a display name
- only the person may extend their own update chain, and only between their own
  records
- only the person may publish an About Me to the circle index
- you may only attach your own acknowledgement, and never to your own record
- an acknowledgement must reference a real About Me entry
- acknowledgements cannot be edited
- only the agent who created a link may remove it
- only the author of a record may delete it

### What an acknowledgement does and does not prove

It proves **this key asserted it had read this exact version**. `role` is free
text and is not a credential — nobody checks that "district nurse" is true.

**This distinction must survive into the interface.** Show *"Read by \<identity\>,
role claimed: District Nurse"*, never *"✓ Read by District Nurse"*. The second
implies a verification that does not exist, and a family could reasonably rely
on it.

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

## Running the demo

Two terminals. The first serves the interface, the second starts two agents:

```bash
cd ui && npm run dev
```

```bash
cd ui && npm run demo
```

**Two windows on one machine is the whole argument**: two separate people, two
separate stores, talking to each other with nothing in between. Make a circle
in one, invite the other, watch the acknowledgement arrive.

`hc-spin` shells out to `kitsune2-bootstrap-srv`, `holochain`, `lair-keystore`
and `hc` **by bare name, and does not bundle them**. If they are not on PATH
the only symptom is an empty error:

```
[hc-spin] | [hc run-local-services] ERROR:
```

Empty because the *spawn* failed rather than the process, so there is nothing
to report. `npm run demo` goes through `ui/scripts/demo.mjs`, which puts `bin/`
on PATH and checks the binaries and the hApp are present before starting.

## Gotchas already paid for

Six things cost time. They are fixed here; do not rediscover them.

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
6. **DNA properties must be YAML-representable.** A properties struct holding
   an `AgentPubKey` serialises to a byte array, and anything that converts a
   `DnaFile` back into a bundle then fails with *"DnaDef properties were not
   YAML-deserializable: invalid type: byte array"*. Hold hashes as base64
   strings instead — `holo_hash` with the `encoding` feature converts both
   ways, and `Display` and `TryFrom<&str>` round-trip. This only surfaces when
   something round-trips a DNA, such as a test harness, so it can hide for a
   long time.

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

1. ~~**Membrane proof**~~ — **done.**
2. ~~**Cloned cells**~~ — **done.** Circles are clones; see below.
3. ~~**Capability grants for revocation**~~ — **wrong item.** See *Revocation*.
   They turned out to belong to signals instead: `init` grants access to
   `recv_remote_signal` so members can deliver into this cell.
4. ~~**Remote signals**~~ — **done.** When a professional acknowledges, their
   device tells the holder's device directly. No polling, no server, no
   notification service in the middle — which is how a family finds out that
   somebody actually read it.

   Sent fire-and-forget and deliberately unable to fail the write. The
   acknowledgement on the chain is the evidence; the signal is only the nudge.
   A family should never lose the record that somebody read the notes because
   a phone happened to be off.

Not needed: countersigning (nothing here requires atomic multi-party agreement)
and warrants (automatic).

**The build order is empty.** What the back end does not have is a face: there
is no interface, and the demo needs one. That is the critical path now.

## Revocation

There are three different things people mean by this, and conflating them
produces software that lies:

| | |
|---|---|
| **Membership revocation** | Stop someone writing to the circle in future |
| **Access revocation** | Stop someone reading what they already hold |
| **Erasure** | Remove data from their device |

**Validation cannot enforce membership revocation.** `must_get_agent_activity`
needs a known `chain_top`, so to ask "has the founder removed this person?" a
validator would need the founder's *current* chain head — mutable, unknown at
validation time, and different for each validator. Deterministic validation
cannot see it. Letting the writer cite the head themselves does not help: a
removed member simply cites an older one.

**Capability grants do not fix this.** They gate remote calls into your own
cell. They say nothing about what somebody already holds, or about what the DHT
will serve them. Listing them as the revocation mechanism was a mistake in an
earlier version of this file.

**The sound answer falls out of the architecture: revocation is re-forming the
circle.** A circle is a clone, and clones are nearly free. To remove someone,
make a new circle with a new network seed and invite everyone except them. They
are excluded by mathematics rather than by a rule someone has to enforce, and
the cost is one clone.

What that does *not* do — and nothing can — is retrieve what they already have.
Once a person has legitimately received plaintext, it is theirs. **Access
revocation and erasure are not achievable against someone who has already read
the data**, on this architecture or any other, and the DPIA says so rather than
implying otherwise.

Best-effort removal within an existing circle (the founder records a departure,
other members' apps stop showing that person and stop sharing new material with
them) is worth building for the ordinary case of a professional leaving. It is
a courtesy, not a control, and must never be described as one.

## The membrane: who gets into a circle

The founder's public key is written into the DNA **properties**, which are part
of the DNA hash. So a different founder produces a different DNA hash, which
produces a genuinely separate network. **Circles cannot see each other, and that
is a fact about the maths rather than about anyone's access control list.** This
is why one clone per person works.

An **invitation** is the founder's signature over the invitee's public key. The
invitee presents it as their membrane proof when joining. Every peer verifies it
independently against the founder key baked into the DNA. Nobody is asked for
permission, because there is nobody to ask.

Signing over the invitee's *own* key means an invitation cannot be passed on to
somebody else.

Checked in two places:
- `genesis_self_check` runs locally before joining, so a bad invitation fails
  immediately with a readable reason instead of being silently rejected later.
- `FlatOp::CreateRecord(OpRecord::AgentValidationPkg { .. })` in `validate` is
  the real enforcement, done by the network.

`invite()` in the coordinator issues one. There is deliberately no permission
check on it: anyone may call it, and a non-founder's signature simply will not
verify. **There is nowhere to enforce a permission, because there is no server —
enforcement lives in every peer's copy of the rules.** That is the whole
architecture in one function.

### Two boundaries, not one

An external red-team review found the central mistake in the first version:
**"closed circle" had been implemented far more strongly than "person-owned
record".** Those are different boundaries.

Who may **enter** was enforced by the membrane. Who may **write** was not
enforced at all — any admitted member could author an About Me and publish it
to the circle index, so a circle could hold several competing accounts of the
same person, each legitimately maintained by whoever invented it.

Worse, and missed by that review: the validation function ended in a catch-all
that returned Valid for everything it did not name, which included **all link
creation and all deletes**. That allowed a sharper attack than a competing
record. A member could create an `AboutMeUpdates` link from the person's own
original record to an entry of their own; any reader following the update chain
would then be shown the impostor's content **as the person's current record** —
without ever touching the person's entry, and so without tripping the
update-author rule. The same hole let any member delete the person's record
outright.

Both are now closed. Links carry meaning in this design, so they have rules of
their own, and the catch-all is documented as covering only Holochain's internal
bookkeeping.

**The lesson worth keeping: a validation function that ends in a permissive
catch-all is a security hole with a comment on it.** Enumerate what you allow.

### What the tests actually proved

`tests/tests/adversarial.rs` — nine tests, run in CI on every push. **Verified,
not asserted:**

- an uninvited agent cannot join
- an invitation signed over one person's key does not admit anybody else
- a member can call `invite()`, and their signature admits nobody
- the person can write their own About Me
- **a member cannot write the person's About Me** — the red-team finding
- a member cannot extend the person's update chain
- a member cannot delete the person's record
- a member can acknowledge a record
- nobody can acknowledge their own record

**And the run that mattered was the one before green.** Four passed and five
failed, and among the failures was *the person can write their own About Me*.
Alice could not write either — so *a member cannot write* had been passing
because **nobody** could write. A green test proving nothing.

The cause was the link rule added to close the red-team hole: it demanded that
every `CircleToAboutMe` link point at an About Me, and `Path::ensure` builds the
anchor tree with links of that same type pointing at Path entries. The security
fix had broken every write in the application.

Reading the code would not have found that. Running it did.

### Also proven

- a circle is a cloned cell, and circles with different holders are different
  networks even with an identical seed
- nobody can create a circle in another person's name
- anyone may enter the lobby, nobody may write in it, and a real circle cloned
  from it does accept the holder's own record
- **the holder is told, by the reader's own device, when their record is read**

- a circle with no founder configured admits nobody
- a circle with a malformed founder admits nobody
- an About Me must have a display name
- **two peers with identically-timestamped updates converge on the same
  version** — `order_versions` is a pure function in the integrity crate,
  unit tested directly, because a timestamp collision cannot be provoked
  through a conductor on demand. The coordinator calls that same function, so
  the tested code is the running code.

### Still untested

Twelve green integration tests and three unit tests are not a proven system.
These rules have no coverage:

- acknowledgements cannot be edited
- only the agent who created a link may remove it

### The lobby, and configuration that fails closed

Reading the DNA properties yields one of three states, never an `Option`:

- **`Founder`** — a real circle, closed around one person.
- **`Lobby`** — anyone may join, **nobody may write**. The app's provisioned
  cell is a lobby, and its only job is to exist so the app is installable and
  can clone real circles out of itself. Everyone who installs the app shares
  it, so it must hold nothing: entry is unrestricted precisely because there
  is nothing there to reach. It must be asked for in writing (`lobby: true`).
- **`Misconfigured`** — no properties, unreadable properties, or a founder that
  is not a valid agent key. **Admits nobody and lets nobody write.**

An earlier version treated a missing founder as an open circle, so a typo, a
missing config or a botched clone would have produced a wide-open circle around
a vulnerable person, silently. Those same mistakes now produce a circle nobody
can enter: visibly broken rather than invisibly exposed.

**Absence of configuration must never mean absence of a membrane.** Both failure
cases are covered by tests.

This must be made to fail closed before the software goes in front of anybody.
It is marked in `founder()` in the integrity zome.

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

## Licence

Apache License 2.0. See [LICENSE](LICENSE).
