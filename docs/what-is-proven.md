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

**67 tests, run in CI on every push**, in
[`tests/tests/adversarial.rs`](../tests/tests/adversarial.rs), plus 3 unit tests
on the ordering rule. They are written as attacks rather than as feature checks.
Among them:

- An uninvited agent cannot join a circle.
- An invitation cannot be passed on to somebody it was not made for.
- A member cannot forge an invitation.
- A member cannot write the person's record, cannot hijack its update chain,
  and cannot delete it.
- Nobody can acknowledge their own record, and an acknowledgement cannot be
  attached to something that is not a record.
- A circle with no founder, with a malformed one, or naming a second person it
  cannot read, admits nobody — a broken configuration closes the door rather
  than opening it.
- Nobody can create a circle in another person's name.
- Two holders' circles are genuinely different networks.
- With a second yes configured: half an invitation opens nothing; the holder
  cannot give the second agreement herself; a circle with no second yes is
  unaffected.
- **A signal that claims to be from somebody else reaches no screen**, and one
  that names its real sender still does.
- **Correcting yourself is the version that shows** — a corrected introduction,
  and a changed decision about a suggestion, both come back as the latest rather
  than as whichever arrived first.
- **Being asked to agree is a question somebody answers.** Only the person an
  appointment names may answer it; an unanswered asking reads as neither a yes
  nor a no; and answering again replaces the answer without erasing it.
- **A knock says nothing to the rest of the room.** Somebody else standing in
  the same waiting room sees that a knock happened and which key wrote it, and
  cannot read the name or the relationship. The holder can, and so can the
  person who wrote it.

These run against a real conductor, not a mock. **What they prove is that the
rules are enforced. What they cannot prove is that the rules are the right
rules** — that is a question for a reviewer, not a test.

And there is a second thing they cannot prove, which this week made plain.
Every fault found by walking the demo — a button that did nothing, a screen
that said nothing had happened when it had, a knock that had to be agreed to
twice — passed every test in this file while it was happening. The rules were
enforced correctly throughout. The tests are about the boundary; somebody
sitting in front of it is about everything else.

There is one honest limitation in how much of this can be tested at all.
Everything is written through the app's own functions, and several of the link
rules cannot be reached that way — the function that introduces you always links
to your own introduction, so "you may only introduce yourself" has nothing to
attack it with, and nothing deletes a link, so the rule about who may is
unreachable too. Those rules are correct as read, and untested because there is
no legitimate route to breaking them. Reaching them would mean shipping
attack tools inside the app, which is a worse trade.

**A note for anybody running these on Windows: you cannot.** The test crate
pulls in a keystore that builds OpenSSL from source, which needs a full Perl
toolchain that Git for Windows does not ship. CI runs them on Linux on every
push, and that is where they are verified. It is why the CI badge at the top of
the README matters more here than it would elsewhere.

### The whole journey works on one machine

`the_whole_journey` walks the real sequence end to end. Beyond that, the
interface has been walked repeatedly by hand with two and three separate
conductors running side by side, each in its own window, on this machine.
Invitations, joining, writing, suggestions, acknowledgement, leaving and
rejoining all work.

### Two machines, and then no internet

**14 September 2026, the first time Hearth ran anywhere but one computer.**
Two Windows machines — a desktop PC and a laptop — each with the released
0.2.2 installer, on the same home wifi.

**Online:** Margaret's circle was made on the PC. Dave, on the laptop, knocked
at its door address, was let in, offered a suggestion, and the holder accepted
it on the other machine.

**Then with the internet taken away:** the broadband cable into the router was
unplugged, leaving both machines on the local network and neither able to reach
anything beyond it. "What matters most to me" was changed on the holder's PC,
and the change appeared on Dave's laptop.

That is the thing a hosted service cannot do. Its devices only know how to
talk to its server; with the internet gone, the people in the room stop
sharing. Here they did not.

**⚠️ But only while it was already running.** The same afternoon, both apps
were restarted with the broadband still unplugged. Both opened, and they never
found each other — left ten minutes, twice, with no error on either screen.

Why, read in the Holochain 0.7.0 source rather than guessed:

- **The list of where other devices are is kept in memory only.** Holochain
  builds its networking on kitsune2's default setup, which uses
  `MemPeerStoreFactory`. A restart forgets every peer, and the only way to
  learn them again is to ask the bootstrap server — which is on the internet.
- **Saving that list would not be enough.** Each entry is signed to expire
  twenty minutes after it is made, and an expired one is refused. And a
  device's address is *reach me through this relay server*, a server on the
  internet; a direct path is only found once two devices are already talking.
  That is why the connection made before the cable came out kept working, and
  a fresh one after a restart could not be made.
- **Stock Holochain 0.7 cannot look for devices on the local network.** The
  transport's own documentation says there is no discovery service.

So the accurate claim for Hearth as released is: **it keeps sharing through an
internet cut while it is running. After a restart with no internet, devices
cannot find each other.**

### ✅ Restarting with no internet: solved in a field test, 19 September 2026

The same two Windows machines, each running **"Hearth LAN test"** — Hearth
built with the Lightningrod Labs field-test Holochain,
[holochain-0.7.0-mdns.2](https://github.com/lightningrodlabs/holochain/releases/tag/holochain-0.7.0-mdns.2),
with its local-network discovery switched on
([download](https://github.com/topeuph-ai/hearth/releases/tag/lan-field-test-1),
marked not for general use).

- **The laptop joined by scanning the QR code** on the desktop's screen with
  its camera — the first use of the scanner, and nothing was typed.
- Both apps were **quit fully, the broadband was unplugged, and both were
  started again.** They found each other in **about a minute**, and a change
  made on one arrived on the other.
- **They found each other faster offline than online.** Not measured, and not
  explained from the source; the likely reason is that on the local network
  they connect directly, where online the first contact goes through the
  bootstrap and relay servers.
- **Norton 360 asked about nothing** on the desktop during the test. It had
  flagged the installer itself as new and rarely seen, which is a reputation
  warning about any unsigned build, not a finding.

Getting there took three attempts, and the first two tested nothing: the test
app was not installed on one machine the first time, and the second time the
test app itself could not start, because of a packaging fault now written up
in [building-it.md](building-it.md). Recorded so the result is read as one
clean run, not three.

**What this does and does not change.** It shows the fix works on this
hardware, for Hearth, in a real house. It does **not** change what Hearth
ships: the field-test Holochain cannot talk to ordinary Holochain, and its
authors say it is not for general use. Hearth moves onto local discovery when
official Holochain has it — see
[Finding each other without the internet](to-a-product.md#finding-each-other-without-the-internet).

**A second fault, found in the same test and fixed.** Closing Hearth's window
only hides it — it keeps running beside the clock so it can keep sharing — and
opening it again started a second copy, which waited forever at "Starting lair
keystore" because the first copy was still holding the keys. The desktop app
now allows one copy per profile, and opening it again brings the window back.
Checked by starting the built app twice: the second copy closed itself and
nothing extra was left running. Not in a release yet.

What this does **not** yet show, so nobody reads more into it:

- **The two machines had already met online** before the internet went. Two
  devices meeting for the very first time with no internet at all is untried,
  even with the field-test build.
- **Both were on one local network.** Two machines on different networks,
  with no internet between them, cannot reach each other and nothing claims
  they can.
- **Two machines, not twenty.** Nothing here is evidence about scale.

### It conforms to the standard it claims

Checked field by field against the PRSB About Me JSON. See
[`standard-and-gap.md`](standard-and-gap.md).

---

## Built, but not proven

### Two machines across the internet, in different places

**Two machines on one home network now work, online and offline** — see
[above](#two-machines-and-then-no-internet). What has not been tried is two
machines in different buildings, on different networks, finding each other
across the internet through the bootstrap and relay servers. Every piece that
needs is in the build, and that is the next thing worth watching happen.

### Nobody outside the project has installed a release

Releases are published and downloadable. Nobody but the author is known to
have installed one.

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

### Two of the six checks Holochain offers are not switched on

Found in an audit on 2026-09-09, and stated here rather than left to be
discovered, because it is the sort of thing a reviewer finds in ten minutes.

When somebody writes something in a circle, Holochain does not ask one machine
whether it is allowed. It asks several, each looking at a different aspect of
the same act. This zome answers four of those questions and lets two through
unchecked — the one about the record as a filed document, and the one about the
running list of what an agent has done.

**No way to exploit it was found.** The two that *are* checked cover the
contents and the links, and every list this app reads is reached by following a
link, so a forged entry or a forged link is refused before anybody could see
it. But the reason to write it down is that the same shape of gap has already
happened here once: link creation was, at one point, entirely unchecked, and it
was found by reading the code rather than by any test.

It cannot be fixed now. The check lives in the frozen file, so changing it
means every existing circle becomes unreachable. It is queued in
[`to-a-product.md`](to-a-product.md) as the first thing to do whenever that
file next moves.

### Nothing has been through a full security review

No external review, no penetration testing of any kind. There has been one
audit — the one above came out of it — but it was carried out by the same kind
of process that wrote the code, which is exactly the limitation you would
expect it to have.

---

## What we would want help with, in order

1. **Two machines in different places.** Two on one home network work, online
   and offline. Somebody in another building installing the release and joining
   a circle across the internet is the next largest answer it is possible to
   get.
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
