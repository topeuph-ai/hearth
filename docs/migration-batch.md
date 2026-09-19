# The migration batch: every change to the frozen rules, in one go

**Drafted 19 September 2026, for Ceri to decide item by item.** Nothing here is
built.

## Why one list

The rules a circle is built from live in one file,
`dnas/aboutme/zomes/integrity/aboutme/src/lib.rs`, and changing it — even a
comment — makes a different network. Every existing circle has to be moved
onto the new version. So changes to that file are saved up and made **once**.
Anything missed now costs another move later.

Released Hearth (0.2.4) is untouched while this is built. It happens on a
separate branch, and nothing is released until circles can be carried across
(see [What has to come with it](#what-has-to-come-with-it)).

## How to read it

Each item says what it is, why, how big it is, and **my recommendation**. The
last column is yours: **Yes** (in this batch), **No**, or **Later** — later
means a second migration, so it is only worth it for things that are genuinely
not ready.

Sizes: **S** a day or two, **M** about a week, **L** several weeks, **XL**
needs its own design first.

## The list

### Already owed — faults and loose ends in the current rules

| # | What | Why | Size | Recommend | Ceri |
| --- | --- | --- | --- | --- | --- |
| 1 | **Check the two kinds of write that are waved through** | Holochain asks several questions about every write; the rules answer four and wave two through (the record as a filed document, and each person's list of what they have done). No attack was found, but the same shape of gap happened once before. See [what-is-proven.md](what-is-proven.md#two-of-the-six-checks-holochain-offers-are-not-switched-on) | S | **Yes** | |
| 2 | **Sign the name on an invitation** | The name the holder gives somebody she lets in is carried beside the signature, not inside it, so it could be altered on the way to the second person. Not a way in — a way to mislead the person deciding | S | **Yes** | |
| 3 | **Limits every device checks** — 500 words a section and a suggestion, a cap on knocks per person at a door | Today the app keeps to these, but a changed copy need not, and could fill other people's disks or flood a door | S | **Yes** | |

### Removal

| # | What | Why | Size | Recommend | Ceri |
| --- | --- | --- | --- | --- | --- |
| 4 | **Marking somebody as gone** — the holder writes a departure into the circle | The everyday removal: every app stops showing them the circle, and anything they write afterwards is visibly from somebody removed. Moving the circle stays for the serious cases | M | **Yes** | |
| 5 | **Delete the circle from the removed person's device** | Their own app, on seeing the departure, switches the circle off and deletes it. Needs item 4; the deleting itself is outside the frozen file | S | **Yes**, with 4 | |
| 6 | **Encrypt the record, with a new key on removal** | Everything written after somebody is removed arrives on their device locked with a key they were never given, so even a modified app shows nonsense. The strongest protection on this list, and the largest change: the network can no longer read what it is checking, so some checks move into the app | XL | **Yes, but designed first** — its own design note before code, because it touches every entry type | |

### The About Me standard

| # | What | Why | Size | Recommend | Ceri |
| --- | --- | --- | --- | --- | --- |
| 7 | **Photos, sound and video** — designed for all three, built photos first | Decided; see [multimedia.md](multimedia.md) | L | **Yes** | |
| 8 | **Coded values** — an optional code beside each section | The standard allows one per section. Almost nobody will fill it in by hand, but leaving the space costs nothing now and a migration later | S | **Yes**, as an empty optional field | |

### Who is in charge when things change

| # | What | Why | Size | Recommend | Ceri |
| --- | --- | --- | --- | --- | --- |
| 9 | **A successor** — somebody the holder names in advance who may move the circle if she cannot | The hardest question in [hard-questions.md](hard-questions.md#5-what-happens-when-the-holder-disappears): today a circle whose holder dies or loses capacity is stuck read-only for good. Moving is built; *who may press the button* is not | M | **Yes** — the space for it, even if the screens come later. The safeguards need your decisions: see below | |
| 10 | **The outer ring** — a pass that lets somebody read without joining | For professionals seeing somebody once, and the answer to "why does everyone see everything?". It needs a new way through the door, which is in the frozen file. See [outer-ring.md](outer-ring.md) | XL | **Decide deliberately.** Its design is not finished. In: bigger batch, later release. Out: another migration when it is ready | |
| 11 | **History that travels with a moved circle** | Readings and suggestions come across a move as words kept on each device. An entry for them would put them in the circle itself, signed by the holder | S | **Later** — words work, and nobody has asked for more | |
| 12 | **Short codes** — a door address of 16 characters | Your number for anything typed. Needs a way to look a short code up, which does not exist yet | M | **Later** — QR codes cover most of it now | |

## Decided, 19 September 2026

**Ceri agreed every recommendation above**: items 1–9 are in, 11 and 12 are
later. Item 10, the outer ring, is still to be decided.

**Item 9, the successor:**

- **Anybody in the circle may be named.**
- **A waiting period** before a successor can move the circle, visible to the
  whole circle, during which the holder can say "I'm still here" and stop it.
- **The holder can change or remove the successor at any time.**
- **Ceri's addition: somebody who can check in person.** When a successor
  starts, somebody in the circle who lives near the holder, or can phone her,
  is asked to check and confirm. The danger this answers is the real one —
  somebody claiming the holder has gone when she is away, in hospital for a
  week, or has simply not opened the app.

How that last part might work, not yet decided:

- **(a) Any other member** confirms — the first to answer.
- **(b) The holder names a checker in advance**, somebody close who can visit
  or call, as she names a successor.
- **(c) Both:** the named checker is asked first, and anybody else may confirm
  if the checker cannot.

Whichever it is, the checker can never be the successor, and a confirmation
is written in the circle where everybody can see who gave it and when. It is a
person's word, like everything else here; what makes it a safeguard is that
it is a second person, and that it is visible.

## What has to come with it

Not changes to the frozen file, but nothing can be released without them:

- **Carrying circles across versions.** Circle-moving carries people to a new
  circle of the *same* version. Upgrading means the app holds the old version
  and the new side by side for a while, reads from the old and moves into the
  new. Not built.
- **The tests**, extended for every new rule, and new fingerprints written into
  `FROZEN.sha256` in the same change.
- **The [DPIA](DPIA.md)** updated for photos, voice, video and encryption.
- **Two-machine testing** before release, as on 14 and 19 September.

## Suggested order, once decided

1. Items 1–3 and 8: small, and they make the rest safer.
2. Item 4, then 5: the everyday removal.
3. Item 7: media entry types, then photos.
4. Item 9, once its questions are answered.
5. Item 6: its design note first, then the work — the largest.
6. Carrying circles across versions, the tests, the DPIA, and two machines.
7. Release.

Item 10 slots in wherever you decide.
