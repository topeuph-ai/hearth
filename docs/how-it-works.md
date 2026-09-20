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

#### Being asked is a question, not an instruction

Appointing somebody used to be entirely one-sided. The holder wrote their key
into the circle and that was the whole of it: they might not know, might not
want it, and the first they heard was strangers appearing on their screen for
approval.

So the person asked is told, and **answers**. The answer is an entry in the
circle naming the appointment it answers, and only the person that appointment
names may write one. Answering again changes your mind; nothing is erased,
because the holder may have acted on what you said before.

There are **three states, not two**. Nobody asked; asked and not yet answered;
answered. "Has not got round to it" and "said no" are the same silence from
outside, and the holder does completely different things about each — which is
the whole reason the answer is written down rather than inferred.

What this does **not** do is make anybody agree to an arrival. Nothing could:
refusing has always been available by simply never endorsing anybody. What it
adds is that refusing reaches her, so she can ask somebody else.

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

### It is the only way in now

The waiting room began as a convenience sitting on top of inviting somebody by
their identifier. Both routes existed, which meant two long lines of characters
went between people and looked identical — and pasting one into the other's box
produced a raw error about a missing signature. Walking it, that happened
immediately.

So there is one route and one box. **Nobody sees an invitation.** It still
exists, and it is still the thing that actually admits somebody, but it is made
by the holder's app when she presses a name and collected by theirs; it is
never shown, carried, or pasted anywhere by a person.

What this costs is the thing to say plainly: **a circle's door lives on the
holder's device.** Which room belongs to which circle is remembered there,
because a circle cannot hold a reference to a network its members may not be
in. A holder who moves to a new device keeps the circle and loses the door, and
opens a new one. Before, inviting by identifier was a second way in that did
not depend on her device at all.

It buys the step that mattered more. Collecting an identifier from each person
before you can invite them is the step that defeats an elderly holder, and it
was the reason inviting anybody was a chore.

**One message still leaves**, to the person joining, because they are outside
the circle by definition. That is the door, and it is the whole point of there
being one.

### Why the answer can be left lying in an open room

An invitation is signed over one person's own key. It admits nobody else and is
a useless blob to anybody who picks it up — the code has said so since the
beginning, and this relies on it being true rather than hoping.

So the answer does not have to be carried by hand to the one person it is for.
It can be left where they will find it.

### A knock is sealed, because the room is not private

Anybody with a circle's address can read everything in its waiting room. While
inviting by identifier existed alongside this, somebody could join without ever
being announced at a door. Make the room the only way in and **every arrival
becomes visible to everybody holding the address** — "Ronnie Smythe, her
cousin", in the open. Who visits somebody is itself sensitive: a psychiatrist,
a substance misuse worker, a domestic abuse advocate.

So the words are boxed to the holder with
`ed_25519_x_salsa20_poly1305_encrypt`, which is in the HDK and needs no key
exchange — the holder's agent key is already in the room's own properties.

Two copies are sealed, not one. Boxing is between two keys and opened with the
**recipient's** secret, so a knock sealed only to the holder would be
unreadable to the person who wrote it. That is not academic: somebody let in
after a restart arrived nameless, in a circle that then asked them who they
were when they had already said it. Their app reads it back from their own
copy.

**What cannot be hidden is the key that wrote the knock.** It is the action's
author, and it is the whole reason nobody had to collect it by hand. So the
room still shows that somebody knocked and which key it was; it no longer says
who they say they are or what they say they are to the person.

One rule had to move. "Say what you are called" used to be checked by every
peer, and cannot be: a peer that cannot read a thing cannot have an opinion
about it. The check is now in the app, where it is a courtesy rather than a
rule. Nothing was lost by it — an empty name was never dangerous, only useless,
and the person it inconveniences is the one who wrote it.

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
Sealing them changes who can read a claim, not whether it is true. What would
actually help — a shared secret sent by a different channel — is written up in
[`to-a-product.md`](to-a-product.md) and is not built.

"Not now" writes nothing anywhere. A refusal recorded in an open room would be
a public snub, readable by everybody with the address including the person
refused. Nothing is owed to somebody who knocked uninvited, and silence is the
kindest available answer as well as the safest.

**That limit again, because it now matters more than it did.** Which room
belongs to which circle is remembered on the holder's own device. A holder who
moves to a new device keeps the circle and loses the door, and opens a new one.
While inviting by identifier existed alongside this, that was an inconvenience;
now it is the only way in, so it is the thing to fix next.

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

### Two different things called "removing somebody"

**Marking somebody as gone.** The holder records a departure; everybody else's
app stops listing that person as being in the circle. Worth building for the
ordinary case of a professional moving on, and it is **a courtesy, not a
control, and must never be described as one.**

An earlier version of this section said such an app would also "stop sharing
new material with them". That is not right: gossip is not something an app
chooses to do person by person. Every member's node holds and serves the
circle's data to every other member of it, with no per-agent filter anywhere
in that, so the **data** keeps arriving on their device whatever anybody
decides.

**But the data arriving is not the same as the person seeing it**, and an
earlier draft of this correction ran the two together — concluding that such a
removal would be near enough useless. That was wrong in the direction that
matters. The app that would refuse to show the circle is **their own app**,
which is this app. For somebody using the software as it ships, being removed
means opening Hearth and finding the circle gone. That is what removal means
to almost everybody who will ever be removed from anything.

So the honest picture has three levels, not two:

| | Does removal hold? |
| --- | --- |
| Somebody using the app as it ships | **Yes** — and this includes everything they had already read. The circle is gone from their screen |
| A copy they made outside the app: written down, screenshotted, photographed, remembered | **No.** Nothing anywhere changes that |
| Somebody who modifies the app, keeps an old build, or reads the database file directly | **No** — but it is work, and that is the point |

**The first line is stronger than it first looks, and an earlier draft of this
section got it wrong.** It is tempting to write the second line as "what they
have already read", and that is not the same thing at all. What they have
already read is still *inside the app*, and the app has just been told they
are not in this circle — so it goes, along with everything else about that
circle. Removal is not only about what happens next; it takes back the whole
of it.

The only thing genuinely beyond reach is a copy made **outside** the app,
which is a deliberate act: a photograph, a note on paper, a screenshot, a
memory. Somebody who sees removal coming can make one. That is true of every
record that has ever existed, on paper or otherwise, and it is the honest
boundary — not "they keep the data", which is not what happens.

**And the third line is a threshold, not a hole.** Nothing here is a
cryptographic guarantee that a determined person cannot read what is on their
own disk, and this document should not pretend otherwise. What it is, is a
real cost: modify the app, or find and read a SQLite file, or keep an old
build and stop it updating. That cost is beyond almost everybody, and for the
people this is built around — families, support workers, district nurses — it
is the difference between "I cannot see it any more" and "I can".

Making something difficult is a legitimate answer when the alternative is
nothing at all. What is not legitimate is calling it impossible, which is why
these three lines are set out separately rather than summarised into one.

It is also worth saying what the third line is **not**: it is not a
peculiarity of having no server. Anybody with a copy of anything can keep it,
and a server-based app in which data has already synced to a phone is in the
same position. What a server genuinely adds is that it can refuse to *send*
future updates — a real difference, and the subject of the next paragraph.

**One consequence to design around.** Somebody removed this way keeps
receiving every future version onto their disk, silently, for as long as their
app is installed. They cannot see it; the device has it. So a device that
falls into the wrong hands a year later holds the record as it is **now**, not
as it was on the day they were removed. Re-forming the circle does not have
that property: their copy stops at the moment of removal and stays there,
because nobody is writing to that circle any more.

That is the real difference between the two, and it is what decides which to
offer for what:

### Can a tampered-with app be spotted?

Asked because if it cannot be stopped, being able to notice it is the next
best thing. The answer is in two halves and they are opposites.

**Reading cannot be noticed. Not now, not ever, not by any design.** Somebody
who modifies their app to keep showing a circle after removal makes no request
to anybody: the entries are already on their disk, put there by ordinary
replication, and drawing them on their own screen touches no network at all.
There is nothing to observe because nothing happens.

This is the same fact that makes acknowledgements in this app a deliberate
written act rather than something inferred. **A DHT has no read receipts.** If
this app ever appears to tell you who has read something without them pressing
a button to say so, it is lying.

**Writing is the opposite: it is visible, signed, and the network denounces
it by itself.** Everything anybody writes is an action on their own chain,
signed by them, and served by the neighbourhood of their public key. Holochain
0.7 gives a zome one window onto it — `get_agent_activity` — which returns,
in its own documentation's words: the highest observed chain item; whether the
chain contains only valid items, contains at least one invalid item, is
forked, or is empty; **any warrants collected for invalid actions committed by
the agent**; and the hashes of valid and rejected actions.

A warrant is not a log entry somebody has to remember to write. When a peer
validates something and it fails, it issues a `ChainIntegrityWarrant` naming
the author, the action, its signature, and a human-readable reason — and that
warrant is gossiped like anything else. There is a second kind,
`ChainFork`, which carries two actions with the same sequence number as proof
that one identity has been run in two places at once. **That is exactly the
shape of "kept an old build alongside the new one", and it is self-proving.**

So, for a person who has been marked as gone and has modified their app:

| What they do | Noticed? |
| --- | --- |
| Read the record they still hold | **No.** Nothing is requested and nothing happens |
| Write anything at all into the circle | **Yes.** It appears, signed, with their name on it |
| Author something that breaks a rule | **Yes**, and the network says so of its own accord, with evidence |
| Run their identity in two places to keep an old build | **Yes** — a forked chain, proven by two actions at the same sequence |

**And the one thing that is not available at any price**: there is no way to
ask who is online, or whose node is running. The whole of what a zome may
learn about another agent in this version is `get_agent_activity`. Presence is
not observable, which is consistent with everything else here — a circle whose
members are all asleep is not a circle in trouble.

### Making removal hold in both directions

Two further ideas, both Ceri's, which between them close most of what is left
in the tables above. Neither is built. Both were checked against the Holochain
0.7.0 source before being written down here.

#### Everything written after removal: encrypt the record, change the key

The record is locked with a key that only members of the circle hold. Each
member's app is given its own copy, sealed so that only that member can open
it — the same kind of sealing the knocks at the door already use.

When somebody is removed, the holder's app makes a **new key** and gives it to
everybody except them, and everything written from then on is locked with it.

Their device still receives the new versions, because replication does not
choose person by person. But they arrive locked with a key that device was
never given. **A modified app does not help: there is nothing on the device
that can open them.** This is the part that is mathematics rather than
software behaving itself.

The HDK at 0.7.0 has what it needs: `x_salsa20_poly1305_shared_secret_create_random`
makes a key that stays inside the keystore, `..._export` and `..._ingest` pass
one to a named person, and `x_salsa20_poly1305_encrypt` / `..._decrypt` use it.
Somebody offline when the key changes finds their copy waiting in the circle,
addressed to them, when they come back — which is how everything else here
already works.

What it costs: the network cannot check the contents of something it cannot
read. Rules about **who** may write still hold; rules about **what** was
written move into the app, as they did for knocks. Suggestions and
acknowledgements need the same treatment, or they give away what the record
says. And it changes the shape of the record, which is a migration.

#### Everything written before removal: delete it from their device

The obvious question once the record is encrypted: the removed person's app
stops decrypting new versions, so could it also make the old ones unreadable?

It can do better than re-encrypt them. **It can delete them.** On seeing that
they have been removed, their app switches the circle off and deletes it —
and in Holochain 0.7.0 deleting a switched-off circle removes that circle's
database file from the device. Checked in the conductor: `delete_clone_cell`
calls `delete_cell_databases`, which deletes the store for any DNA no other
installed app still uses, and purges every row if the file cannot be removed.

That is different from leaving, on purpose. **Leaving switches the circle off
and keeps it**, so that somebody let back in finds it as it was — which is what
somebody who chose to go and changed their mind wants. **Being removed would
delete it**, and anybody later let back in starts fresh and receives the record
as it is then.

The limits, stated plainly:

- **It is carried out by their own app.** An ordinary copy does it. A modified
  one ignores the removal, and somebody who takes their device offline before
  the removal reaches it keeps everything. A threshold, as in the table above.
- **Deleting a file is not wiping a disk.** Recovery tools can sometimes find a
  deleted file. With the record also encrypted, what is recovered is
  scrambled — but the HDK has **no way to delete a key from the keystore**, so
  the key may still be on that device, and a determined examiner who also has
  the keystore's passphrase could open it.
- **A copy made outside the app is untouched**, as always.

#### Where that leaves the three levels

Both of the first two columns are now built, on the `migration-batch` branch —
see [encryption.md](encryption.md) for exactly what is locked and what is not.
The third is the removed person's own app deleting its copy, which is built as
well; what it cannot do is reach a device that has been changed to keep it.

| | Marking as gone | + encryption and a new key | + deleting on removal |
| --- | --- | --- | --- |
| Ordinary use of the app | Holds | Holds | Holds |
| Reading what arrives *after* removal, with a modified app | Does not hold | **Holds** | **Holds** |
| Reading what arrived *before* removal, with a modified app | Does not hold | Does not hold | Does not hold |
| A device found or examined later | Everything, readable | Old readable, new scrambled | Old deleted, new scrambled |
| A copy made outside the app | Does not hold | Does not hold | Does not hold |

With both, the everyday removal does nearly everything re-forming the circle
does — which makes re-forming a rarer last resort still.

#### What is worth building from that

Two cheap things. The first is built; the second is not.

**Write the departure down** — built, migration batch item 4. Marking somebody
as gone is an entry in the circle rather than a setting on one device, so who
was removed and when is part of the record everybody holds. Anything that
person writes afterwards is then visibly a write from somebody who was removed
— which is the whole of the detection anybody needs, and it cost one entry type.

**Show the health of each member's chain.** `get_agent_activity` on the people
in a circle would surface a forked or warranted chain without anybody going
looking. It is a genuine integrity signal and it is one call per member.

Neither of these detects reading, because nothing detects reading. What they
do is make continued *participation* impossible to do quietly, which is a
different and achievable goal.

**Re-forming the circle: the last resort, and the one that holds.** They are
excluded by mathematics rather than by everybody's app agreeing to behave, and
their copy is frozen on the day it happens.

It is the last resort because a circle takes time to establish — people
invited, a second person appointed, professionals who have read it and
acknowledged it — and re-forming starts that history again. For a support
worker moving to another job, marking them as gone is the proportionate
answer. For somebody who should not have been let in, or a safeguarding
situation, it is not, and this is what to reach for.

The objection to re-forming has always been the cost: everybody else has to
join the new circle. But that cost is an interface problem, not an
architectural one, and it is smaller than it looks:

1. The holder's app makes a new circle and copies the record into it.
2. It makes a door for the new circle, and an invitation for every member
   except the one being removed.
3. It sends each of them their invitation **as a signal in the old circle** —
   which it can, because the holder is still in the old circle with them.
4. Their apps join the new one and carry the label across. Nobody types
   anything.

The person removed is in the old circle, which simply stops being written to.
They keep what they already had, which is true of every design and cannot be
otherwise.

**None of that needs the integrity zome.** Signals, circle creation,
invitations and joining all exist and all live in the coordinator, so this is
buildable without a migration — which makes it a very different proposition
from most of what is left on this list.

**Built, 14 September 2026.** As set out above, with these decisions of
Ceri's:

- **Acknowledgements and decided suggestions come across as history**, written
  down on each device at the moment it moves: "Dr Patel read this before the
  circle moved, on 3 September". A reading stays beside the record until the
  record is next changed, as it would have in the old circle. Suggestions not
  yet decided are offered again by the person who made them, in their own
  name, because they are the only person who can sign them.
- **The second person's agreement is carried across**, unless they are the
  person removed. The holder's app asks the same person again in the new
  circle, and if they had agreed in the old one, their app agrees again on
  its own and tells them so. If they are the one removed, the holder is warned
  before the move that she will need to ask somebody else.
- **The people moved are told who was removed, by whom, and why** — "Margaret
  has removed Dave Smythe from the circle", and the reason if she gave one.
  Not that the circle moved: they cannot invite anybody, so a new door address
  is nothing to them, and the move is only how removal works.

Only the holder can send the message that moves people, and every member's
own app refuses it from anybody else — tested against running conductors:
another member was refused, the member moving received it, the member removed
received nothing.

The holder's app repeats the message once a minute, while it is open, to
anybody not yet in the new circle. Somebody who never introduced themselves in
the old circle is not moved, because nothing on the holder's device knows they
are there.

What it does **not** do, and nothing can: get back what they have already
read.

#### It can happen out of sight

Only the holder needs to know it happened. She presses one button; every other
member's app moves them in the background, and what they see afterwards is the
same record, the same names, the same tabs. Nobody is asked to do anything.

What cannot be hidden, stated plainly:

- **The removed person sees the old circle go quiet.** Nothing new arrives.
  They are not told why, but somebody paying attention will notice.
- **Somebody offline moves when they are next online at the same time as
  somebody who has already moved.** Until then they are in the old circle.
- **The history comes across as history.** Acknowledgements and suggestions
  are copied as a record of what happened, not as the original signed entries,
  because a signature belongs to the chain it was written on.
- **The door has a new address.** Any invitation not yet used has to be sent
  again.
- **A second person's agreement** has to be either carried across as history
  or asked for again. Not decided.

#### When re-forming is the right answer, and not just the strongest

Deleting the removed person's copy and changing the key covers the everyday
case. The test for reaching past it is simple: **re-form when the problem
cannot be fixed by the removed person's app co-operating, or when what needs
hiding is something encryption does not hide.**

1. **A lost or stolen device.** Nobody has done anything wrong, but somebody
   else is now holding a member's identity. "Their app deletes its copy" does
   nothing, because a stranger controls that app. The member joins the new
   circle from a new device; the old identity is never invited.
2. **Somebody who keeps writing after removal.** Marking somebody as gone hides
   them; it does not stop them *writing* into the old circle — suggestions,
   false introductions at the door, harassment. It still arrives on everybody's
   device, even if honest apps hide it. In a new circle they cannot write at
   all.
3. **When even the shape of the activity is dangerous.** Encryption hides
   *what* was written, not *that* something was written, *when*, or *by whom*.
   Where the removed person is a risk to the person the record is about, seeing
   that somebody new joined on Tuesday, or that three people read the record at
   two in the morning, can be enough to guess an admission or a move. A new
   circle shows them nothing.
4. **Something was written that should never have been.** Changing the record
   does not remove earlier versions: every member's device keeps them. A new
   circle carries across only the record as it stands, so the mistake does not
   travel with it.
5. **Something is getting out and nobody knows who.** The door address has
   spread, or something has leaked. Move, and carry across only the people you
   are sure of.
6. **Until encryption is built.** Today nothing locks the record, so re-forming
   is the only removal that stops a determined person reading what is written
   next.

#### The same machinery upgrades will need

Moving every member to a new circle, without anybody typing anything, is
exactly what [changing the integrity zome](to-a-product.md#0-the-one-nobody-has-written-down-upgrades)
will require once real records exist: every circle moved onto the new version.
Built well once, it serves both.

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
