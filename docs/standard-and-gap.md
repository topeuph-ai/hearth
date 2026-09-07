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

| PRSB section | Hearth |
| --- | --- |
| What is most important to me | yes — `what_matters_to_me` |
| People who are important to me | yes — `people_who_matter` |
| How I communicate and how to communicate with me | yes — `how_to_communicate_with_me` |
| My wellness | no |
| Please do and please do not | no |
| How and when to support me | partly — `how_to_support_me`, shown as "How to help me feel at ease" |
| Also worth knowing about me | no |

Each section also allows a **coded value** and **multi-media** (filename, MIME
type, file, URL). Hearth has neither: it is free text only.

Two metadata elements sit outside the sections:

| PRSB element | Hearth |
| --- | --- |
| Supported to write this by | no — see below, this one matters |
| Date last updated | partly — every version is a signed action with a timestamp, so it exists but is never shown |

## What this means

**Hearth is a strict subset, not a divergence.** Everything it stores maps onto
a section of the standard, and nothing it stores falls outside it. Because
every element is optional, holding four of seven does not contradict the data
model.

**It is not conformant, and should not be described as such.** Conformance is
an assessed process with a quality mark, run by PRSB, which this has not been
through. "Scoped to the About Me standard" is accurate. "Conformant with About
Me" is not.

**Different labels are fine.** The standard "does not specify how it should be
presented, recorded, or managed across different systems" (NHS Standards
Directory), so "What matters to me" for "What is most important to me" is a
presentation choice, not a deviation.

### The one gap worth closing: "Supported to write this by"

The standard has a field for **who helped write this**. Hearth already has the
concept — holder is not always the subject, which is load-bearing here, because
a daughter or case manager holds the circle where the person cannot — and it
records who wrote every version cryptographically. But the record itself never
says, in the person's own record, that somebody else wrote it on their behalf.

That is the standard describing something the design already believes. It is a
small change, and it is the only place where Hearth is silent and About Me is
not.

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

**The one genuine person-held comparator is RIX Multi Me / the RIX Wiki** —
conformant with About Me, built with and for the learning disability community,
an NHS Innovation Accelerator alumnus. It is the closest thing to this that
exists and it deserves study rather than dismissal. The likely difference is
that it is a hosted service with an organisation behind it, but that needs
verifying rather than assuming.

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

1. **Verify RIX Multi Me properly.** Hosted or not? Who is the data controller?
   What happens when funding stops? It is the nearest neighbour and the most
   useful thing to understand.
2. **Ask `england.dtac@nhs.net` how a system with no operator should be
   assessed.** DTAC assumes a supplier organisation to answer it. That question
   is publishable on its own, and the answer shapes everything.
3. **Add "Supported to write this by."** Small, and it closes the only place
   where Hearth is silent and the standard is not.
4. **Do not claim conformance.** Say "scoped to About Me v2.0.1" and be exact
   about which four of the seven sections are implemented.

## Sources

- `About-Me-v2.01-May25.json` (PRSB), fetched 2026-09-07
- NHS Standards Directory, About Me: https://standards.nhs.uk/published-standards/about-me
- PRSB About Me standard: https://theprsb.org/standards/aboutme/
- PRSB conformant partners: https://theprsb.org/partners/conformant-partners/
- RIX and the About Me standard (PRSB news): https://theprsb.org/news/rix-on-a-mission-to-transform-peoples-care-experience-with-the-about-me-standard/
- RIX Multi Me, NHS Innovation Accelerator: https://covid19.nhsaccelerator.com/innovation/rix-multi-mi/
