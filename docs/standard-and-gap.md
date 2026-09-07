# Two questions: are we following the standard, and are we filling a gap?

**Status: research. Written by Claude on 2026-09-07 from primary sources, to be
checked and owned by the project lead.** Every claim below is either quoted
from a source or marked as a judgement. Where a source could not be
cross-checked, it says so.

---

# Part one — the standard

Checked against the PRSB **About Me** JSON, `About-Me-v2.01-May25.json`, and
the entry on the NHS Standards Directory.

## Status of the standard itself

- **Version 2.0.1**, published May 2025. Listed as **Active** — "stable,
  maintained and have been approved, assured or endorsed for use by qualified
  bodies."
- Developed by PRSB, commissioned by NHS England. **From 1 January 2026 it is
  owned and managed by NHS England**, under the Open Government Licence v3.0.
- **31 systems** are listed as conformant.

## What the standard contains, and what Hearth has

The standard has seven sections. Every element in it is optional (`0..1`).

**Updated 2026-09-07: all seven sections and "Supported to write this by" are
now implemented.** The table below is the state after that change.

| PRSB section | Hearth |
| --- | --- |
| What is most important to me | yes — `what_matters_to_me` |
| People who are important to me | yes — `people_who_matter` |
| How I communicate and how to communicate with me | yes — `how_to_communicate_with_me` |
| My wellness | yes — `my_wellness` |
| Please do and please do not | yes — `please_do_and_please_do_not` |
| How and when to support me | yes — `how_to_support_me` |
| Also worth knowing about me | yes — `also_worth_knowing` |

Each section also allows a **coded value** and **multi-media** (filename, MIME
type, file, URL). Hearth has neither: it is free text only, and that is the
remaining distance between this and the full standard.

Two metadata elements sit outside the sections:

| PRSB element | Hearth |
| --- | --- |
| Supported to write this by | yes — `supported_to_write_this_by` |
| Date last updated | partly — every version is a signed action with a timestamp, so it exists but is never shown |

## What this means

**Hearth is a strict subset, not a divergence.** Everything it stores maps onto
a section of the standard, and nothing it stores falls outside it. Because
every element is optional, an unanswered section is a real answer rather than a
hole.

**It is not conformant, and should not be described as such.** Conformance is
an assessed process with a quality mark, run by PRSB, which this has not been
through. "Scoped to the About Me standard" is accurate. "Conformant with About
Me" is not.

**Different labels are fine.** The standard "does not specify how it should be
presented, recorded, or managed across different systems" (NHS Standards
Directory), so "What matters to me" for "What is most important to me" is a
presentation choice, not a deviation.

### "Supported to write this by" — done, and why it mattered

The standard has a field for **who helped write this**. Hearth already has the
concept — holder is not always the subject, which is load-bearing here, because
a daughter or case manager holds the circle where the person cannot — and it
records who wrote every version cryptographically. But the record itself never
says, in the person's own record, that somebody else wrote it on their behalf.

That was the standard describing something the design already believed. It is
now a field on the record and a line under it: *"Supported to write this by Pam
Smythe."* Whoever sets a circle up for somebody else has already given their
name on the way in, so it arrives filled in and can be changed. Self-declared,
like everything else here, and never inferred.

**What remains missing is coded values and multi-media**, which the standard
allows against every section. Hearth is free text only. That is the honest
description of the remaining distance.

### The tension worth confronting

The NHS Standards Directory entry states that About Me information "will be
shared via a link so professionals access the master version, not old versions
copied into local systems."

**Hearth does the opposite by construction**: every member's device holds a
readable copy. That is the point — no master, no operator, works when the
network does not — but it is the model that guidance argues against, and it
compounds the known unresolved problem that entry contents are not yet
encrypted.

*Caveat: that sentence comes from one summarised fetch of the NHS Standards
Directory page and was not corroborated by the PRSB page.* **Read the directory
entry directly before quoting it anywhere.**

---

# Part two — is there a gap?

## The honest answer: not the one it might look like

**About Me is not a gap. It is a well-populated market.** Of the 31 conformant
systems, the great majority are provider-side care management software:

> Access Care Planning, AnchorApp, Birdie, CACI Certa, Carebeans, Careberry,
> Care Docs, Care Vision, CareLineLive, Cura, Dom Portal, everyLIFE, Fusion
> eCare, Health Connect, iStaffrota, KareInn, Leecare Platinum, Log My Care,
> Nourish, OneAdvanced, One Touch, Person Centred Software, Qwikify, Roundsys,
> Storii, Sumo Optimus

plus shared-care-record vendors (**Graphnet**, **Orion Health**) and one NHS
trust's local Rio implementation.

Anyone claiming "nobody has built About Me" is wrong, and would be corrected in
the first meeting.

## Where the gap actually is

**Every one of those systems has an operator, and almost all are bought by the
organisation, not the person.**

- A care company's system holds About Me *inside that company*. The person
  moves provider, or is admitted to hospital, and it sits in a system the next
  organisation does not use.
- Shared care records exist to cross that boundary, and social care is the part
  that keeps not working. The trade press in January 2026 was still describing
  it as the "stubbornly hard" element — the same sentence has been written
  about it for years.
- The standard's own guidance points at a *link to a master version*, which
  presupposes somebody hosting the master. **That is the operator problem
  restated, not solved.**

**The one genuine person-held comparator is RIX Multi Me / the RIX Wiki**, and
it has its own section below, because studying it properly changed one of this
project's arguments.

## The nearest neighbour: RIX Multi Me / the RIX Wiki

Studied on 2026-09-07. **This is the closest existing thing to Hearth, and it
is a good deal more than "a hosted competitor".** It deserves to be understood
rather than waved at.

**What it is.** A private multimedia portfolio — a "go-to place for all the
information about an individual" — built on a mind-map interface, holding text,
files, links, audio and video. Developed out of years of research at **RIX
Research & Media, University of East London**, and managed and distributed by
**Multi Me Ltd**, a private company incorporated in 2010. Conformant with the
About Me standard. An NHS Innovation Accelerator alumnus. ISO 27001 certified
in 2024, with a named Data Protection Officer.

**It is genuinely person-held.** The person decides what to share and with
whom. It is not a provider system with a family portal bolted on.

**It has a safeguarding idea this project did not have.** A user's **Buddy can
veto** what the user shares — a real answer to a real problem, somebody who
might be persuaded to over-share.

*Addressed 2026-09-07.* A circle may now appoint a **second yes**: somebody
already in it, without whose signature nobody else may join. Deliberately the
opposite way round to a veto — nothing happens unless they actively agree,
because a veto must arrive in time to stop something already moving whereas a
signature that was never given stops nothing. It is enforced by the membrane
rather than by the interface, so the holder cannot waive it under pressure,
which is the situation it exists for. Appointing one re-forms the circle, which
is cheap because circles are clones.

**What is still all-or-nothing is reading.** Everybody in a circle reads
everything in it; there is no per-item sharing to veto, so RIX's control has no
equivalent at that level and may not need one. Worth revisiting only if
somebody asks for it.

**Who pays.** Wikis are commonly paid for by a **local authority, school or
other organisation** — piloted in Herefordshire, Oxfordshire and Abingdon &
Witney College under the name "WikiMe". An individual "My Wiki" licence is
reported at **£72 a year including VAT**.

*That figure comes from a search summary of their help desk, which could not be
fetched directly — three of their pages returned 403 or redirect loops. Verify
it before quoting it anywhere.*

### The part that costs this project an argument

**When the organisation stops paying, the person can move onto an individual
subscription and keep their Wiki.** That is a considered answer to the funding
cliff, and it is a good one.

So **"we are the only one that survives being defunded" is not a claim this
project can make.** RIX already survives that, for about the price of a
streaming service.

The honest claim is narrower and structural:

> RIX survives its *commissioner* going away. It does not survive its
> *operator* going away. Multi Me Ltd is a private company; ISO 27001 does not
> help if it stops trading, and neither does a DPO. Every Wiki depends on that
> company continuing to exist and continuing to host.

That is precisely the HealthVault shape — a well-run, well-intentioned,
well-certified hosted service whose users' data depended on a corporate
decision they had no say in. **The differentiator is not "person-held". It is
"no company to fail."**

### What could not be established, and should be asked directly

1. Where the data is hosted, and **who the data controller is** — the person,
   Multi Me Ltd, or the commissioning authority.
2. What happens to a Wiki if **Multi Me Ltd ceases trading**. Is there an
   export? An escrow? A published continuity commitment?
3. Whether professionals in *other* organisations can read a Wiki in practice
   during, say, an unplanned hospital admission — or whether it works mainly
   within the commissioning authority's own settings.

Question 3 is the important one. RIX is strongest in learning-disability
advocacy, with a heavy multimedia emphasis Hearth does not have at all. **If a
Wiki turns out to work well across a hospital boundary, the gap this project
aims at is smaller than assumed and should be restated. If it does not, that
boundary is the gap, and it is the thing to demonstrate.**

Contact for all three: info@multime.com. This is a reasonable thing to ask as
somebody building in the open, and their answers are worth more than any amount
of further searching.

## What is genuinely unoccupied

Nothing on that list of 31 is **operator-free**. The claim is not "an About Me
app" — that is taken thirty-one times over — it is:

> An About Me that no organisation holds, that keeps working when the
> organisation that introduced it stops existing, and that a person carries
> between providers who will never share a system.

That is a real and unoccupied position. It is also the position HealthVault
occupied commercially and vacated in 2019, taking the data with it, which is
the argument for the operator being absent by design rather than trustworthy by
promise.

## The counter-argument, stated fairly

**Nobody buys "no operator".** Those 31 systems exist because organisations
purchase them, are supported by them, and can sue them. Removing the operator
also removes the funder, the support desk, the training budget and the
accountable body — and a commissioner's first question will be "who do I call
when it goes wrong?"

The answer cannot be "nobody". This is the hardest question the project faces,
it is a business-model question rather than a technical one, and no line of
code answers it.

## What would strengthen the case, in order

1. ~~Verify RIX Multi Me properly.~~ Done 2026-09-07 — see the section above.
   Three questions remain that only they can answer; email info@multime.com.
2. **Ask `england.dtac@nhs.net` how a system with no operator should be
   assessed.** DTAC assumes a supplier organisation to answer it. That question
   is publishable on its own, and the answer shapes everything.
3. ~~Add "Supported to write this by."~~ Done 2026-09-07, along with the three
   missing sections.
4. **Do not claim conformance.** Say "scoped to About Me v2.0.1" and be exact
   about which four of the seven sections are implemented.

## Sources

- `About-Me-v2.01-May25.json` (PRSB), fetched 2026-09-07
- NHS Standards Directory, About Me: https://standards.nhs.uk/published-standards/about-me
- PRSB About Me standard: https://theprsb.org/standards/aboutme/
- PRSB conformant partners: https://theprsb.org/partners/conformant-partners/
- RIX and the About Me standard (PRSB news): https://theprsb.org/news/rix-on-a-mission-to-transform-peoples-care-experience-with-the-about-me-standard/
- RIX Multi Me, NHS Innovation Accelerator: https://covid19.nhsaccelerator.com/innovation/rix-multi-mi/
- Multi Me, about us: https://www.multime.com/about-us
- Multi Me, about My Wiki: https://www.multime.com/about-my-wiki
- RIX Software: https://rixsoftware.org/

**Pages that would not load and are worth retrying**: rixinclusiveresearch.org
(403), rixsoftware.org/about-rix-software (403), the RIX Wiki help desk
(redirect loop), and digitalhealth.net's January 2026 piece on the social care
data barrier (403).
