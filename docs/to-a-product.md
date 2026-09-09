# From a demo to something somebody could rely on

**Status: roadmap, 2026-09-08.** Code only — everything outside the code
(funding, partners, clinical safety, procurement, support) is deliberately left
out. Ordered by what blocks what, not by effort.

The honest summary: **there are two problems that decide whether "product" is
even the right word, and neither is the outer ring.**

---

## 0. The one nobody has written down: upgrades

**A circle's identity is the hash of its code.** Change the integrity zome and
every existing circle becomes unreachable — not broken, not migrated,
*unreachable*, because the new build is a different network. Everyone's records
are still on their own machines and no version of the app will show them.

Today that is fine. Nobody has a real record in this, and when something changes
we restart the demo. **The moment one real person keeps one real record here,
that stops being acceptable**, and a routine bug fix in the wrong file destroys
their work silently.

**Decided 2026-09-08: the integrity zome is frozen** — with one question still
open, see [the collision with the outer ring](#and-it-collides-with-the-freeze).

From now on, `dnas/aboutme/zomes/integrity/aboutme/src/lib.rs` — and anything
else that feeds the DNA hash — changes only for a reason worth stranding every
existing circle for. The interface and the coordinator zome stay free to change
as much as they like, which is where nearly all the work happens anyway.

The cost is real and should be said plainly: **the data shape has to be right
now.** Adding a variant to an enum, adding a field to a struct, renaming a type
— every one of those changes the hash. A field nobody thought of in 2027 is not
a commit, it is a migration, planned as such. That is the price of never
destroying somebody's record with a bug fix, and it is worth paying.

### The freeze is stricter than expected: not even comments

**Measured, not assumed, 2026-09-08.** The obvious place to write "this file is
frozen" is the top of the frozen file. So a seventeen-line comment was added to
`integrity/aboutme/src/lib.rs` and the wasm rebuilt, expecting the hash to be
unchanged.

It changed — `dd942cac…` to `941f0b41…`. Reverting the comment and rebuilding
gave `dd942cac…` back exactly, so builds *are* reproducible; **the comment
itself was the difference.** Almost certainly panic and debug location strings,
which carry line numbers, and adding lines at the top moves every one of them.

Two consequences, and the first is the one that matters:

- **The freeze means the file, not just the types.** No comments, no
  reformatting, no reordering, no touching it at all. A tidy-up commit on that
  file is as destructive as a schema change.
- **The note saying so cannot live in the file it describes.** It lives here,
  and in the README, and nowhere else.

That is an unusually literal kind of freeze and it needs to be understood by
anybody who works on this, including a future version of whoever wrote it.

The two answers this was chosen between:

- **Freeze the integrity zome.** Everything else — interface, coordinator zome,
  behaviour — can change freely without touching the DNA hash. **Chosen.**
- **Build a migration path.** Export from the old circle, import into the new,
  re-invite everybody. This is the same answer the project already gives to
  revocation and to appointing a second yes, so it is at least consistent — but
  it has never been built, and "everybody rejoins" is a heavy thing to ask of
  six people including one who is unwell.

**Nothing else on this page matters if an update can quietly strand somebody's
record.** This is first because it is first.

### What is waiting for the next version of the integrity zome

Because the file cannot be touched, things found in it queue up here rather
than getting fixed. There is one so far, found in an audit on 2026-09-09.

**Two of the six jobs Holochain hands out are not checked.**

When somebody writes something, Holochain does not ask one machine whether it
is allowed. It asks several, each looking at a different aspect of the same
act — is this a well-formed record, is this entry allowed, is this link
allowed, is this deletion allowed. The zome answers four of those questions and
says "fine" to the other two, because those two fall through the catch-all at
the bottom of `validate`.

The two that are not checked are the ones that hold **the record as a filed
document** and **the list of what an agent has done**. The ones that *are*
checked are the ones that hold the contents and the links, and every list this
app reads is reached by following a link. So no attack was found: a forged
entry is refused by the machine holding that kind of entry, and a forged link
is refused by the machine holding that link, and nothing in the app ever
reaches a record any other way.

But "no attack was found by the person who wrote it" is exactly the sentence
this project has learned to distrust, and there is a precedent sitting in the
test file: link creation was once entirely unchecked, and that was found by
review rather than by anybody's tests. This is the same shape of gap, one
layer up.

So it is written here, and in [`what-is-proven.md`](what-is-proven.md), and it
is the first thing to fix whenever the integrity zome next moves. The fix is
small — a handful of extra arms in the `match`. It is only the freeze that
makes it expensive.

## 0b. Windows and Linux builds are different networks

**Found 2026-09-08, by the CI check written to enforce the freeze.** The check
built the integrity zome on Linux and printed its hash so it could be pinned.
It did not match the Windows one.

    Windows  dd942cac…    where the released installer is built
    Linux    7ca250f3…    GitHub Actions, and anybody building from source

Identical source. Identical pinned compiler — `rust-toolchain.toml` fixes
rustc 1.98.0 precisely so this cannot happen. Different wasm anyway.

**Different wasm is a different DNA hash is a different network.** A circle made
in the installer published today and a circle made from a Linux build are not
the same circle. Their members cannot find each other, and nothing anywhere
says so.

**This makes an instruction in the README wrong.** It tells Linux and macOS
users to build from source — which hands them an app that cannot talk to any
Windows user. The two lines in `FROZEN.sha256` are the proof.

**Likely cause, not yet confirmed:** source paths embedded in panic and debug
strings, `C:\Users\user\…` against `/home/runner/…`. Rust has
`--remap-path-prefix` for exactly this, and a `[build] rustflags` entry in
`.cargo/config.toml` would apply it to every build on every machine.

**Why it was not fixed on the spot.** The fix changes the hash again, and
therefore the DNA of the release published hours earlier. That is the right
thing to do — the free window is open, nobody has a real record — but it
retires a published artefact and should be a deliberate decision rather than a
late-night one.

**What it means for the freeze.** The freeze is not settled. A hash pinned per
platform is not one frozen thing, it is two, and the honest position is that
this must be fixed *before* the freeze means anything. Sequence: make the build
reproducible across platforms, confirm one hash everywhere, publish that as the
release, and freeze from there.

**It also quietly explains a gap in the testing.** The adversarial suite runs on
Linux in CI. The installer is Windows. They have been testing the same rules on
a different DNA all along — harmless, since the source is identical, but it
means "the tests pass" and "the shipped app is correct" were never quite the
same sentence.

## 1. The record is not encrypted at rest

Entries are validated, signed, and reachable only by people the membrane
admitted. But **every member's machine holds the contents in the clear**, and
keeps them after they leave.

For the About Me scope this is smaller than it sounds — nothing clinical, by
design. It is still the thing that stops this being trustworthy with anything
more, and the README already notes it should be encrypted to circle members at
application level regardless.

The hard part is not the encryption. It is **key management for a group whose
membership changes**: new members must be able to read what was written before
they arrived, or must not, and that has to be decided rather than fallen into.

## 2. Losing a device loses the person, not the record

If Margaret loses her phone, the record survives — every member holds a full
copy. **Margaret does not.** Her keys are gone, so she cannot write, cannot be
recognised as herself, and cannot rejoin except as a new stranger who must be
invited into her own circle.

For the population this app is for, a lost or broken phone is not an edge case.
There is currently **no recovery, no backup, and no second device**, and all
three are the same problem: one person, one keypair, one machine.

This is genuinely hard in an agent-centric system and it is the thing most
likely to be underestimated.

## 3. One person, two devices

Related but separate: a phone *and* a laptop, both being Margaret. At present
they would be two different members of the circle with two different
identifiers, which is wrong in a way that shows — she would appear twice in her
own list of people.

## 4. Mobile

**An About Me record that only runs on Windows is not a product.** The person it
is about is unlikely to be at a desk, and the professional reading it certainly
is not.

The README already establishes the routes exist —
`holochain/android-service-runtime` pins 0.7.0 and runs a system-wide conductor
as a foreground service, and iOS became plausible when 0.7.0 added the `wasmi`
interpreted backend. **Neither has been tried here.** "A route exists" and "it
runs" are different claims, and only one of them is worth anything.

---

## 5. The outer ring

Now the new work, and by comparison it is small. Scoped to five steps and
nothing else:

1. The person grants a pass to **one section**.
2. The other party pastes it.
3. They read **that section and nothing else**.
4. The person withdraws it.
5. The next read fails.

What that needs:

- **Capability grants with assigned access**, granting a single read function.
  The README is already clear that grants "gate remote calls into your own cell"
  — which is exactly right for this, and exactly why they were wrong as a
  revocation mechanism. Get that distinction into a comment before writing a
  line, or it will be re-litigated.
- **Answering from the circle rather than from the person.** See below: this
  removes the offline caveat and is a better design, but it is not free.
- **Section granularity.** `AboutMe` is one entry with nine fields, so the
  filtering happens at the call boundary: the caller is handed one field and
  never sees the entry. That is simpler than splitting the entry and should stay
  that way.
- **Withdrawal**, and a screen showing which passes are outstanding — a person
  cannot withdraw what they cannot see they gave.
- **An honest failure when nobody answers.** Not a spinner. Something that says
  so and says what to do, because this will happen in a hallway to somebody in
  a hurry.

**Explicitly not in this piece of work:** discovery, a professional's interface,
offline caching of granted sections. Naming them as excluded is what stops the
work sprawling.

### Who answers the request — and why the first answer was wrong

The first version of this note said the outer ring only works while **the
person's own device** is reachable, and called that its unavoidable cost. That
was too quick.

**The pass is an entry on the person's chain, so it is published to the DHT, so
every circle member can see it.** And every circle member already holds a
complete copy of the record. So there is no reason the person has to be the one
who answers: any member who is online can check the pass and serve the one
section it names, from their own copy.

That is strictly better. The record already survives a flat battery for circle
members; this extends the same property to the outer ring, and the always-on
device the README already recommends for joining does this job too.

**What it costs, honestly:**

- **The enforcement moves from Holochain to us.** A capability grant is checked
  by the conductor. A pass checked by each member is checked by our code on
  their machine, so a member running a modified client could serve anything.
  **This adds no new exposure** — that member already holds the plain text and
  could publish all of it — but it is a weaker mechanism and should not be
  described as if the conductor were enforcing it.
- **Withdrawal becomes eventually-consistent.** The person deletes the pass;
  that deletion has to reach the members before they stop honouring it. "She
  withdrew it and it stopped working everywhere at once" would be a lie.
- **The reader has to reach the circle at all.** This is the unsolved part. A
  remote call goes between peers in one network, and somebody outside the
  membrane is not in that network. Either the pass admits them as a member who
  stores nothing — the README already notes `target_arc_factor: 0` gives
  exactly that, a node that participates without storing — or the outer ring
  needs a different mechanism altogether. **This has not been checked against
  the Holochain source and must be before any of it is built.**

The third point is the real work, and it is the difference between a good idea
and a design.

### Checked against the 0.7.0 source, 2026-09-08

Read in the vendored crates, not recalled from documentation.

**Zero-arc nodes are real, supported, and have a name.** From
`holochain_conductor_api-0.7.0/src/config/conductor.rs`:

> "The target arc factor to apply when receiving hints from kitsune2. In normal
> operation, leave this as the default 1. **For leacher nodes that do not
> contribute to gossip, set to zero.**"

And the factor genuinely produces nothing: `apply_arc_factor` in
`holochain_p2p-0.7.0/src/local_agent.rs` multiplies the arc span by the factor,
and returns `DhtArc::Empty` when the result is zero — with a fixture test
asserting exactly that. Every agent otherwise joins with `DhtArc::FULL`
(`spawn/actor.rs`), which is where the "everybody holds everything" property
comes from.

So **a node that participates without storing is a first-class thing in 0.7.0**,
not a trick.

**But it is conductor-wide, not per-circle.** `target_arc_factor` sits in the
network config for the whole conductor and is applied to every space it joins.
There is no way to be a full member of your mother's circle and a leacher in a
stranger's from the same installation.

That is not fatal — it just says what the shape has to be. **The professional's
app is a different build, or at least a different mode**, that is a leacher
everywhere. Which is arguably what it should be anyway: a nurse should not
accumulate copies of the records of everybody she visits, and this makes that
structural rather than a promise.

**There is a whole test suite for this**, in `holochain-0.7.0/tests/tests/zero_arc/`
— 621 lines of it, with `target_arc_factor = 0` set explicitly. What it proves:

- `get_missing_from_coordinator` — "zero arc nodes can use the various get host
  functions to get missing records, actions and entries from authorities."
- `self_validation_get_missing` — they can fetch what they need to validate.
- `zero_arc_get_details_discover_updates` — they see updates made by others.
- `zero_arc_delete_link_get_links` — deleted links stop coming back.

So **a leacher can read from the DHT**, tested by the people who wrote it. That
is more than was hoped for.

**And it changes the design, because reading from the DHT is the wrong tool
here.** A `get` returns the whole entry. `AboutMe` is one entry with nine
fields, so a leacher who can `get` can read *everything* — which is precisely
what the outer ring exists not to allow.

Per-section access needs the request to go through code that can filter, which
means a **remote call into a member's coordinator zome**, not a DHT get. And
remote calls from a zero-arc node are **not covered by those tests** — nothing
in the suite touches `call_remote` or capability grants at all.

So the open question is narrower and sharper than before:

> Can a zero-arc node make a capability-gated remote call to a circle member,
> and can that member answer it?

Everything else about the leacher route is now established. **This one question
is the whole risk**, and it is answerable in an afternoon with a test rather
than an argument.

**A second consequence worth writing down now.** If a pass-holder is admitted to
the network at all, they can `get` the whole entry regardless of what the
interface offers them — the membrane is the boundary, not the zome function. So
per-section access is **not enforceable against a determined reader** by this
route. It is enforceable against an ordinary one using the app as built, which
is a real but much weaker claim, and the difference must never be blurred in
anything this project publishes.

If per-section access has to hold against a determined reader, the sections have
to be **separately encrypted**, and this becomes a key-management problem rather
than an access-control one — which is item 1 on this page, arriving from a
direction nobody expected.

### ⚠️ And it collides with the freeze

**Getting into the network at all requires passing `check_membrane`, which is in
the frozen file.**

A pass-holder is not a founder and has no invitation. Admitting them means a new
`Membrane` variant, or a new membrane-proof shape, or both — and every one of
those changes the DNA hash and strands every circle in existence.

So the two decisions taken today are in tension, and it has to be resolved
deliberately rather than discovered later:

- **Option A — one more change, then freeze.** Nobody has a real record in this
  yet. Make the outer ring's integrity changes now, in one deliberate piece of
  work, and freeze from that release onwards. The freeze exists to protect real
  records, and there are none.
- **Option B — freeze from today, and the outer ring lives in a separate DNA.**
  A second cell the professional joins, with the circle publishing sections into
  it. Keeps the promise made today, and is a good deal more machinery.
- **Option C — the outer ring never joins the network**, and the pass is
  honoured some other way entirely. No mechanism for this has been found.

**Recommended: A.** The freeze is worth everything the day somebody real depends
on it and costs almost nothing today, so spend the last change on the thing that
was always going to need it. But it is a decision, not a detail, and taking it
by accident would be the worst of the three.

## 6. Discovery

A pass has to reach the nurse. Paper solved this with a sticker on a fridge; see
[`what-to-borrow.md`](what-to-borrow.md). **Nothing is built**, and a record that
is never found is the failure that matters most.

Partly a code problem — a QR code, a lock-screen entry, a short link — and
partly not, which is why it sits here rather than inside the outer ring.

## 7. Joining takes about ninety seconds

Measured and understood; see [`latency.md`](latency.md). Not fixed. It is a
rendezvous cost rather than a bug, but a person watching a blank screen does not
care whose fault it is.

## 8. Accessibility, which is not optional here

The people this is for include those with learning disabilities, dementia, and
communication difficulties, and the person writing it is often exhausted. That
makes screen-reader support, plain language, text scaling and cognitive load
**functional requirements rather than polish** — and none of it has been tested
with any assistive technology.

This is real code work and it is listed last only because everything above can
strand a record, and this cannot.

---

## What this adds up to

Roughly: **two hard problems (upgrades, encryption), two underestimated ones
(key loss, mobile), one contained new feature (the outer ring), and a long tail
that is ordinary work.**

The outer ring is the smallest thing on this page and the most interesting to
show, which is exactly why it should be built next — after the release is
proven on a second machine — and why it should not be allowed to grow.

**None of the above is a reason the demo should not exist.** A demo is for
showing that something is possible. This list is what stands between possible
and relied upon, and knowing it precisely is worth more than having less of it.
