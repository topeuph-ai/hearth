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

Reading a circle's identity gives one of exactly three answers, never a "maybe":

- **A real circle**, closed around one person.
- **A lobby** — anybody may join, **nobody may write.** The app ships with one,
  and its only job is to exist so the app is installable and can make real
  circles out of itself. Everybody who installs the app shares it, so it must
  hold nothing: entry is unrestricted precisely because there is nothing there
  to reach. It has to be asked for in writing.
- **Misconfigured** — no settings, unreadable settings, or a holder that is not
  a valid key. **Admits nobody and lets nobody write.**

An earlier version treated a missing holder as an *open* circle. So a typo, a
missing setting or a botched clone would have produced a wide-open circle around
a vulnerable person, silently.

Those same mistakes now produce a circle nobody can enter. Visibly broken rather
than invisibly exposed.

**Absence of configuration must never mean absence of a membrane.** Both failure
cases have tests.

### The optional second yes

A circle may name somebody whose agreement is *also* needed before anybody
joins. Most circles will never use it.

The case it exists for is not a stranger breaking in. It is somebody being
**talked into** letting a person in — a plausible caller, a new "friend", a
relative nobody trusts. The holder is the person under that pressure, so a rule
the holder can switch off alone would be no protection at all.

So it lives in the circle's identity, where every peer enforces it and nobody
can quietly turn it off.

**But be precise about what it is.** The holder can always make a *new* circle
without one — she is not locked in. What she cannot do is drop it **quietly**,
because a new circle means everybody has to join again and they would all
notice.

It is a tripwire, not a lock. The stronger claim is the tempting one and it is
false.

---

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
