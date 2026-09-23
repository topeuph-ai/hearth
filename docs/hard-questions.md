# The hard questions

In September 2026 a red-team review of Hearth at v0.2.1 — carried out by
another AI model, asked to attack the project rather than admire it — put
fourteen questions to it. They are not feature requests. They ask whether the
proposition survives contact with reality.

Each one is answered here, checked against the code and the other documents
rather than accepted or dismissed. Where the honest answer is "this is not a
software question", that is the answer given.

The review's sharpest line is worth keeping in front of everything else:
**in several places Hearth has not solved a problem, it has moved it** — from
database engineering into governance, identity, safeguarding or law. A
hosted service has administrative answers to most of these questions. Hearth
removed them on purpose, and has to find social ones instead.

| | Question | Where it stands |
| --- | --- | --- |
| 1 | Why does everyone in the circle see everything? | **Open — a decision** |
| 2 | Who has legitimate authority to act for the person? | Not a software question; stated, not solved |
| 3 | What does "held by the person" actually mean? | Needs saying precisely |
| 4 | What can Hearth do that RIX cannot? | **One of three partly shown** |
| 5 | What happens when the holder disappears? | **Built, not yet tried on machines** |
| 6 | What does an acknowledgement prove? | Answered below |
| 7 | Should set-aside suggestions live forever? | **Decided and built** |
| 8 | What does the waiting room expose, and can it be flooded? | **Limit per key, and a new address, built** |
| 9 | What happens if somebody lies? | Answered: access, not identity |
| 10 | What does "no operator" really mean? | Overclaimed; corrected |
| 11 | Who authorises the next version of a circle? | Answered by circle-moving |
| 12 | Can a person recover their identity? | Not solved |
| 13 | Whose voice wins when the circle disagrees? | The holder's; rests on 2 |
| 14 | Does the circle survive organisational churn? | The outer ring |
| — | How large may an entry be? | **Decided; interface built** |

---

## 1. Why does everyone in the circle see everything?

**The review is right that this is the most important question, and right
that the answer changes what a circle is.** Today, being admitted means
reading the whole record and, until today, every suggestion anybody made. A
district nurse who needs "how to talk with me" also reads "my wellness".

Two defensible answers, and they lead to different products:

- **(a) A circle is the people trusted with the whole account.** About Me is
  one person's account of themselves, meant to be read whole. Anybody who
  needs less — a professional seeing the person once, a ward at 3am — does not
  join; they are given a pass to read in the
  [outer ring](outer-ring.md), which is already designed. Circles stay small
  and simple, and the permission question becomes "inner or outer", which a
  person can understand.
- **(b) Access section by section.** Closer to what a hosted system offers,
  much more complex to build, and it turns Hearth into an access-control system
  — with every screen having to explain who can see which part.

**Not decided.** Before recommending either, read what the PRSB About Me
implementation guidance says about sharing part of a record rather than all
of it.

## 2. Who has legitimate authority to act for the person?

**Hearth cannot know, and must say so rather than imply otherwise.**
Cryptography proves which key acted. It does not prove that the person behind
the key has any right to act for somebody else. A daughter, a case manager and
a deputy all look the same to the software.

What the software does, and it is worth something: it makes the holder
**explicit, singular and visible**, where today "who is in charge of Mum's
information" is an unwritten understanding. Who was asked to agree to who
joins, and when, is written in the circle for everybody in it to see.

What it does not do: decide whether that holder is the right one. That is a
Mental Capacity Act question — lasting power of attorney, deputyship, best
interests — and it is answered by the care arrangement around the circle, not
by the circle. The [DPIA](DPIA.md#holder-and-subject-are-different-roles) says
so; product language needs to say so too, up front.

**The malicious holder** is the hardest version. The architecture makes the
holder powerful on purpose, so an abusive relative holding the circle is
powerful too. The protections that exist are social, not technical: the
second person's agreement is visible, the record says who supported it to be
written, and every professional in the circle can see what is written about
the person and raise a safeguarding concern through the ordinary route. Hearth
does not replace safeguarding and must never be described as though it does.

## 3. What does "held by the person" actually mean?

Five different things could be meant, and they come apart as soon as there is
a proxy:

| Kind of ownership | Who has it |
| --- | --- |
| **Subject** — who the record is about | The person |
| **Author** — who may write it | The holder |
| **Cryptographic** — whose key the circle is built around | The holder |
| **Membership** — who decides who joins | The holder, with a second person if the circle asks for one |
| **Legal** — who is responsible for the data | See the [DPIA](DPIA.md#3-who-is-responsible-for-what) |

So when the holder is the person themselves, "held by the person" is true in
every sense. When it is their daughter, it is true in one sense — it is about
them — and the rest belongs to the daughter.

**The precise claim:** Hearth is held by the person, **or by whoever acts for
them**, and never by an organisation. That is the distinction that matters
against a hosted service. Product language should say it that way rather than
let "held by the person" suggest more.

## 4. What can Hearth do that RIX cannot?

RIX Multi Me is person-held and operator-hosted, established, NHS-experienced
and certified. "People should own their About Me" is not an advantage over it.
**The difference is having no operator, and the review is right that this has
been argued rather than shown.**

Three scenarios that could be *demonstrated*, not claimed:

1. **The organisation behind Hearth disappears, and every circle carries on.**
   Nothing a circle needs is held by the project.
2. **The internet goes, and the people in the room keep sharing.** Two devices
   on the same local network pass a change between them with the broadband
   unplugged. A conventional hosted service does not: every change goes up to
   its server and back down. (One could be built to sync locally too; here it
   is the ordinary shape of the thing, not an extra.) **✅ Shown 14 September 2026**, on two Windows machines on one
   home wifi with the broadband unplugged: a change on one arrived on the
   other. The two had met online first, and **after both were restarted with
   the internet still gone they never found each other** — until 19 September,
   when a field-test build of Holochain with local-network discovery found the
   other machine in about a minute after an offline restart. Released Hearth
   waits for official Holochain to have it. See
   [what-is-proven.md](what-is-proven.md#two-machines-and-then-no-internet)
   for exactly what it does and does not show.
3. **A professional contributes without their organisation joining anybody's
   system.** No data-sharing agreement with a platform, because there is no
   platform to share with.

One of the three has now been watched happening; the other two have not. And
none has been filmed, which is what would let somebody who was not in the room
believe it.

## 5. What happens when the holder disappears?

**Written down as unsolved** in
[to-a-product.md](to-a-product.md#when-somebody-cannot-answer-any-more), and
the review is right that for this population it is the expected course of
events rather than an edge case: dementia progresses, devices are lost,
holders die.

**Half of the answer was built on 14 September 2026.** Moving a circle — a new
circle with the record carried across and everybody moved by their own apps —
is exactly the "start a new circle from this one" that succession needs.

**The other half is authority.** Today only the holder can send the message
that moves people, on purpose, so that nobody else can lead a circle
elsewhere. When the holder cannot press the button, somebody else has to be
allowed to, and deciding who is the same problem as question 2. Options, none
chosen: a successor the holder names in advance; the second person; agreement
of several members. All of them trade safety against getting stuck.

**Since built (migration batch, test releases 0.2.7 onwards):** the holder
names a successor in advance, and a checker who is never the successor. A
claim that she has gone starts a waiting period the whole circle can see, in
which she can say "I am still here" and stop it; the checker, or failing them
anybody else in the circle, confirms. Then the successor moves the circle, and
the moved circle starts with keys of its own, so nobody removed comes back.
Not yet tried on two machines. See [migration-batch.md](migration-batch.md).

## 6. What does an acknowledgement prove?

| It proves | It does not prove |
| --- | --- |
| A key in this circle said it had read **this exact version** | That the key belongs to the role it claims |
| When it said so | That the person understood, agreed with, or followed it |
| That the version has not changed since, if it is still shown | That they remember it three weeks later |
| | That what was written is correct |

**What it is for:** a family can see that professionals have opened the
current record, rather than hoping they have. That is a thing families
currently have no way of knowing, and it is enough to justify it.

**What it is not:** a verified professional attestation. If the NHS ever needs
that, it needs a verified identity underneath, and this is a placeholder for
it. The interface already says "Role claimed"; it should never say less.

## 7. Should set-aside suggestions live forever?

**Decided 14 September 2026, and built: no longer shown to everybody.** A
suggestion that was set aside is shown only to the holder, who decided, and
to whoever offered it, who deserves to know what became of it.

Ceri's reasons were both of the review's and one more: a circle of ten loving,
enthusiastic people will have a great deal to suggest, and the box fills with
things that went nowhere; and something sensitive a support worker offered and
the holder turned down should not stay readable by the whole circle.

What it is, stated the same way as removal: every copy of Hearth as it ships
hides it. It is still on members' devices, because nothing written in a circle
can be unwritten, and a modified app could show it. Making it unreadable rather
than hidden needs the encryption planned for the migration batch.

## 8. What does the waiting room expose, and can it be flooded?

What is exposed to anybody with the address: that somebody knocked, the key
they knocked with, and when. The words are sealed to the holder. That is
[documented](how-it-works.md#a-knock-is-sealed-because-the-room-is-not-private).

**Flooding is new, and the review is right.** Nothing counts knocks. Anybody
with the address can knock as often as they like, and every knock is stored
on the holder's device.

- **Cheap, no migration:** a button to change the door address, so a flooded
  or leaked door can simply be abandoned — circle-moving already does this as
  part of a move. And the holder's screen showing repeated knocks from one key
  as one.
- **In the migration batch:** a limit on knocks per key, checked by every peer.

**Since built:** ten knocks per key, checked by every device. An outside review
(23 September 2026) rightly points out that keys cost nothing to make, so
somebody determined can knock from many. The answer on the list above is
built in 0.3.4: **Give the door a new address**, under the address on the
invite page. The flooded door is simply left behind. It costs the holder
something, and says so before she presses it: the old address stops working,
and so does every pass she has given, since passes are read through the door.

## 9. What happens if somebody lies?

**Hearth controls who has access. It does not establish who anybody is.**
"I am her district nurse" is a claim, shown as one everywhere. An invitation
cannot be passed to another key, but a person let in can describe themselves
however they like.

That is a deliberate line and the review's advice is right: future versions
must not drift across it quietly. Any feature that makes a claim *look*
verified — a badge, a title, more fields — without verifying it is worse than
none. See [the note on who is at the door](to-a-product.md#being-fairly-sure-the-person-knocking-is-who-they-say).

## 10. What does "no operator" really mean?

**The README overclaimed, and has been corrected.** It said "There is no
server". There are servers: devices use public bootstrap and relay servers to
find each other across the internet, and the desktop app currently points at
Holochain's public test servers. They carry addresses, never records.

The accurate claim, and a stronger one: **no central service stores or
controls the record.**

The real test the review proposes is the right one: *could every circle carry
on if the organisation maintaining Hearth disappeared?* Record storage — yes,
it is on members' devices. Finding each other across the internet — only while
somebody runs a bootstrap server, which anybody can (the binary is in
`./bin`). Software updates and security fixes — only while somebody maintains
it. So "no operator" means no operator *of the data*, and the remaining
infrastructure should be listed plainly wherever the claim is made.

## 11. Who authorises the next version of a circle?

**Answered by what was built on 14 September 2026.** A change to the
integrity zome makes a different network, so no release can reach into an
existing circle and change it. The project can publish a new version; **only
the holder's own button moves her circle onto it**, using the same machinery
that removes somebody.

So the project can offer an upgrade and cannot impose one. That is the
difference the review hoped for: the maintainer never becomes an authority
over anybody's circle.

## 12. Can a person recover their identity?

The review's three-way split is the right one:

- **Data recovery — yes.** Every member holds a full copy of the record.
- **Identity recovery — no.** A lost key is a new agent. There is no account to
  reset and nobody to reset it.
- **Authority recovery — only by moving the circle**, which today only the old
  key can start. For a member, the holder lets their new device in. For the
  holder, see question 5.

Written down in [to-a-product.md](to-a-product.md#2-losing-a-device-loses-the-person-not-the-record);
elevated here because for this population a lost or replaced phone is routine.

## 13. Whose voice wins when the circle disagrees?

**The holder's.** Suggestions let everybody offer, and the holder decides —
deliberately, so the record has one voice. That is a governance answer, and it
rests entirely on question 2: the holder's voice is only the right one if the
holder is the right person. Hearth keeps the trail of what was offered; it does
not referee.

## 14. Does the circle survive organisational churn?

A real care arrangement passes through many professionals, most of them for a
short time. Making each of them a permanent member would turn membership into
administration.

The answer already designed is the one the review arrives at independently:
**a stable inner circle, and temporary access outside it** — the
[outer ring](outer-ring.md). It also connects to question 1: if professionals
who need less sit outside the circle, "everyone in the circle sees everything"
becomes much easier to defend.

## How large may an entry be?

**Right, and confirmed in the source.** Nothing in the integrity zome limits
the length of any text. The only limit is Holochain's own —
`ENTRY_SIZE_LIMIT`, four megabytes an entry, in `holochain_integrity_types`
0.7.0 — and every change stores another whole copy on every member's device.

**Decided 14 September 2026: 500 words a section**, Ceri's number, easy to
raise if anybody asks. Built in the interface for the seven sections and for
suggestions. The count appears in the last hundred words, and going over is a
sentence under the box rather than a refusal with no reason.

That holds for Hearth as it ships. The rule every peer checks belongs in the
migration batch.
