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

**Second: name the rule in the DNA, not the person.**

Today the second person's key is in the circle's properties, so changing who
they are means a new circle and everybody joining again. That is fine when it
is decided at the start and painful whenever it is not — and it is often not,
because you need their identifier before the circle exists.

The answer is not to weaken the safeguard. It is to notice that the properties
only have to carry **the rule** — a `requires_second_yes` flag — and not
**the person**. The flag stays immutable, so the requirement still cannot be
dropped quietly, which is the whole of what it protects. Who the second person
is becomes an entry the holder writes whenever she has their key.

**This is not speculation; it is how Moss does it.** Their group DNA properties
carry a single `progenitor`, and every other permission is a `StewardPermission`
entry in the DHT:

```rust
pub struct StewardPermission {
    pub permission_hash: Option<ActionHash>,
    pub for_agent: AgentPubKey,
    pub expiry: Option<Timestamp>,
}
```

The trick that makes it deterministic is that **an action cites the permission
it relies on, by hash**. Validation does `must_get_valid_record(permission_hash)`
— a fixed hash, so every validator reaches the same answer — and the chain back
to the progenitor is enforced implicitly, because a *valid* record was already
checked against whoever issued it. Permissions cannot be updated or deleted,
only expire. See
[`dnas/group/zomes/integrity/group/src/steward_permission.rs`](https://github.com/lightningrodlabs/moss/blob/main/dnas/group/zomes/integrity/group/src/steward_permission.rs).

The one thing that could have sunk this was checked: the membrane is enforced
in two places, and `genesis_self_check` has no network access — it "verifies as
much as it can without network access", while `validate` "can access DHT data".
So the local check would confirm the founder's signature and that a second
signature is present at all; the network check would resolve the appointment and
verify who gave it. Per Holochain's own documentation, that split is exactly
*why* the two callbacks exist.

**What it would buy:** appointing a second person later, or changing them, stops
meaning a new circle and stops meaning everybody re-joins. What it costs is a
migration, which is why it is on this page and not in the code.

**Third: sign the name on an invitation.**

An invitation now carries who the holder says the key belongs to, because
without it the second person was being shown a line of base64 and asked to
agree — and the only thing they could honestly agree to was that they had been
asked. It is her claim, nothing checks it, and every screen that shows it says
so.

**It is not signed**, and the reason is the freeze: signing it means putting it
inside the membrane proof, which is built in the integrity zome.

The limit that leaves is narrow but real. The signature is over the key, so
altering the name in transit cannot admit anybody the holder did not already
sign for — what it can do is mislead the person deciding. And misleading that
person is exactly the threat the second yes exists for: somebody being talked
into an admission. Whoever is doing the talking is usually also the one
carrying the message.

So when the integrity zome next moves, the name belongs inside what gets
signed.

**For contrast, two routes deliberately not taken.** Unyt's answer to mutable
membership is [`joining-service`](https://github.com/unytco/joining-service), "a
per-hApp REST API that brokers onboarding… controlling who can join" — which is
an operator, and this project exists because nobody will be one. AD4M has no
`genesis_self_check` anywhere in its repository; its membranes live above
Holochain in the Language layer, which is a different architecture rather than a
different answer to this question.

## 0c. Who is actually at the door, and what happens when somebody cannot answer

Two questions that came out of walking the waiting room. Neither is built.
Both are written down here because the reasoning is the expensive part.

### Being fairly sure the person knocking is who they say

A knock carries a name and a relationship, and neither is checked by anything.
The people deciding are being asked to judge a stranger from two lines of text.

**The tempting answer is to ask for more: a full name, an address, a date of
birth.** It should be resisted, and not because it is hard.

Nobody can check any of it. There is no operator — that is the whole project —
so the screen would show a home address with all the authority a home address
carries, and behind it would be a text box anybody can type into. Every other
name in this app says "claimed", or "in their own words", for exactly this
reason. Collecting more unverified detail does not make identity more certain;
it makes it *look* more certain, which is worse than saying nothing.

It is also a real data protection escalation. A waiting room is an **open**
network. A name and a relationship sitting in it is one thing; the home address
of somebody connected to a vulnerable person is another, and it would need the
[DPIA](DPIA.md) rewritten rather than amended.

**The answer that does work is a shared secret, sent by a different channel.**

> The holder tells the person a word, on the phone. They type it into their
> knock. She sees it beside their name.

This proves nothing to the world, and does not have to. It proves it **to her**,
which is the only thing she needs in order to decide. It carries no personal
data, it cannot pretend to be more than it is, and it costs one box.

The same idea can be applied to the room itself — a password to get in at all,
stored as a hash in the room's properties and presented as a membrane proof, so
every peer checks it and no server is needed. Two things to be clear about if
it is built:

- The hash is readable by anybody who has the address, so a weak password can
  be guessed offline. It wants a real secret, or a deliberately slow hash.
- **The address already contains a random secret.** So a room password is not a
  second lock so much as a second *channel* — address by message, password by
  voice. That is a genuine improvement, and it is worth building for that
  reason rather than the one it appears to offer.

An app password is a separate matter and an ordinary one: it protects the keys
on the device, and the desktop shell can do it.

### When somebody cannot answer any more

**If the second person is incapacitated, this is solved.** Who agrees is an
entry the holder writes and can write again, so she appoints somebody else and
nobody re-joins anything. That was the whole reason for moving the person out
of the circle's identity.

**If the holder is incapacitated, it is not solved.** Her key *is* the circle's
identity and cannot be changed. Nobody can write the record, nobody can admit
anybody, and the circle becomes read-only for good.

One suggestion was to make the holder always be the person the record is about,
so that the circle's purpose ends when they do. **It does not work, and the
reason matters:** the population About Me exists for most is often exactly the
people who cannot hold their own record — advanced dementia, learning
disability, brain injury, a child. That is why holder and subject are separate
in the first place.

And death is not the failure case. **Capacity is lost gradually.** If the
holder had to be the subject, a dementia circle would become unmanageable
precisely as the dementia progressed — unusable at the moment it is most
needed.

What does survive from that suggestion, and is worth keeping: **when the
subject dies, the circle's job is finished.** So succession only has to answer
the narrower case of the *holder* becoming unable while the subject is still
living. That is a much smaller problem than "what happens when somebody dies".

**The way out that exists today is to re-form the circle**, and it works here
for a specific reason: every member already holds a complete copy of the
record. A daughter can make a new circle, seed it from the copy on her own
machine, and bring everybody across. Nothing is lost except the old circle's
identity.

What is missing is that nothing distinguishes a rightful successor from
somebody helping themselves. There is no authority to appoint one — inventing
that authority means inventing an operator — so the members decide by which
circle they join, and disagreement means two circles. That is uncomfortable and
it is the honest consequence of having nobody in charge.

There is a reading that makes it less uncomfortable, and it may be the right
one. **An About Me is the person's own account of themselves.** Quietly
transferring authorship of it to somebody else when they can no longer object
is arguably the wrong thing to build. A successor circle, seeded from theirs,
with `supported_to_write_this_by` naming whoever took over, says what actually
happened.

Either way it should be a supported act with a button on it — *"start a new
circle from this one"* — rather than something a family works out during a
crisis.

## 0d. The shape of joining, and the shape of holding

From walking the waiting room on 2026-09-11. The first three are decided; the
last is not, and is the one that matters most.

### Decided: the waiting room is how you join

Not an option beside the invitation route — **the** way in. The invitation
route begins with "send me the long line of characters from your app", and that
is the step where this stops being possible for somebody elderly or being
helped. It becomes the fallback behind the door, not a thing anybody is offered.

### Decided: two choices on the front page, and one box behind them

**Create a circle** and **Join a circle**, and nothing else. Everything else
belongs inside a circle, where it has a context.

"Agree to somebody joining" is already vestigial — nothing in the app produces
the half-invitation it consumes.

**And "Join a circle" is one box that takes whatever you were sent.** Not two
routes with two screens. A waiting room address and an invitation are genuinely
different things — one is public, reusable and lets you *ask*; the other is
made for one person and lets you *in* — but that is a difference the app can
work out for itself, and it is not a difference anybody should have to hold in
their head before they can begin.

The app already has both detectors. They were written on 2026-09-10 to catch
people pasting into the wrong box, after a waiting room address went into the
invitation field and came back as "Cannot read properties of undefined (reading
'signature')". **With one box there is no wrong box**, and that whole class of
mistake stops existing rather than being caught and explained.

The forms underneath are the same anyway: both ask your name and how you are
connected. Knocking uses them to tell the holder who is asking; an invitation
uses them to introduce you to the circle. Same two questions.

### Decided: an invitation stops being something anybody sees

There is one way in — the address — and no second route to choose between.

**The invitation does not disappear; it becomes invisible.** It cannot be
removed: the membrane requires the holder's signature over the joiner's key,
and that signature *is* the invitation. Without one the door refuses. But
nobody has to see it, and in the waiting room flow nobody already does — her
app makes it, leaves it at the door, his app collects it and uses it. Neither
of them has ever looked at one.

So the screens go and the machinery stays. The zome functions stay too; they
are tested, and if this turns out to want a fallback the screens come back
cheaply.

**Two things this gives up. The second one has to be paid for.**

**The fully asynchronous hand-off.** An invitation could be made on Tuesday,
texted, and used on Friday. A knock needs the holder reachable when somebody
knocks, and the knocker reachable when she answers. Both are stored — neither
has to be live — but it is two meetings instead of one delivery.

Judged acceptable, on the grounds that whoever is knocking can be told plainly
that they are waiting, and **ringing the holder up to remind her is a perfectly
good thing to do**. The app does not have to carry every message. It already
says there is nothing else to do; it should also say that this may take a
while, and that a phone call is allowed.

**Everybody now arrives through a semi-public room, and this is the real
cost.** A knock says "Ronnie Smythe, her cousin" in a network anybody with the
address can read. An invitation let somebody join without ever being announced
at a door. Make the room the only way in and **every arrival becomes visible to
everyone holding that address** — including a psychiatrist, a substance misuse
worker, a domestic abuse advocate. Who visits somebody is itself sensitive.

**So encrypting knocks moved from a nice-to-have to a requirement**, and
landed in the same change rather than after it. The name and relationship are
boxed to the holder with `ed_25519_x_salsa20_poly1305_encrypt`, and a second
copy is boxed to the person knocking so their own app can read back what they
said. What stays in the open is the key that wrote the knock, which cannot be
hidden — it is the action's author, and it is the reason nobody has to collect
it by hand.

One validation rule had to move to the app to pay for it: "say what you are
called" cannot be checked by peers who cannot read the words. See
[`how-it-works.md`](how-it-works.md).

### Already true, and worth not rebuilding: she answers at leisure

A knock is an entry in the room, not a message. It sits there. The holder does
not have to be online when somebody knocks, does not have to catch a
notification, and can look tomorrow morning and find three people waiting. The
banner is a convenience; the list is the fact.

That is the half of the waiting room that helps *her* rather than the joiner,
and it is easy to miss. With invitations she had to **produce** something on
demand — collect an identifier, make a token, send it. Now she **responds**,
when she feels like it. The work moved off the person who is already exhausted.

**What she cannot see is who is merely present in the room.** Peers announce
themselves in order to find each other, so the keys are visible, but that is
all: `uhCAkIspW47G89hwKm1…` and nothing else. The knock is what turns "a key is
present" into "Ronnie Smythe, her cousin, is asking" — which is why it cannot
be skipped. Not for security; because a list of keys is not something a person
can act on.

### Built: nominate by name, and ask the person

The holder picks the second person from the list of people in the circle, by
pressing their name. There is no key to paste and no separate panel: the act
is where the people are.

The person is then **asked**, and answers. Before this she appointed
unilaterally and they discovered they had a job — the first they heard was
strangers appearing on their screen for approval.

The answer is an entry, `Consent`, naming the appointment it answers, and only
the person that appointment names may write one. That is the whole of the new
integrity-zome surface: one entry type, one link type, one rule.

**Three states, not two.** Nobody asked; asked and not yet answered; answered.
The holder does entirely different things about each, and "has not got round to
it" and "said no" are the same silence from outside. That is the only reason
the answer is written down rather than inferred.

It cannot make anybody agree to an arrival, and does not try. Refusing has
always been available by never endorsing anybody. What it adds is that refusing
**reaches her**, so she can ask somebody else — which she does by pressing
another name.

### Open: redundancy and a check are not the same thing

The suggestion was to replace "two people must agree" with **two admins of
equal rights**, on the grounds that it also answers what happens when one of
them cannot be reached.

**They are opposite mechanisms.**

| | requires | protects against |
|---|---|---|
| Two must agree | **both** | the holder being **pressured** |
| Two equal admins | **either** | the holder being **unavailable** |

One raises the bar for admission. The other lowers it.

**Equal admins make the pressure case worse.** The scenario the second yes
exists for is somebody talking the holder into admitting a person. With two
equal admins there are two people who can be leaned on and either one is
enough — strictly weaker than one holder. Building that and calling it the same
safeguard would be the most dangerous kind of mistake available here: a
protection that reads as stronger and is not.

**But the worry underneath it is right, and it is the real unsolved problem.**
If the *second* person is incapacitated the holder appoints somebody else and
the circle carries on. If the **holder** is incapacitated the circle freezes for
good, and nothing in the design answers that. For a record about somebody in
declining health, that is not a rare case.

So these are **two features, not one**. Redundancy, so a circle survives the
person holding it. A check, so admission cannot be done quietly. A circle could
have either, both or neither.

**One constraint on any co-holder, and it is not negotiable:** they may admit
people; they may **not** write the record. "Only the person may write their own
About Me" is load-bearing — the record keeps one voice, and everybody else's
knowledge comes in as suggestions the holder accepts. A co-holder who can edit
it breaks the thing the whole record is.

**Where this stands after the four built items.** The co-holder is not built.
Nothing above it required a decision about it, and it is the one piece of this
that would change what a circle *is* rather than how it is used, so it is worth
leaving until somebody outside this room has looked at it.

Two things about it are now firmer than they were.

The second yes is already **only a tripwire**, and the consent work above makes
that plainer rather than changing it: who agrees is an entry the holder writes,
so she can write another one naming anybody, including a second device of her
own. The network cannot refuse an invitation that names no appointment, because
"has she appointed anybody yet" is a question whose answer changes. What stops
a quiet admission is that everybody can see who was asked and when. A co-holder
would not weaken a wall; it would stand beside a tripwire.

And **the door now lives on one device**. With inviting-by-identifier gone,
a holder who loses her device keeps the circle and loses the only way into it.
That is the incapacity problem arriving early, by a different route, and it
makes the co-holder case stronger than it was when this section was written.

**Claude's recommendation, not a decision taken:** if only one of the two gets
built, build the co-holder. The pressure case is real but rarer; the protection
against it is already only a tripwire; and "the person holding this died, and
the circle died with them" is the thing that will actually happen.

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
