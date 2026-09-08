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

**Whether a leacher can still make and receive remote calls is reasoned, not
tested.** The arc governs what an agent stores and gossips; calls are
agent-to-agent messaging. Nothing found suggests the arc gates them, and the
word "leacher" implies fetching from peers works. **Not proven. Prove it before
building on it.**

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
