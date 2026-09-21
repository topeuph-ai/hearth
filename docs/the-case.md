# The case for Hearth, and the questions people will ask

**Written 21 September 2026.** For conversations with funders, and with anybody
who might champion this. Facts about other organisations were read at source on
the date given; where something was not checked, it says so.

This does not repeat [funding.md](funding.md), which covers the UK routes and
was source-checked on 8 September. What is here is **the argument**, the
**objections**, and **who to show it to — inside the UK and outside**.

---

## The thirty-second version

> Hearth is a record of what one person wants their carers to know — what
> matters to them, how to communicate with them, what to please not do. It is
> shared across the organisations around that person, who will never share a
> system with each other. It has **no operator**: no company, no server, no
> database anybody owns. The people in the circle hold it between them, and it
> is locked so that somebody removed from the circle cannot read what is written
> afterwards.

Everything else follows from those two words, *no operator*.

---

## Why it exists: the deadlock, not the technology

Every previous attempt at a shared record around one person has failed at the
same place, and it is not a technical place.

To span organisational boundaries, somebody must **become the operator** — the
legally responsible party holding data about a vulnerable person, contributed by
seven organisations, who must still exist in twenty years. That party carries
the liability, the subject access requests, the breach exposure and the
obligation to outlive everybody. Microsoft HealthVault was not that party; it
closed in 2019 and took the data with it.

**The blocker was never technical. It was that nobody will hold the parcel.**

With no operator, the act changes shape. A professional **shares from** their
own system rather than **deposits into** somebody else's. That is a different
act legally, and it is the whole of why this is worth building.

This is the single most important thing to say in any funding conversation, and
it is the thing most likely to be missed if the demo goes first.

---

## How it differs from what already exists

Four families of thing, and Hearth is none of them.

### 1. Provider-side records — the care home's system, the council's system

Digital social care records, case management systems, EPRs. The record belongs
to the **organisation**. It is excellent inside those walls and stops at them.
Change provider and the record does not come with you; the new provider starts
again, and the person repeats themselves.

Hearth is not a competitor to these. It is the layer they have never had: the
person's own account, which every provider can read and none of them owns.

### 2. Patient portals and personal health records — the NHS App, PHR vendors

The person gets a view onto data, and some control over sharing. But the
platform has an owner: a company or a health system. "Person-controlled" here
means *controlled within somebody's platform*, and the platform must be paid
for, procured, assured, and kept alive.

### 3. Personal data pods — Solid, and this is the closest comparator

The strongest version of this idea is running in Flanders. The Flemish
government's utility company **athumi** is deploying Solid pods for **6.5
million citizens**, on Inrupt's Enterprise Solid Server, with five Belgian
hospitals writing hospital-visit information into people's pods
([Inrupt case study](https://www.inrupt.com/case-study/flanders-strengthens-trusted-data-economy),
[athumi](https://athumi.be/en/technologies/solid), read 21 September 2026).

Read that again, because it makes the argument better than any slide could:
**to give citizens personal data pods, Flanders had to create a state-owned
company to host them.** That company *is* the operator. Somebody had to be
invented to hold the parcel.

That is not a criticism of Solid — it is a serious, well-funded, genuinely
person-centred programme, and if it works it will help millions. It is the
proof that the deadlock is real: the most advanced attempt at person-held data
in Europe answered "who holds it?" by founding an institution.

**Hearth asks whether you need one at all.** No pod provider, no hosting, no
per-citizen cost, no entity to procure. The circle is held by the devices of
the people in it, and there is nothing in the middle to fund or to breach.

### 4. Paper — "This is me", hospital passports, one-page profiles

The thing Hearth is actually competing with, and it is worth respecting.

Paper has no operator either. That is exactly why it has survived every digital
attempt to replace it: it crosses every boundary, needs no procurement, and
nobody has to sign a data-sharing agreement for a laminated sheet in a bag.

What paper cannot do: be in two places at once, be updated by a daughter in
another town, show who has read it, or stop being readable by somebody who has
been removed. **Hearth is the digital form of paper's one great property** —
nobody owns it — with the properties paper lacks.

---

## Does it matter outside the UK?

Yes, and the honest split is this: **the vocabulary is British, the shape is
not.**

- **What is British:** the PRSB *About Me* standard, the seven headings, the
  language of the sector. That is a thin layer. Every country has some version
  of "what this person wants people looking after them to know" — *This Is Me*,
  one-page profiles, the Dutch *persoonlijk gezondheidsomgeving*, the German
  *Patientenverfügung* adjacent traditions.
- **What is universal:** somebody with dementia, a learning disability or a
  complex condition, surrounded by people from organisations that do not share
  systems, repeating themselves to each new face.

And the legal wind is behind it in Europe. The **European Health Data Space**
regulation (EU) 2025/327 came into force on 26 March 2025, phasing in across the
rest of this decade — citizens getting access to and control over their health
data across member states, with patient summaries portable EU-wide by March 2029
([overview](https://en.wikipedia.org/wiki/European_Health_Data_Space), read 21
September 2026). EHDS is about clinical data held by providers, which is *not*
what Hearth holds — but it establishes the principle that the person, not the
institution, is the centre of the record. Hearth is the same direction, arriving
from the other end.

**On "surely someone abroad would have built it":** they have built the
centralised versions, repeatedly, and in the most advanced case founded a public
company to run it. What nobody has shipped is the version with nothing in the
middle — because until recently the tooling did not exist, and because the
funding model for "no operator" is unobvious: there is no per-seat licence to
sell.

---

## Why it should be adopted, per audience

Different people need different sentences. These are the true ones.

**For the person and the family.** Say it once. It goes with you when the
provider changes, when you move, when you are admitted at 2am. Nobody can take
it away because nobody is holding it but you and the people you chose.

**For the professional.** Read what matters in a minute, before you knock on the
door. One tap to record that you read it — cheap for you, and the thing families
currently have no way of knowing. You are not depositing anything into anybody's
database, so nothing new is being asked of your organisation's information
governance.

**For the provider organisation.** No contract, no procurement, no data
processing agreement to negotiate, no new liability. Your staff read something
the person shared with them; you are not the controller of it.

**For the commissioner or system.** Nothing to buy, nothing to host, nothing to
decommission in five years. The failure mode of every previous attempt —
the operator gives up — cannot happen here, because there is no operator to
give up.

---

## The questions that will come

Answered honestly, including where the answer is uncomfortable. Most of these
are worked through in [hard-questions.md](hard-questions.md),
[how-it-works.md](how-it-works.md) and the [DPIA](DPIA.md); this is the short
form to have ready in a room.

**1. Who is liable when it goes wrong?**
Nobody is the operator, so there is no central controller. Each person's device
holds a record they were given access to by the person or their proxy — legally
closer to being told something than to being given a database. This is the
question that most needs a qualified opinion, and the DPIA says so rather than
claiming it is settled. *Do not bluff this one.*

**2. How can it be GDPR-compliant with no controller?**
The circle is closed and membership is by invitation; each member holds the copy
they were given. The honest tension is erasure: entries sit on other members'
devices, so "delete everything" is not a single action anyone can take. The
answers built are removal that every copy honours, encryption with a new key on
removal so that nothing written afterwards can be read on that device, and the
removed person's own app deleting its copy. None of that is erasure and the DPIA
refuses to call it erasure.

**3. What if the person loses their phone, or dies?**
A circle survives in the other members' devices. There is a named successor
mechanism: the holder names who takes over and who checks on her, with a waiting
period visible to everybody and the ability to say "I'm still here".

**4. How does a professional know the record is genuine?**
Every entry is signed by the key that wrote it, and who may write what is
checked by every device independently. What the app never claims is that
somebody is a district nurse — an acknowledgement proves a key asserted it had
read a version, and the word "claimed" appears every time a role is shown.

**5. What stops a coercive family member controlling the record?**
Nothing in software can fix a coercive relationship, and the project says so.
What exists: the person or their proxy decides who is in, removal is honoured by
every copy, and a circle can be moved away from somebody entirely. The deeper
answer is that this is a safeguarding matter, not a feature.

**6. Why Holochain and not a blockchain, or Solid?**
No global ledger is wanted: nobody needs a permanent public record of a
vulnerable person's care preferences. Each circle is its own small network, and
the data lives only on the devices of the people in it. Solid is the nearest
neighbour and needs a pod host; that host is the operator this design removes.

**7. Who pays for it, and what is the business model?**
There is no hosting cost to recover, which removes the usual model and is the
point. Sustainable futures: grant-funded development of a public good;
organisations paying for support and assurance rather than for access; nothing
at all, because the code is Apache-2.0 and outlives its author.

**8. What happens when you lose interest?**
The code is open, the standard is public, and there is no server to switch off.
Circles that exist keep working with no maintenance, because there is nothing in
the middle to maintain. That is a genuinely different answer from every hosted
product's.

**8a. Where is the evidence that this problem is real?**
The LeDeR 2024 report (King's College London for NHS England, July 2026) found
that adults with a learning disability died at a median age of 62.8 in 2024,
19.0 years below the general adult population, and that 39.0% of their deaths
were avoidable against 21.1% in the general population. The two most common
problems in care were **"problems with organisational systems" (40.5%)** and
delays in care or treatment (39.8%); among autistic adults without a learning
disability the report lists **"fragmented and poorly coordinated care across
services"**. That is the gap Hearth is aimed at. It is not evidence that Hearth
closes it — see the note in [EHIA.md](EHIA.md), which quotes the report exactly
and is careful about what it does and does not show.

**9. Has a real person ever used it?**
Not yet, and the project does not pretend otherwise —
[what-is-proven.md](what-is-proven.md) has a section titled "Nobody outside the
project has installed a release". What *is* proven: two machines sharing over the
internet, sharing through the internet being cut mid-session, and — with a local
discovery build — two machines finding each other with no internet at all, in
about a minute, on 19 September 2026.

**10. Is it accessible? This is a population with cognitive impairment.**
The target is WCAG 2.2 AA and the DTAC framework. Photos, sound and video are
built for exactly this reason: somebody who cannot read a screen can hear their
daughter's voice describing what matters. Plain language is treated as a
requirement, not a nicety.

**11. Is it safe, clinically?**
It is deliberately **not a clinical record** and must never be described as one.
Scoping to About Me keeps DCB0129 clinical safety certification and clinician
indemnity out of scope. Adding clinical fields drags all of that back in, which
is why the scope is load-bearing rather than modest.

**12. Has it been security reviewed?**
No. There is an adversarial test suite of 94 tests that exists specifically to
break the project's own claims, and every claim in the documents is written to
be falsifiable. A real review by somebody qualified is on the list of things to
ask help for.

---

## Who to show it to

Ordered by what it costs you to ask. Everything marked ⚠️ was **not** verified
today — check before relying on it.

### Outside the UK

**1. NLnet — Restack.** Already the plan: deadline **3 November 2026**,
€5k–50k, individuals eligible, everything open source. See
[funding.md](funding.md), which was checked at source. This remains the only
route you can walk into alone, today, anywhere.

**2. Internet Society Foundation — Research Grant Programme.** Global,
**grants made directly to individuals** as principal investigator, up to
US$200,000. The 2026 round ran 7 April – 22 May 2026, so the next one is
roughly spring 2027 ([programme page](https://www.isocfoundation.org/grant-programme/research-grant-programme/),
read 21 September 2026). It is *research*-framed, so it would need a question
rather than a product — "can a record with no operator cross organisational
boundaries in practice?" is a real research question, and probably wants a
university partner. Worth preparing for months ahead rather than discovering
late.

**3. MyData Global (Finland).** Not a funder — a community and an annual
conference of exactly the people who would understand this in one sentence, and
the network that funders in this space read. Cheap to join, and the single best
place to find a collaborator outside the UK. ⚠️ not checked today.

**4. SolidLab / athumi / the Flanders programme (Belgium).** Not a funder, and
arguably a competitor — but they are the most serious people in the world
working on the same problem, and they have already hit the wall Hearth is
designed around. A conversation with them is worth more than most grant
applications, and "I have built the no-operator version, will you tell me why I
am wrong?" is a good opening. ⚠️ approach unverified.

**5. Sovereign Tech Agency (Germany) — probably not, and here is why.** Their
programmes fund **maintainers of critical open-source infrastructure** and
standards work; the Fellowship closed 6 April 2026 and the Standards programme
in May 2026 ([programmes](https://www.sovereign.tech/programs), read 21
September 2026). Hearth is an application on somebody else's infrastructure, so
it does not fit today. It could fit later if the *pattern* — person-held records
with no operator — becomes standards work.

**6. SIDN fonds (Netherlands) — only with a Dutch partner.** Grants to natural
persons, but the project must have impact on **Dutch society**
([Pioneers](https://www.sidnfonds.nl/funding/pioneers), read 21 September 2026).
Real if a Dutch care organisation ever partners; not otherwise.

**7. EU Horizon / EIT Health and the EHDS-adjacent calls.** Serious money,
consortium-shaped: you would be a technical partner on somebody else's bid, not
a lead applicant. Worth knowing exists, not worth chasing alone. ⚠️ not checked
today.

**8. The Holochain ecosystem — Holo Ventures and the Holochain Foundation.**
The one place where "no operator" needs no explaining, and where a working
application is scarce and valuable. Already noted in funding.md.

### Inside the UK

The detail is in [funding.md](funding.md) and is not repeated here. The short
version: **NLnet is the only one open to you as an individual**, and the
highest-value unblocking move is becoming a constituted thing — Cwmpas, free,
Welsh — which opens the Lottery, SBRI and most of the rest.

Beyond money, the people who would *understand* it fastest:

- **PRSB** — it is their standard. Somebody building a working implementation
  of About Me is of direct interest to them, and they are reachable.
- **Digital Care Hub** and **National Care Forum** — the sector's own digital
  bodies, closer to providers than NHS England is.
- **TLAP (Think Local Act Personal)** — personalised care is their whole remit,
  and "I Statements" are their language.
- **Alzheimer's Society** and **Dementia UK** — the population most obviously
  served, and both run innovation programmes. ⚠️ not checked today.
- **england.dtac@nhs.net** — a real address that will answer how a no-operator
  system should be assessed. No introduction needed.

### The one you can do this week

**Let somebody outside the project install a release and use it.** It is the
single largest gap in [what-is-proven.md](what-is-proven.md), it costs nothing,
and it changes every conversation above from "I have built" to "people are
using".

---

## What not to claim

Getting caught overstating once would cost more than every one of these is
worth.

- Not "NHS-approved", not "conformant to About Me" — it implements a subset,
  and conformance is an assessed process with a quality mark it has not been
  through.
- Not "secure" without qualification. It has had no security review.
- Not "GDPR-compliant". The DPIA is honest about the erasure tension and says a
  qualified opinion is needed.
- Not "in use". It is not, yet.
- Not "encrypted", in a released build — the encryption is built and tested on a
  branch as of 21 September 2026, and is not in a release until the batch ships.
