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

There are two honest answers and the project has to pick one:

- **Freeze the integrity zome.** Everything else — interface, coordinator zome,
  behaviour — can change freely without touching the DNA hash. This is a real
  and disciplined option, and it means the data shape has to be right *now*.
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
- **Section granularity.** `AboutMe` is one entry with nine fields, so the
  filtering happens at the call boundary: the caller is handed one field and
  never sees the entry. That is simpler than splitting the entry and should stay
  that way.
- **Withdrawal**, and a screen showing which passes are outstanding — a person
  cannot withdraw what they cannot see they gave.
- **An honest failure when the device is off.** Not a spinner. Something that
  says the person's device is not answering and what to do about it, because
  this failure will happen in a hallway to somebody in a hurry.

**Explicitly not in this piece of work:** discovery, a professional's interface,
offline caching of granted sections. Naming them as excluded is what stops the
work sprawling.

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
