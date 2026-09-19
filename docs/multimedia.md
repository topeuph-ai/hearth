# Photos, sound and video in the record

**Planned 19 September 2026. Not built.** For people who find reading hard,
or whose eyesight is poor, and for people who express themselves better on
video than in writing — which the standard itself singles out.

## What the rules ask for

Checked against the texts themselves.

### The About Me standard wants it

The About Me Implementation Guidance (v1.0, PRSB, October 2020):

> "Ideally this information is also available in a multimedia format e.g.
> video, particularly when a person has difficulties expressing themselves."

Every one of the seven sections may carry a multimedia item — file name, file
type, the file, and a link (see [standard-and-gap.md](standard-and-gap.md)).
Media is the larger half of what separates Hearth from the full standard.

What the guidance asks of an implementation that has it:

- **Pass files on whole** — "transferred in their entirety". No lossy
  re-compression when a file is shared.
- **Keep videos short** — prompt people to consider "when multimedia is
  effective and ensure that videos are kept short".
- **Care where it plays** — video and sound "could result in confidentiality
  breaches if accessed in busy workplaces". Nothing plays by itself.
- **Keep it current** — media "can get out-of-date", so when one section
  changes, prompt a check of the others.
- **Say when and by whom** — already true of every version in Hearth.

The standard moved from PRSB to **NHS England on 1 January 2026**. Questions
about conformance go there.

### The Accessible Information Standard (DAPB1605, updated mid-2025)

It binds NHS and publicly funded adult social care **providers and
commissioners**: identify, record, flag, share, meet and review people's
communication needs, in formats including audio, video, easy read, braille and
BSL. It reaches software through contracts — providers "must specify the
requirement to comply with the standard in IT system and software provider
contracts".

Hearth is not bound by it directly. Its fit is obvious: the person's own "How
to talk with me", in their own voice or on video, is exactly what a provider is
required to act on.

### DTAC, the NHS's criteria for buying digital tools (from 6 April 2026)

Requires **WCAG 2.2 AA** and that suppliers **show they have considered the
Accessible Information Standard**. For media, WCAG asks for captions on
recorded video and a text alternative for recorded sound.

Families recording a video of their mother will not caption it, and must not
be made to. So every video and sound clip gets an optional **"What this says
in words"** box — suggested, never required — which is the text alternative
WCAG looks for, written by the person who knows what the clip says.

### Data protection

A face, a voice or a video identifies somebody far more than a paragraph does,
and anything showing a person's health is special category data. Other people
turn up in them — grandchildren in a photo, a daughter's voice in a clip — and
it is their data too. **The [DPIA](DPIA.md) must be updated before media goes
in**, and encryption of the record becomes more pressing, not less.

## Decided (Ceri, 19 September 2026)

| Question | Decision |
| --- | --- |
| Video quality | **480p.** Decent even on a large screen, more than enough on a phone or laptop. HD as an option in the finished product |
| Longest video or sound clip | **Two minutes** |
| "What this says in words" beside a video or clip | **Suggested, never required** |
| Where the files live | **In the circle**, so every member holds a copy — for now |
| Order | **Easiest first:** photos, then sound, then video |

## How it would be built

### All three need the frozen integrity zome to change

Nothing that exists today can hold a picture: the record is text, and a text
field is the wrong place for a file. So media is a **new kind of entry**, which
changes the rules a circle is built from — the migration batch in
[to-a-product.md](to-a-product.md).

Because each change to that file costs every circle a move, **the entry types
are designed for all three at once**, even though the screens are built photos
first. Designing for photos alone and adding video later would mean two
migrations.

### Sizes, and why video is the hard one

Holochain's limit is **4 MB per entry** (`ENTRY_SIZE_LIMIT` in
`holochain_integrity_types` 0.7.0), and every member stores every piece.

| Kind | Two minutes, or one picture | Fits in one entry? |
| --- | --- | --- |
| Photo, shrunk to ~1600 px | a few hundred KB | Yes |
| Sound, speech-quality | well under 1 MB | Yes |
| Video, 480p | roughly 10–20 MB | **No — split into pieces** |

These are rough figures and must be measured. The shape that follows from
them:

- **A media item** — which section it belongs to, the file type, its length,
  the "in words" text, and the list of its pieces by hash, so every peer can
  check that exactly those pieces make it up.
- **Pieces** of at most about 3 MB, each checked by every peer for size.
- **Limits checked by every peer**, not only by the app: a maximum number of
  pieces, and the two-minute length as far as it can be checked.

### Shrinking happens on the device that records it

Photos are resized, sound is recorded at speech quality, and video at 480p,
**before** anything is written — the only way to keep every member's device
from filling up. "Transferred in their entirety" is then about the file as the
person saved it: nobody re-compresses it on the way to anybody else.

### In the order decided

1. **Photos.** Choose a picture, see it shrunk, add a sentence of "in words" if
   you like. Shown beside its section.
2. **Sound.** Record from the microphone or choose a file; a play button and a
   length; never plays by itself.
3. **Video.** Record or choose, shrunk to 480p and two minutes; split into
   pieces; put back together to play; never plays by itself.

## What has to come first

- **The migration batch has to start**, because media cannot be built without
  it. The batch also carries marking as gone, encryption with a new key on
  removal, deleting a removed person's copy, coded values, the unchecked
  validation arms, the rule-not-person change and signing the name on an
  invitation.
- **An upgrade path across versions.** Circle-moving carries people to a new
  circle of the *same* version. Moving onto a *new* version needs the app to
  hold both versions side by side for a while — the old to read from, the new
  to move into. Not built. Planned with the batch.
- **The DPIA updated** for photos, voice and video.

## Sources

- [About Me Implementation Guidance v1.0](https://theprsb.org/wp-content/uploads/2020/10/About-Me-Implementation-Guidance-Version1.0.pdf), PRSB, October 2020 — sections 2.2, 2.5 and the About Me section notes
- [About Me](https://standards.nhs.uk/published-standards/about-me), NHS Standards Directory
- [Accessible Information Standard – requirements (DAPB1605)](https://www.england.nhs.uk/long-read/accessible-information-standard-requirements-dapb1605/), NHS England
- [New NHS Digital Technology Assessment Criteria](https://www.burges-salmon.com/articles/102mnjh/new-nhs-digital-technology-assessment-criteria-what-health-tech-suppliers-need-t/), Burges Salmon — a secondary source; read the DTAC form itself before relying on it
