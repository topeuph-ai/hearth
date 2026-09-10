# How it works, and why it is shaped this way

The design decisions, and what they cost. For what is actually tested see
[what-is-proven.md](what-is-proven.md); for what to build next see
[to-a-product.md](to-a-product.md).

---

## The membrane: who gets into a circle

A circle is a private network built around **the public key of whoever holds
it**. That key is part of what the circle *is*, not a setting stored inside it.

So a different holder produces a completely different network. **Circles cannot
see each other, and that is a fact about the mathematics rather than about
anybody's list of permissions.** It is also why one circle per person is
affordable: a circle is a clone, and clones are nearly free.

**An invitation is the holder's signature over the invitee's public key.** You
present it at the door, and every existing member checks it themselves against
the holder's key, which they already have because it is built into the circle.

Nobody is asked for permission, because there is nobody to ask.

Signing over *your own* key is what stops an invitation being passed on to
somebody else. It admits exactly one person and it is useless to anybody else,
which is why it is safe to send by text message or read down the phone.

It is checked in two places:

- **Before you join**, on your own machine, so a bad invitation fails
  immediately with a readable reason instead of being quietly refused later by
  strangers.
- **By the network**, which is the enforcement that actually matters. Every peer
  makes the same check independently.

The function that issues invitations has **no permission check on it at all**,
deliberately. Anybody may call it, and a non-holder's signature simply will not
verify. **There is nowhere to enforce a permission, because there is no server.**
That is the whole architecture in one function.

### Configuration that fails closed

Reading a circle's identity gives one of exactly four answers, never a "maybe":

- **A real circle**, closed around one person.
- **A lobby** — anybody may join, **nobody may write.** The app ships with one,
  and its only job is to exist so the app is installable and can make real
  circles out of itself. Everybody who installs the app shares it, so it must
  hold nothing: entry is unrestricted precisely because there is nothing there
  to reach. It has to be asked for in writing.
- **A waiting room** — anybody may join and knock, and only the holder of the
  circle it serves may answer. See [The waiting room](#the-waiting-room).
- **Misconfigured** — no settings, unreadable settings, or a holder that is not
  a valid key. **Admits nobody and lets nobody write.**

That this is an enum with no catch-all is not tidiness. When the waiting room
was added, the compiler produced three errors rather than three silently wrong
answers — who may join, who may speak as the person, and who may give a second
agreement. A fourth state that defaulted to something would have been a fourth
state nobody checked.

An earlier version treated a missing holder as an *open* circle. So a typo, a
missing setting or a botched clone would have produced a wide-open circle around
a vulnerable person, silently.

Those same mistakes now produce a circle nobody can enter. Visibly broken rather
than invisibly exposed.

**Absence of configuration must never mean absence of a membrane.** Both failure
cases have tests.

### The optional second yes

A circle may ask for a second person's agreement before anybody joins. Most
circles will never use it.

The case it exists for is not a stranger breaking in. It is somebody being
**talked into** letting a person in — a plausible caller, a new "friend", a
relative nobody trusts. The holder is the person under that pressure, so a rule
the holder can switch off alone would be no protection at all.

**Be precise about what it is.** The holder can always make a *new* circle
without one — she is not locked in. What she cannot do is drop it **quietly**.

It is a tripwire, not a lock. The stronger claim is the tempting one and it is
false.

#### The rule is in the circle's identity; the person is not

This distinction is the whole design, and it was arrived at the hard way.

The second person's *key* used to be part of the circle's identity, which is
immutable. Three things followed, and all three were bad:

- Her identifier had to be collected **before the circle could exist**, putting
  the hardest step in the app right at the beginning.
- Changing who agrees meant a new circle, the record copied across, and every
  member joining again.
- So did the situation every circle will eventually meet: **the second person
  dies, or loses the device their keys were on.** For a record about somebody
  in declining health that is the expected course of events, not an edge case.

So the identity carries only the rule — *two people must agree* — and who the
second person is, is an entry the holder writes and can write again. Appointing
somebody is one line written in the circle. Nobody re-joins anything.

#### What that costs, stated plainly

When the person was named in the identity, the network **refused** to admit
anybody without their signature.

It cannot now. "Has she appointed anybody yet?" is a question whose answer
changes, and validation has to give the same answer on every machine forever.
So an invitation that names no appointment is accepted.

**The safeguard is therefore enforced by being visible, not by being
impossible.** Every admission records whether it was seconded, and under which
appointment, where the whole circle can see it.

That is a real reduction from what came before, and it is deliberate — because
it is also exactly what this project has always claimed the second yes to be.
She could always have made a circle without one. What she cannot do is do it
in secret.

#### How it stays checkable at all

Everything that must be verified **names the appointment it relies on, by
hash** — the invitation and the agreement both.

A fixed hash is a question with one answer on every machine forever. "Who is
appointed now" is not, and never can be. So the door checks a signature against
the appointment that was in force when the agreement was given, rather than
against whoever holds the job by the time somebody arrives.

One consequence worth knowing: **an agreement already given stays good.**
Replacing the second person does not invalidate invitations they had already
agreed to. An agreement is something somebody did at a moment, and moments do
not become undone. What changes is that they cannot give another.

There is a second consequence, in `genesis_self_check`. That callback runs on
the joiner's own machine before they have joined anything, so it has no network
and cannot fetch an appointment. The membrane check is therefore told how far
it may look: locally it catches an unfinished invitation and says so readably,
and the network does the rest. That split is what the two callbacks are for.

### The two agreements find each other

Both agreements are signatures over the same key. They used to reach each other
**by hand**: the holder made half an invitation and sent it to the second
person, who agreed and sent it back, who sent it on. Five copy-and-pastes of
near-identical base64 between two devices that were already members of the same
circle and perfectly able to talk to each other.

It was as bad as that sounds. Walking it, a finished invitation went into the
box marked "Their identifier", because that was the first box on the screen
that wanted a long line of characters. That is not a mistake anybody should be
given the chance to make about who may read a vulnerable person's record.

So the agreements travel in the circle instead:

1. The holder writes down who she wants to let in, and what she calls them.
2. It appears in the second person's own copy of the circle, with a button.
3. They agree. Their signature is written beside the proposal.
4. The finished invitation appears on the holder's screen, assembled from the
   two agreements, ready to send.

**None of this changed the membrane.** Whoever joins still presents both
signatures at the door and every peer still checks both. Somebody who has not
joined cannot read anything inside, so the invitation still has to carry its
own proof. What changed is only how the two signatures find one another.

Two rules, both checked by every peer:

- Only the holder may put somebody forward, **and** the proposal must carry her
  real signature over the key it names — so a proposal cannot promise something
  that would fail at the door later, silently.
- **Only the person the circle has appointed may give the second agreement.**
  If any member could agree, the second yes would be a second yes from whoever
  happened to be about.

## The waiting room

Joining used to begin with *"send me the long line of characters from your
app"*. That is the step where this stopped being possible for somebody elderly,
or somebody being helped — an invitation is signed over a key, so the key had
to be collected first, one person at a time, by hand.

A waiting room turns it round. **The holder shares one address**, which never
changes and works for everybody: a family group, a phone call, a note on the
fridge. Whoever has it can knock — say who they are and ask. They bring their
own key with them by arriving.

### Why it has to be a separate network

A circle is closed. Somebody outside it **cannot write to it** — that is the
membrane doing its job, and no amount of interface removes it. So the room has
to be somewhere they *can* write.

The app already had the shape of this. The lobby — anybody may join, nobody may
write — exists so the app is installable at all. A waiting room is a lobby that
names the circle it serves and permits exactly two things: a knock, and an
answer to one.

### What happens

1. Somebody knocks: a name, how they say they are connected, and nothing else.
2. The holder sees them, with a caution that nothing has been checked.
3. She lets them in — or does not.
4. Her app leaves their invitation in the room. Their app collects it and opens
   the circle.

Where the circle asks two people, step 3 puts them forward instead, and the
invitation is left once the second agreement arrives.

**One message still leaves**, to the person joining, because they are outside
the circle by definition. That is the door, and it is the whole point of there
being one.

### Why the answer can be left lying in an open room

An invitation is signed over one person's own key. It admits nobody else and is
a useless blob to anybody who picks it up — the code has said so since the
beginning, and this relies on it being true rather than hoping.

So the answer does not have to be carried by hand to the one person it is for.
It can be left where they will find it.

### The rules, all checked by every peer

- **Anybody may knock, and only for themselves.** No check on who is asking,
  deliberately: a room where you must already be known in order to ask is the
  closed door this replaces.
- **Knocks mean nothing outside a room built for one.** Without this they would
  be writable in the plain lobby every installation shares, which would put
  "somebody wants to join Margaret Smythe's circle" in front of every person
  who ever installs this app.
- **Only the holder may answer.** A forged answer could admit nobody, since the
  circle's own door still checks the signature — but it would let a stranger
  hand somebody a thing that looks like a welcome and silently is not.
- A room naming a holder it cannot read is closed, exactly as a circle with a
  mistyped founder is.

### What it does not do

It does not tell anybody who is really knocking. A name and a relationship are
claims, exactly like every other name in this app, and the screen says so.
What would actually help — a shared secret sent by a different channel — is
written up in [`to-a-product.md`](to-a-product.md) and is not built.

"Not now" writes nothing anywhere. A refusal recorded in an open room would be
a public snub, readable by everybody with the address including the person
refused. Nothing is owed to somebody who knocked uninvited, and silence is the
kindest available answer as well as the safest.

**One limit worth knowing.** Which room belongs to which circle is remembered
on the holder's own device, because a circle cannot hold a reference to a
network its members may not be in. A holder who moves to a new device keeps the
circle and loses the door, and would have to open a new one.

## Two boundaries, not one

An outside red-team review found the central mistake in the first version:
**"closed circle" had been built far more strongly than "person-owned record".**
Those are different boundaries.

Who may **enter** was enforced. Who may **write** was not enforced at all — any
admitted member could author an About Me and publish it, so a circle could hold
several competing accounts of the same person, each honestly maintained by
whoever invented it.

**And there was a worse one, which that review missed.** The validation function
ended in a catch-all that said "fine" to everything it had not been told about
— which included all link creation and all deletion.

That allowed a sharper attack. A member could draw a link from the person's own
record to an entry of their own, and any reader following the chain of revisions
would then be shown **the impostor's words as the person's current record** —
without ever touching the person's entry, and so without tripping the rule about
who may revise it. The same hole let any member delete the person's record
outright.

Both are closed. Links carry meaning in this design, so they have rules of their
own.

> **The lesson worth keeping: a validation function that ends in a permissive
> catch-all is a security hole with a comment on it.** Enumerate what you allow.

### The run that mattered was the one before green

When those rules first went in, four tests passed and five failed — and among
the failures was *the person can write their own About Me*.

Nobody could write. Which meant *a member cannot write the person's record* had
been passing because **nobody at all** could write. A green test proving
nothing.

The cause was the new link rule. It demanded that every link into the circle's
index point at an About Me, and the anchor machinery builds its own scaffolding
using links of that same type, pointing at its own internal entries. The
security fix had broken every write in the application.

**Reading the code would not have found that. Running it did.**

---

## Revocation

There are three different things people mean by this word, and running them
together produces software that lies.

| | |
|---|---|
| **Membership revocation** | Stop somebody writing to the circle in future |
| **Access revocation** | Stop somebody reading what they already hold |
| **Erasure** | Remove data from their device |

**Validation cannot enforce the first one.** To ask "has the holder removed this
person?", a checker would need the holder's *current* position in their own
record of what they have done — which changes constantly and looks different to
everybody. Checks have to give the same answer everywhere and at any time, so
they cannot see it. Letting the writer state it themselves does not help: a
removed member simply quotes an older position.

**Capability grants do not fix this.** They control who may call into your own
device. They say nothing about what somebody already holds, or about what the
network will hand them. Listing them as the answer to revocation was a mistake
in an earlier version of this document.

**The sound answer falls out of the architecture: revocation is re-forming the
circle.** A circle is a clone and clones are nearly free. To remove somebody,
make a new circle and invite everybody except them. They are excluded by
mathematics, not by a rule somebody has to remember to enforce.

What that does **not** do — and nothing can — is get back what they already
have. Once a person has legitimately received something readable, it is theirs.

**Access revocation and erasure are not achievable against somebody who has
already read the data**, on this architecture or any other. The
[DPIA](DPIA.md) says so rather than implying otherwise.

Best-effort removal within an existing circle — the holder records a departure,
everybody else's app stops showing that person and stops sharing new material
with them — is worth building for the ordinary case of a professional moving on.
**It is a courtesy, not a control, and must never be described as one.**

---

## Offline is not a failure state

Everybody being offline usually means everybody is busy living, not that
anything is broken. And if nobody is online, nobody is trying to read it either.
The worry largely cancels itself out.

This is easy to lose by accident, because every interface convention we have
inherited comes from cloud software, where offline genuinely does mean broken.
Those defaults are all anxiety: reconnecting banners, sync spinners, staleness
warnings, notifications nagging you back.

**None of it applies here, and none of it should be built.** A member holds a
complete copy. When they open the app it is simply there. There is nothing to
reconnect to.

For these users this is not a nicety. A carer who is already exhausted does not
need software implying she has fallen behind on something.

> **Rule: no part of the interface may suggest that being away is a problem.**
> No sync status, no "you are offline" bar, no last-updated warnings.
> Information that arrived while somebody was away is shown as new, never as a
> backlog they are late on.

There is a related restraint about counting. A district nurse could be in thirty
circles. The list of them must read as a phone book of people she visits, never
as an inbox — no unread counts, no badges, nothing implying she owes anybody a
reply. She is a reader of thirty short documents, not a participant in thirty
conversations. Getting that wrong is how this becomes another thing nobody
opens.

---

## Availability, which turned out better than expected

An earlier draft treated this as the question that decided whether the project
was viable at all. That was wrong, and the correction is worth keeping.

**Every member of a circle holds a complete copy of it.** In Holochain 0.7.0 an
agent joins holding everything, and the configuration documents that as normal
operation. Six members means six complete copies.

So there is no redundancy problem — no situation where data is thinly spread or
partly lost. What is left is a much narrower claim: **if nobody is online,
nobody answers.** True of any peer-to-peer system.

And it is narrower still, because reading is local. A member already holds the
whole thing: they read from their own device, needing no network and nobody else
awake. Writing is local too. **An existing member is never blocked by anybody
else being offline.**

The only case needing a live peer is **joining**. A newly invited member, or an
existing member on a replacement phone, has no copy yet and somebody has to hand
them one. The realistic worst case is a professional invited into a circle at
3am who cannot receive anything because nobody else is reachable.

The mitigation is one always-on device per circle — a home laptop, or a tablet
left on charge — whose job is specifically to make joining possible at any hour.
Note that such a device holds a full readable copy like any other member, which
is one more reason contents should eventually be encrypted.

**Sharding is not the answer and is not needed.** It is planned for a later
release and exists for large networks where holding everything becomes a burden.
At the scale of a family circle everybody holds everything regardless. **This
works on 0.7.0 as shipped, with no dependency on an unreleased feature** — which
is a far stronger position for a funding application than "viable in a future
release".

A design note for later: a node can take part without storing anything. That is
wrong for a device at home and right for a phone — hold everything on the
machine in the kitchen, hold nothing on the phone in a pocket.

---

## Mobile

Not a blocker any more, but not free either.

**iOS.** Holochain 0.7.0 added an interpreted backend that complies with Apple's
ban on downloading executable code, so the App Store barrier is gone — they have
demonstrated a Holochain app on an iPhone. What remains: the interpreter is
slower (irrelevant at this size), a keystore bug fixed in 0.7.1, and no
ready-made packaging template.

**Android.** Proven since 0.3 — Volla ship a phone with a Holochain app on it —
and there is now a first-party route, see
[building-it.md](building-it.md#the-surrounding-ecosystem-as-of-september-2026).

The blocker moved from physics to packaging. The desktop demo needs neither.

**One open question for Holochain rather than for us:** thirty to fifty circles
in one conductor, each its own network with its own gossip. Nobody knows whether
that is comfortable on a mid-range Android phone or whether it melts. That
decides feasibility, not polish.
