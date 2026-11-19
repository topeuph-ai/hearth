# What others on Holochain are building, and what Hearth could learn

**Read 19 November 2026**, from the repositories themselves: Holochain,
Wind Tunnel, Flowsta, Your Own AI, Moss and Lightningrod Labs, and Volla.
Everything here is an **idea or a lead, not a plan**. Nothing has been built
into Hearth from it. Where code could one day be borrowed, its licence is
given. Where there is no licence, it is ideas only.

The short version, in order of how much each could matter:

1. **A phone route with an open licence now exists.** Holochain's own
   in-process Tauri plugin declares MIT or Apache-2.0. Hearth has been waiting
   for exactly this.
2. **A bug that very likely slows Hearth down is fixed**, and on its way to
   Holochain 0.7.2.
3. **Somebody has solved adding a password to the keystore after install**,
   which is the biggest gap in Hearth's threat model.
4. **Somebody has a working pattern for recovering a person's identity on a
   new computer**, with the traps they fell into written down.
5. **Somebody has a library for moving between Holochain versions**, the
   problem Hearth meets again at 0.8.
6. Smaller things: signed updates, Holochain performance patches, lessons from
   Moss on slow machines, Wind Tunnel for measuring Hearth, and Lightningrod
   Labs' work on networks with no internet at all.

---

## 1. Holochain's own Tauri plugin, open-source licensed

**Where:** [holochain/android-service-runtime](https://github.com/holochain/android-service-runtime),
crate `crates/tauri-plugin-holochain`, version 0.3.0 (27 August 2026),
described as "Tauri plugin that runs an in-process Holochain conductor
(FFI-free, desktop + mobile)". Written by Eric Harris-Braun.

**Licence:** the workspace `Cargo.toml` declares `MIT OR Apache-2.0`. There is
no separate LICENSE file in the repository, so it is worth asking Holochain to
add one before relying on it.

**Why it matters.** [to-a-product.md](to-a-product.md#the-tauri-plugin-the-likeliest-app-route-and-a-licence-to-wait-for)
says the likeliest route to Hearth on Android was darksoil's plugin, which is
source-available with no licence, so Hearth should wait. This is a different
plugin. The repository's own plan
(`docs/holochain-0.6-and-unified-plugin-plan.md`) says it is "greenfield",
with "zero dependency (code or environment) on darksoil". So the licence
question that blocked the phone route may be answered.

**Who uses it:** Volla Messages, a private chat app on Holochain for Android
and desktop ([HelloVolla/volla-messages](https://github.com/HelloVolla/volla-messages)),
pinned to a revision of it.

**Not yet:** it pins Holochain 0.7.0, and Hearth is on 0.7.1. Nobody here has
built with it. One stack for Windows *and* Android would be a large change
from Kangaroo, and would want its own careful look.

## 2. Several networks between the same two devices: a bug, fixed

**Where:** [holochain#5962](https://github.com/holochain/holochain/issues/5962),
fixed by [#6057](https://github.com/holochain/holochain/pull/6057), and
backported to the 0.7 branch on 18 November
([#6070](https://github.com/holochain/holochain/pull/6070)). So it should be
in **Holochain 0.7.2**, which is not released yet.

**What it is.** When a device first connects to another, it sends a list of
its agents. Holochain kept one list for the whole program, and each network
overwrote it with its own. So, in the issue's words, "with N spaces against
the same remote, at most one space gets its agents across on a given
connection; the rest stay blocked until bootstrap polling happens to fill the
gap".

**Why it matters.** Hearth always has several networks between the same two
devices: the door and every circle, plus circles from earlier versions. This
is a likely cause of slow joins ([latency.md](latency.md)) and of the first
pass read failing because the door had not found the other device yet. Not
proven to be the cause: worth measuring before and after.

**What to do:** move to Holochain 0.7.2 when it comes out. Unlike 0.7.1, the
backported changes do not look as if they change capability grants, but check
the changelog first.

**Also in the same backport:** clone cells can now be given init properties
(#6054), and an uninstalled app's data is cleaned up properly (#6034). And on
`develop`: revoking a grant no longer breaks an identical later grant (#6063),
and a stopped conductor closes its network (#6061).

## 3. Adding a password to the keystore later, without locking anyone out

**Where:** [WeAreFlowsta/flowsta-hc-lines](https://github.com/WeAreFlowsta/flowsta-hc-lines),
`src/keystore/rewrap.rs`. **MIT licence.**

**What it is.** Lair, the keystore, has no way to change its passphrase. But
the passphrase only protects two secrets in the keystore's settings file. So
Flowsta open those two with the old passphrase and seal them again under the
new one, leaving every key in the store untouched. It refuses while the
keystore is running, and an interrupted change is finished by the next try.

**Why it matters.** [threat-model.md](threat-model.md): "The released app does
not yet ask for a password, so holding the device is holding the keys.
Decided; not built — it needs a careful test so existing installs are not
locked out." This is that careful method, written and tested by somebody else.
Hearth's desktop app is TypeScript, not Rust, so it would mean a small helper
program or a careful port. Read lair's own code alongside it.

## 4. Recovering a person's identity on a new computer

**Where:** [WeAreFlowsta/ProofPoll](https://github.com/WeAreFlowsta/ProofPoll),
`src-tauri/src/device_seed.rs` and `seed_adopt.rs`. **MIT licence.** ProofPoll
calls itself "a complete, working template for any desktop Holochain app".

**What it is.** The agent's key normally comes from inside the keystore and
cannot be exported. ProofPoll makes the key's seed in the app, keeps a copy in
the person's encrypted backup, and imports it into the keystore. A restore on a
new machine imports the same seed, so the person carries on as the same
author.

**The lessons they paid for, worth keeping whatever Hearth does:**
- A backup must **refuse to overwrite a backup holding a different key**.
  Otherwise a fresh install's first backup destroys the key the person needs
  to recover.
- Replacing a key goes in a strict order: stop everything, wipe what came
  from the old key, **check** the wipe, and only then commit the new key.
  Doing it the other way round left an install that would not start. On
  Windows, stopping needs `taskkill /T /F`, because a plain kill silently does
  nothing.
- The old key is never deleted, only set aside with a date, because records
  locked to it would otherwise be lost for good.

**Why it matters.** [hard-questions.md](hard-questions.md), question 12: can a
person recover their identity? Hearth's answer so far is "when Holochain can
export keys". This is a way that works today. It changes what a lost device
means, so it needs careful thought about custody: the seed in a backup is as
sensitive as the keystore itself.

**And a warning about Holochain's own restore.** 0.7.1 added "source-chain
restore", which [holochain-roadmap.md](holochain-roadmap.md) lists as helpful.
Flowsta's `src/restore.rs` in hc-lines found that **a restore can come back
short**. What the old device wrote last, and nobody else holds yet, is
missing, and the restore still says it finished. Writing on from there forks
the chain. They check against the last position the backup knew of before
switching the restored copy on.

## 5. Moving between Holochain versions: hc-lines

**Where:** [WeAreFlowsta/flowsta-hc-lines](https://github.com/WeAreFlowsta/flowsta-hc-lines).
**MIT licence.** Used to move Flowsta Vault's users from Holochain 0.6.1 to
0.7.1.

**What it does.** It runs the old Holochain beside the new one in the same
app, copies the person's own records across (safely repeatable, and picking up
after a crash), keeps writing to both while another of the person's devices
is still on the old version, and retires the old one when nothing needs it.

**Why it matters.** Hearth carries a circle across by writing it again, inside
one Holochain version. Holochain 0.8 is the next "line", and moving to it
means two different Holochain programs at once, which Hearth has never had to
do. Two of their ideas fit Hearth now:
- **Keep writing to both** while somebody in the circle has not upgraded yet.
  Today a circle is split during an upgrade until everybody has moved.
- **Say where a carried record came from.** ProofPoll's carried records keep
  their original author, time and signed action, and who carried them, so a
  reader can check them without the old network.

Also from hc-lines: lair 0.7.1 opens a 0.6.3 keystore unchanged, but copying
one must include its `-wal` and `-shm` files, because "the write-ahead log can
hold most of the keys".

## 6. Smaller things

**Signed update manifests.** [WeAreFlowsta/flowsta-update-check](https://github.com/WeAreFlowsta/flowsta-update-check)
(MIT). The app carries its trusted signers, and before installing an update
it checks a signed list of the release's files and their hashes, offline. A
beta is never offered to a stable install. Hearth has automatic updates
switched **off** because the default would have let somebody else's
repository push updates to Hearth users ([building-it.md](building-it.md)).
This is a sound way to switch them back on.

**Holochain performance patches.** [WeAreFlowsta/holochain-build](https://github.com/WeAreFlowsta/holochain-build)
builds Holochain 0.7.1 with six small patches, under strict rules: nothing
another device can see changes, and each patch ends when upstream fixes it.
Three bear on Hearth:
- 0003 and 0004 stop writes getting slower as a chain grows (one test went
  from 1,763 to 549 seconds for 8,000 writes).
- 0005 makes capability-grant reads, which every pass uses, start from the
  grants.
- 0006 stops a large sync failing with "too many SQL variables"
  (holochain#6062), which a circle full of video pieces could one day meet.

It is also the model to follow if Hearth ever builds its own Holochain, for
example to fix the `rustls` flaw ([threat-model.md](threat-model.md)). There
is no licence on the repository itself; each patch is offered under the
licence of the files it changes.

**Lessons from Moss 0.16** (release notes, October and November 2026):
- On a slow machine, joining a group could fail with "Failed to read your
  profile". They now make the first call to each new cell on its own, retry
  the start-up step if it times out, and show plain text with a Retry button.
  Hearth makes a new cell for every circle.
- A watchdog stops the keystore and Holochain if the app itself dies, and
  leftover processes are cleaned up at start. Hearth met the leftover-keystore
  problem on its first two-machine test ([building-it.md](building-it.md)).
- Moss's "who is online" now uses Holochain's direct signals, with no app code
  on either end. Hearth's call signalling could do the same.

**Wind Tunnel** ([holochain/wind-tunnel](https://github.com/holochain/wind-tunnel),
MIT) is Holochain's load-testing tool. It can run any app from a URL, and
already has scenarios for remote calls, signals, sync lag and zero-arc
readers. A small Hearth scenario could time a join and a pass read properly,
which [latency.md](latency.md) has so far only estimated by hand.

**Networks with no internet at all.** Lightningrod Labs have experimental
work on two other ways for Holochain devices to talk, in branches of
[their kitsune2](https://github.com/lightningrodlabs/kitsune2):
- `transport-reticulum`: Holochain over [Reticulum](https://reticulum.network/),
  a mesh network that routes without IP addresses, with a backend from
  Beechat. It works over ordinary networks first; radio is "a downstream
  concern". Last changed April 2026.
- `broadcast-transport` (July 2026, a draft design): no connections at all.
  Devices transmit and listen, over Wi-Fi broadcast, Bluetooth, sound, or "a
  screen flashing to a camera". It notes that Holochain's data suits this
  unusually well, because everything is signed and checks itself.

For Hearth this connects to the problems nobody has solved: a paramedic with
no signal, or two phones in a room with no network. A record shown screen to
camera is not far from the QR-code ideas already in
[to-a-product.md](to-a-product.md). Experimental; watch, don't build. It is
probably the mesh work with Volla that was heard about, but no public link
between this and Volla was found.

**Also seen, less relevant:** Volla's `recovery-zomes` (phone backups held
encrypted on friends' devices, with base64 invite codes that carry everything
needed to join; no licence, ideas only); Flowsta's `flowsta-dht-node` (Apache
2.0, a community node holding only public data, kept current by a signed
manifest, which is a model for Hearth's "edge node" idea); and Your Own AI
(AGPL-3.0, so ideas only), which syncs a person's data between their own
devices, encrypted with their own key.

---

## Watching

| What | Where | Why |
|---|---|---|
| Holochain 0.7.2 | holochain/holochain releases | the many-networks fix above |
| A LICENSE file for the in-process plugin, and a 0.7.1 version | holochain/android-service-runtime | the phone route |
| hc-lines reaching 1.0 | WeAreFlowsta/flowsta-hc-lines | the move to Holochain 0.8 |
| Broadcast and Reticulum transports | lightningrodlabs/kitsune2 branches | no-internet use |
