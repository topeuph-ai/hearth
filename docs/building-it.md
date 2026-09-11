# Building it, running it, and the traps already paid for

Everything operational, in one place. If you only want to try the app, you want
the [README](../README.md) instead — this is for somebody building or reviewing
the code.

The second half of this page is a list of things that have already cost this
project real time. They are all fixed here. The point of writing them down is
that nobody rediscovers them.

---

## Versions

Read from `crates/hdk/Cargo.toml` and `crates/hdi/Cargo.toml` at tag
`holochain-0.7.0` — not from documentation, which is out of date.

| | |
|---|---|
| Holochain | **0.7.0** (released 30 July 2026) |
| hdk | **0.7.0** |
| hdi | **0.8.0** |
| holochain_serialized_bytes | **=0.0.57** |

`rust-toolchain.toml` pins the Rust compiler to **1.98.0**, and that pin is
load-bearing. See [Different builds are different networks](#different-builds-are-different-networks).

---

## Getting the binaries

No nix needed. They come straight from the Holochain 0.7.0 GitHub release
(the `holochain/binaries` repo publishes them per platform, MPL-2.0):

```bash
for a in hc holochain lair-keystore kitsune2-bootstrap-srv; do
  gh release download holochain-0.7.0 --repo holochain/holochain \
     --pattern "${a}-x86_64-pc-windows-msvc.exe" -O "bin/${a}.exe"
done
```

`bin/` is gitignored. Verified versions: hc 0.7.0, holochain 0.7.0,
lair-keystore 0.7.1, kitsune2-bootstrap-srv 0.5.0.

---

## Building

> ### ⚠️ Never run `cargo build` directly
>
> Rust writes the file path of every dependency into the compiled output, for
> use in its error messages. Those paths contain your own home directory. So a
> plain `cargo build` produces a **different fingerprint on every computer** —
> and the installer released on 8 September 2026 had the string
> `C:\Users\user\...` sitting inside it.
>
> `scripts/build-zomes.mjs` strips those paths out before building. Always go
> through it.

```bash
node scripts/build-zomes.mjs           # 1. compile
./bin/hc.exe dna pack dnas/aboutme/workdir   # 2. bundle
./bin/hc.exe app pack workdir
```

Then, to run two people locally with no network involved at all:

```bash
./bin/hc.exe sandbox --piped generate workdir/aboutme.happ --run=8888 -n 2 network mem
```

`network mem` keeps everything in memory: two people, one machine, no internet,
no rendezvous server. It is the demo in its smallest possible form.

`hc sandbox generate network` also takes `--target-arc-factor`, which is how you
would make a node that takes part without storing anything.

---

## Running the demo

One command. It compiles, bundles, starts the interface, waits for it and opens
the windows — because a demo command that needs a second terminal is not a demo
command.

```bash
cd ui && npm run demo        # two people
npm run demo -- 3            # three
npm run demo -- 6            # six
```

**Two windows on one machine is the whole argument**: two separate people, two
separate stores, talking with nothing in between. Make a circle in one, invite
the other, watch the acknowledgement arrive.

Use three for anything involving a second yes — one person invites, another
agrees, a third arrives.

If it refuses to start, something is still holding port 5273. Kill Electron,
`holochain` and `lair-keystore` first.

### What actually limits how many people you can run

Not Holochain. Measured on this laptop:

| | each |
|---|---|
| `holochain` | ~52 MB |
| `lair-keystore` | ~6 MB |
| `kitsune2-bootstrap-srv` | ~9 MB, shared by everybody |

A person costs under 60 MB. **The window costs several times that, because it is
Chrome.** So the ceiling here is the browser, not the peer-to-peer part — which
is the opposite of what people assume, and worth saying out loud when somebody
asks whether it would scale.

Three agents on this laptop is about 700 MB across 13 processes, against 7 GB
free. It was once believed that three was too many here, because a run of three
stalled for ten minutes at a time and dropping to two fixed it instantly. That
was an inference, and it was wrong: measured properly, three runs clean with no
`database is locked` and no `VirtualLock` failures at all. Whatever the stall
was, it was not the agent count.

---

## Asking the running conductors instead of guessing

When a screen looks wrong, the cheapest way to find out what is actually true
is to ask every conductor the same question and compare. This found three
faults in one evening that reading the code had not.

hc-spin prints an admin port per conductor into its own output:

```bash
grep -oE 'admin_port":[0-9]+' demo.log | sort -u
```

From there, `@holochain/client` will talk to them — it is already in
`ui/node_modules`, so a script run from `ui/` can import it by name. Two
things are easy to lose an hour to:

- **Every call needs signing credentials**, per cell, or it fails with *"no
  signing credentials have been authorized for cell ..."* — which looks like a
  permissions bug in the app and is not. Call
  `admin.authorizeSigningCredentials(cellId)` first.
- **The app websocket needs an origin of `hc-spin`** (`wsClientOptions: {
  origin: "hc-spin" }`), because that is what the conductor was told to allow.

What it is worth asking: `who_holds_this` distinguishes a circle from a room
in one call, and after that `get_members`, `get_knocks`,
`get_pending_members` and `who_agrees_here` say what each device believes.
**Comparing devices is the whole point** — "Agent 3 can read the room but has
no knock of its own" is a diagnosis; "the knock did not appear" is a symptom.

---

## Getting two machines to talk

By default every window in the demo uses **one rendezvous server running on your
own machine**. Two laptops each running `npm run demo` each start their own, so
they never meet. Not because peer-to-peer fails across machines — because they
are asking two different servers who exists.

**A rendezvous server is a place to leave your address, not a place your data
goes through.** It never sees a record, and if it disappears, people who have
already found each other carry on. That distinction matters for the argument:
needing somewhere to meet is not the same as needing an operator.

Run your own with the binary already in `./bin`:

```bash
./bin/kitsune2-bootstrap-srv --listen 0.0.0.0:8888
```

Then point every machine at it:

```bash
npm run demo -- 2 --bootstrap http://192.168.1.20:8888
```

Every machine needs the same address **and** a build from the same source.

**Verified 8 September 2026**, on one machine but going out through the external
server rather than looping back, which is the same code path a second machine
takes:

- the conductors came up pointing at
  `bootstrap_url: Url2 { url: "http://192.168.1.89:8888/" }`, and the same for
  `relay_url`, instead of the local address they default to
- that server logged 65 connections and 8 relay upgrades from them

**If something goes wrong, check reachability first.** Open
`http://<that-address>:8888/health` in a phone's browser. It answers `{}` — two
bytes — so a blank-looking page is a pass and "site can't be reached" is a
firewall. That took ten seconds and proved a Windows machine running Norton
needed no firewall rule at all.

Still untested: a genuinely second computer, and whatever a home router does to
the hop.

---

## Different builds are different networks

**This is the thing most likely to defeat two machines, and it is not the
network.**

A circle is identified by a fingerprint of the compiled code. Two people who
build with different compilers get different fingerprints, which are genuinely
separate networks — and it fails in the worst way there is. Invitations are made
and accepted. Nothing errors. Nobody ever arrives.

It looks exactly like a firewall problem, and no amount of firewall work will
fix it.

Two things guard against this:

**1. The compiler is pinned** in `rust-toolchain.toml`. Do not remove it. Raise
it only deliberately, because raising it changes the fingerprint and everybody
has to be on the new build before anybody can reach anybody.

**2. `npm run demo` prints the fingerprint on startup:**

```
This network: uhC0ksZDuOqtYLhruRSamMI6uvlXHY7WNvHAONxJjhdLdXZCa0qvD
```

Every machine meant to reach the others must print that exact line. **Check this
before investigating anything else.** It is one line and it rules out the whole
class of problem.

You can also ask directly:

```bash
./bin/hc dna hash dnas/aboutme/workdir/aboutme.dna
```

### Windows and Linux will always disagree, and that is not a bug

The path-stripping above keeps whichever slash the operating system uses:

```
Windows  /cargo\registry\src\...
Linux    /cargo/registry/src/...
```

Different bytes, different fingerprint, different network. No flag on stable
Rust fixes this.

**So the released `.webhapp` is the canonical build.** Anybody making a desktop
app for another platform assembles it from that file rather than compiling. See
[`FROZEN.sha256`](../dnas/aboutme/zomes/integrity/aboutme/FROZEN.sha256), which
records both fingerprints.

---

## Building the desktop app

`npm run webhapp` in `ui/` produces `workdir/hearth.webhapp` — the rules, the
app and the interface in one file.
[`holochain/kangaroo-electron`](https://github.com/holochain/kangaroo-electron)
turns that into an installer.

**Use the released `hearth.webhapp`, not one you built**, unless you are
deliberately making a separate network.

```bash
git clone --depth 1 https://github.com/holochain/kangaroo-electron.git hearth-desktop
cd hearth-desktop
# In kangaroo.config.ts: appId 'uk.topeuph.hearth', productName 'Hearth'.
gh release download --repo topeuph-ai/hearth --pattern 'hearth.webhapp' --dir pouch/
npx yarn@1 install
npx yarn@1 setup       # fetches and checksums the Holochain binaries
npx yarn@1 build:win   # or build:linux, build:mac-arm64, build:mac-x64
```

Produces a setup `.exe` of about 115MB with Holochain and its keystore inside.
**Install it on two machines, unplug the router, and they still find each other
on the local network.** That is the demonstration, and it is a different thing
from two windows on one laptop.

Before sending it to anybody, read the three Windows warnings in the
[README](../README.md#trying-it).

### Three things that cost time here

- **Kangaroo's README says `build:windows`. The script is `build:win`.** It
  fails instantly with `error Command "build:windows" not found`.
- **`corepack enable` needs administrator rights on Windows.** `npx yarn@1`
  works without them.
- **Kangaroo ships pointing at Holochain's public test servers**, which carry no
  availability guarantee. Fine for a demo. Before anybody real uses this you
  need your own — and **changing those addresses after release splits the
  network in two**, so it is a decision to make before, not after.

### After changing the zomes, clear the app's data

**Kangaroo installs the app on first run only.** Rebuild with new code and it
will cheerfully keep running the old version, because something is already
installed inside it.

The symptom is the worst kind: it starts, reports ready, shows no error, and the
interface either does nothing or fails against functions that no longer have the
shape it expects.

Move the data aside rather than deleting it, in case something in there
mattered:

```bash
mv ~/AppData/Roaming/uk.topeuph.hearth/0.1.x/default \
   ~/AppData/Roaming/uk.topeuph.hearth/0.1.x/default-old
```

The next launch installs fresh.

---

## Antivirus that inspects secure connections

**Symptom:** the app installs, launches, starts up, shows no error, and never
finds a single other person. In the log:

```
probe failed: ... https://dev-test-bootstrap2.holochain.org/ping ...
  invalid peer certificate: UnknownIssuer
Failed to connect to relay server: tls connection failed: invalid peer
  certificate: UnknownIssuer
```

**Cause.** Norton re-signs every secure connection with its own certificate so
it can read what is inside. Confirmed here by asking what certificate is
actually being served:

```
cert subject: CN=dev-test-bootstrap2.holochain.org
cert ISSUER : CN=Norton Web/Mail Shield Root,
              OU=generated by Norton Antivirus for SSL/TLS scanning
```

Windows trusts that certificate, so browsers are perfectly happy. **Holochain's
networking does not use the Windows certificate store** — it carries its own
list, sees an issuer it has never heard of, and refuses.

Any antivirus with HTTPS scanning does this: Norton, Kaspersky, Avast, ESET, and
most corporate networks. An NHS laptop very likely will.

**Excluding the one host fixes it, and this is verified.** Afterwards the same
check returns the real certificate while everything else stays inspected:

```
dev-test-bootstrap2.holochain.org  ->  CN=YE2, O=Let's Encrypt   (real)
holochain.org                      ->  Norton Web/Mail Shield Root
github.com                         ->  Norton Web/Mail Shield Root
```

Restart the app afterwards — it does not retry a connection that already failed.

In order of preference:

1. **Exclude `dev-test-bootstrap2.holochain.org`** from HTTPS scanning. Verified.
2. Turn HTTPS scanning off while demonstrating.
3. Exclude the app's bundled `holochain-*.exe` from inspection.

**Nothing in this repository can fix this.** The certificate is rejected inside
Holochain's own networking, before any of this project's code runs. It is worth
warning anybody you send the installer to, because the app gives no clue — it
just sits there looking like peer-to-peer does not work.

`npm run demo` never hits this, because its rendezvous server is plain HTTP on
your own machine and there is nothing to inspect. So "works on the dev machine,
dead as an installed app" is an expected combination, not a contradiction.

---

## Traps already paid for

### Pass URLs to hc-spin as `--flag=value`, never `--flag value`

This one cost an afternoon.

hc-spin is an Electron app, and Electron hands its arguments to Chromium, which
treats anything starting with a URL scheme as a page to open. Written as two
separate words, `http://host:8888` is such a thing — and Electron **exits
immediately with code -1 and prints absolutely nothing.**

Bisecting the value is what found it, and the result is oddly specific:

| value | result |
|---|---|
| `host:8888` | fine |
| `//host:8888` | fine |
| `http:8888` | **dies** |

It is the scheme, and it is fatal against any option, not just the URL ones.
Written as one word the argument starts with `--`, so Chromium reads it as a
switch it does not recognise and ignores it, while the argument parser still
gets the value. `demo.mjs` does this and says why.

### hc-spin needs the binaries on PATH, and says nothing when they are missing

It runs `kitsune2-bootstrap-srv`, `holochain`, `lair-keystore` and `hc` by bare
name and **does not bundle them**. When they are missing the only symptom is an
empty error:

```
[hc-spin] | [hc run-local-services] ERROR:
```

Empty because the *launch* failed rather than the program, so there is nothing
to report. `demo.mjs` puts `bin/` on PATH and checks everything is present
before starting.

### Vite binds to `[::1]` unless told otherwise

The check that waits for the interface connects to `127.0.0.1`, and on Windows
the two never meet — which looks exactly like the server failing to start.
`demo.mjs` passes `--host 127.0.0.1` so everything agrees.

### Six Holochain-specific ones

1. **`getrandom` refuses to build for wasm32.** The hdk registers its own
   backend that asks the conductor for randomness, so the build must set
   `--cfg getrandom_backend="custom"`. Without it you get an error that looks
   entirely unrelated. See `.cargo/config.toml`.
2. **`holochain_serialized_bytes` must be a direct dependency** even though
   nothing in the source mentions it. The `hdk_entry_helper` macro expands into
   code that names the crate at the caller's root.
3. **The hdi 0.8 flat-op API changed.** `FlatOp::StoreEntry` is gone. Creates
   are `FlatOp::CreateEntry(OpEntry::CreateEntry { .. })`, updates are a
   separate `FlatOp::Update(OpUpdate::Entry { .. })`, `get_links` takes
   `(LinkQuery, GetStrategy)` rather than a built input, and `author` on a
   `TypedAction` is a method rather than a field.
4. **Manifests are `manifest_version: "0"`, not `"1"`** — despite `"0"` looking
   like a placeholder.
5. **Manifests use `path:`, not `bundled:`.** Most tutorials say `bundled`. That
   was an older format and `hc` 0.7 rejects it outright.
6. **Properties must be expressible as YAML.** A struct holding a raw key
   serialises to a byte array, and anything converting the result back into a
   bundle then fails with *"DnaDef properties were not YAML-deserializable:
   invalid type: byte array"*. Hold keys as base64 text instead. This only shows
   up when something round-trips, such as a test harness, so it can hide for a
   long time.

### `.cargo/config.toml` must be committed

A stock `.gitignore` containing `.cargo/` will silently exclude it, and the
project then fails to build for everybody else with an error pointing at
`getrandom` rather than at the missing file.

### The wider lesson

Every one of the six above was found by reading the crate source in the local
cargo registry, or the Holochain repository at tag `holochain-0.7.0`. **None of
them were in the documentation**, and several tutorials say the opposite.

Holochain's published material lags its releases. **Read the source at the tag.**

---

## The surrounding ecosystem, as of September 2026

**Use `holochain/kangaroo-electron`** for packaging — first-party, pinned to
0.7.0, updated within a day of the release. Desktop only.

**Do not depend on `p2p-shipyard`**: pinned to 0.6, last commit to main 15 May
2026, and the studio behind it has been quiet since mid-July. Holochain has also
paused Launcher development in favour of standalone apps.

**Worth reading, not building on: Moss / The Weave.** Their group DNA gives each
group, and each tool within it, its own private network — architecturally that
is a care circle. But it is alpha, pinned to 0.6, and **has no licence file**,
so all rights are reserved by default.

**Android has a first-party route.** `holochain/android-service-runtime` pins
the same versions as this project and runs a system-wide conductor as a
foreground service, which

> can run persistently, even when the app is closed, ensuring that you can be a
> reliable contributor to the peer-to-peer networks of your apps

That is the answer to "phones suspend background apps", and it would make a
family member's Android phone a dependable always-on member. Young, 4 stars, no
licence file — but first-party and on our version.

**Others worth knowing:**

- `holochain/binaries` — per-platform binaries, MPL-2.0. No nix needed.
- `holochain/hc-spin` v0.700.0 — running apps in development.
- `holochain/scaffolding` v0.700.0 — supports 0.7 and would have generated this
  structure. Hand-rolling it cost time.
- `holochain/ai-tools` — a first-party Claude Code skill for Holochain. **It
  targets a version behind this project** and would give wrong pins here.
- `holochain/hc-http-gw` — a gateway from ordinary web software into Holochain.
  Irrelevant now; relevant if an NHS system ever needs to read a circle.
- `holochain/peerkit` — the Foundation's own experiment in a different
  direction, with *"no Rust dependency, and a lighter footprint where deep
  validation is layered on top rather than built in from the start."* Not a
  reason to move — validation built into the substrate is exactly what the
  no-operator argument rests on — but worth knowing they are hedging.
- Tests use **`sweettest`**, not tryorama.

**The pattern:** first-party tooling tracks the core within a day. Third-party
runtimes run about a minor version behind and move slowly. Take no downstream
dependency you do not control.
