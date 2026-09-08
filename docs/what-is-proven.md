# What is proven, what is not, and what we would want help with

**Status: the honest inventory.** Written 2026-09-08. It exists so that nobody
has to guess which parts of this project are load-bearing and which are hopeful.
Where something is untested, it says so and says why.

If any claim elsewhere in this repository contradicts this page, this page is
the one that is right, and the other one is a bug.

---

## Who is building this, and with what

One person, not a software engineer. My field is music. I have no computing
background, I do not write Rust, and I could not have built this alone.

**The code is written with AI assistance, and I have made the design decisions.**
That division is the honest one, and it is worth stating plainly rather than
leaving to be discovered. What it means in practice:

- **The decisions I can defend** are the ones about shape. Why there is no
  operator. Why the record is scoped to About Me and stops hard before anything
  clinical. Why a membrane cannot be edited after a circle is made, and what
  that costs. Why acknowledgement is a record of reading rather than a
  permission. These are choices, they have reasons, and the reasons are in the
  commit messages and the comments, which are unusually long on purpose.
- **The decisions I am taking on trust** are the ones about implementation.
  Whether the Rust is idiomatic. Whether the cryptography is used correctly in
  every path rather than only the ones with tests. Whether there is a class of
  bug here that somebody who has done this for ten years would spot in an hour.

**The second list is exactly where outside review is worth most**, and it is the
main thing this project would want from anyone reading it.

There is a related point about the process. A large share of the real bugs found
in this project were found by walking the interface as a user and noticing that
something made no sense — not by tests, and not by reading code. Several are
recorded in the commit history. That is offered as evidence that the walking is
being done, not as a substitute for the review above.

---

## Proven

### The rules hold up against somebody trying to break them

**45 tests, run in CI on every push**, in
[`tests/tests/adversarial.rs`](../tests/tests/adversarial.rs). They are written
as attacks rather than as feature checks. Among them:

- An uninvited agent cannot join a circle.
- An invitation cannot be passed on to somebody it was not made for.
- A member cannot forge an invitation.
- A member cannot write the person's record, cannot hijack its update chain,
  and cannot delete it.
- Nobody can acknowledge their own record.
- A circle with no founder, or with a malformed one, admits nobody — a broken
  configuration closes the door rather than opening it.
- Nobody can create a circle in another person's name.
- Two holders' circles are genuinely different networks.
- With a second yes configured: half an invitation opens nothing; the holder
  cannot give the second agreement herself; a circle with no second yes is
  unaffected.

These run against a real conductor, not a mock. **What they prove is that the
rules are enforced. What they cannot prove is that the rules are the right
rules** — that is a question for a reviewer, not a test.

### The whole journey works on one machine

`the_whole_journey` walks the real sequence end to end. Beyond that, the
interface has been walked repeatedly by hand with two and three separate
conductors running side by side, each in its own window, on this machine.
Invitations, joining, writing, suggestions, acknowledgement, leaving and
rejoining all work.

### It conforms to the standard it claims

Checked field by field against the PRSB About Me JSON. See
[`standard-and-gap.md`](standard-and-gap.md).

---

## Built, but not proven

### Two machines have never run this

**This is the most important line on the page.**

The app is built to find its peers over the internet, and every piece it needs
is there: it ships with Holochain inside it, it points at a bootstrap server
that does rendezvous only and holds no data, and the flags that make it work
across machines have been found and tested as far as they can be tested from a
single computer.

**But no second machine has ever run a node. The reason is money, not
confidence.** There is one Windows machine here. The other laptop is a
Chromebook, which cannot run it. Renting a Windows host for an afternoon, or
buying a second machine, is not a technical problem — it is simply not free, and
this project is funded by nobody.

So the claim "install it on two machines and they find each other" is a claim
about the design and about what the code contains. **It is not a claim anybody
has watched happen.** It is the single thing most worth a reviewer's time,
because it is cheap for anybody with two computers and impossible here.

What *has* been established from one machine: multiple conductors on one host
find each other and sync; the bootstrap server is reachable and is not a data
path; and the specific failure caused by antivirus HTTPS interception has been
diagnosed and documented, because it happened here and cost a day.

### No release has ever been published

The installer builds from source, and has been built and installed on this
machine. Nobody has ever downloaded one. Until there is a release, "try it
yourself" is an invitation nobody can accept.

### Joining a circle can take about ninety seconds

Measured here, understood, and written up in [`latency.md`](latency.md). Not
fixed. It is a rendezvous cost rather than a bug, but it is a long time to sit
in front of a screen wondering whether something has gone wrong.

---

## Not built

### The record is not encrypted

**The largest gap, stated first because it is the one a reviewer should ask
about.**

Entries are validated, signed, and only reachable by people admitted through the
membrane, so a circle's contents do not leak to the network at large. But the
contents themselves are stored in the clear on every member's machine. Once
somebody is in a circle — and afterwards, if they keep their copy — they have
the plain text.

For the About Me scope that is a smaller exposure than it sounds: no
medications, no diagnoses, nothing clinical, by design. It is still the thing
standing between this and a record anybody should trust with more.

### Nothing is revocable in the ordinary sense

Leaving takes the circle off your device. It does not reach anybody else's copy,
and the interface says so rather than pretending otherwise. Changing who may
enter means forming a new circle and everybody rejoining. This is honest, and it
is a consequence of having no operator, but it is not what most people expect
the word "remove" to mean.

### There is no way for a stranger to discover the record exists

A paramedic who has never heard of this cannot find it. Paper solved that with a
sticker on a fridge. See [`what-to-borrow.md`](what-to-borrow.md). Nothing is
built, and a perfect record that is never read is the failure that matters most.

### The invitation is a wall of characters

Long, awkward to send, and it carries the name of the person the circle is about
in the clear. Assessed in [`DPIA.md`](DPIA.md). It is shaped that way because
there is no server to hold accounts — the invitation has to carry the grant
itself. That is the operator trade made concrete, and it needs work rather than
a wish.

### Nothing has been through a security review

No audit, no external review, no penetration testing of any kind. The tests
above were written by the same process that wrote the code, which is exactly the
limitation you would expect it to have.

---

## What we would want help with, in order

1. **Two machines.** Somebody with two computers running the installer and
   saying what happened. The cheapest possible thing to do, and the largest
   answer it is possible to get.
2. **A read of the cryptography and the validation callbacks** by somebody who
   has written Holochain before, looking for the class of mistake that tests
   written by the author will never find.
3. **A view on encrypting entry contents** that does not quietly reintroduce a
   party holding a key on everybody's behalf.
4. **Whether the design is right at all** — the membrane model, the second yes,
   acknowledgement-as-record. These are defensible, but they are one person's
   answers and they have not yet been argued with by anybody who would know.

---

## What this project is not claiming

- **Not a clinical record**, and not one that will become one by accident. The
  About Me boundary keeps clinical safety certification, clinician liability and
  the heaviest data protection questions out of scope. That boundary is
  load-bearing, not a limitation to be grown out of.
- **Not production software.** Not deployed anywhere. Nobody's real record is in
  it.
- **Not a finished answer** to decentralised health data. It is one narrow,
  honest slice, built far enough to be argued with.
