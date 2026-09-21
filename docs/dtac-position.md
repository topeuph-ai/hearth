# DTAC: where Hearth stands, and the question DTAC cannot currently answer

**Hearth — a person-held About Me record.**
Self-assessment, 21 September 2026. Published openly at
`github.com/topeuph-ai/hearth`.

**No DTAC assessment has been completed.** This document says honestly where the
project stands against each part of it, and raises a structural question that
needs answering before any assessment would mean anything.

---

## The structural question, first

**DTAC assumes there is a supplier.** It asks an organisation to attest to its
clinical safety officer, its data protection registration, its security
policies, its Data Security and Protection Toolkit submission, and its support
arrangements.

Hearth has no supplier and no central database. The record lives only on the
devices of the people in the circle; there is no server, no hosted service, no
account, and no organisation processing anybody's data. That is not a gap in the
product — it is the product. It is why there is nothing to procure, nothing to
host, and nothing whose closure takes the records away.

So several DTAC questions cannot be answered as written, and answering them
falsely would be worse than saying so:

- **Data Security and Protection Toolkit** — completed by an organisation with
  staff, policies and a Senior Information Risk Owner. There is no organisation.
- **Service management and support** — there is no helpdesk, because there is no
  service to manage.
- **Supplier assurance** — there is no supplier.

**What the project is asking for is a ruling, not an exemption.** If a system
with no operator is to be usable in health and care, somebody has to decide how
it should be assessed. That decision is worth more to this project than any
other support, and it would apply to anything else built this way.

---

## Section by section

### 1. Clinical safety

**Position: out of scope, and stated in writing.** Hearth is not a clinical
record. It implements the PRSB About Me standard only and holds no medications,
diagnoses, observations or care plans. A separate document sets out the medical
device determination and intended purpose.

**Gap:** that assessment is the developer's own. Confirmation that DCB0129 does
not apply — or a clear statement that it does — is one of the things requested
from the NHS Innovation Service.

### 2. Data protection

**Position: a DPIA has been written and published** (`docs/DPIA.md`). It covers
closed membership by invitation, the absence of a central store, and what
happens on removal.

It is honest about the hardest question rather than claiming to have solved it:
**erasure**. Entries sit on other members' devices, so "delete everything" is not
a single action anyone can take. What exists instead is removal that every copy
honours, the removed person's own app deleting its copy, and encryption with a
new key on removal so that what the circle writes afterwards cannot be read on
that device. None of that is erasure, and the DPIA refuses to call it erasure.

**Gap:** a qualified opinion on lawful basis, on controllership where there is
no operator, and on erasure. Requested from the Service.

### 3. Technical security

**Position: no independent review has been done.** This is the largest gap and
is not minimised here.

What exists: the code is open source and public, so it can be inspected by
anybody. Every entry is signed by its author. Each circle is a separate network
whose identity is fixed by the compiled rules, so circles cannot see one
another. Every device checks the rules independently, so a modified copy of the
app cannot write what the rules forbid — this is verified by 94 automated
adversarial tests which exist specifically to break the project's own claims.
Encryption of the record, with a new key whenever somebody is removed, is built
and tested and awaits release.

**Gap:** an independent security review. Requested from the Service; it is one
of two things the project cannot do for itself and cannot currently pay for.

### 4. Interoperability

**Position: built on an open national standard.** Hearth implements the PRSB
About Me standard as a subset, documented field by field in
`docs/standard-and-gap.md`. Conformance is not claimed: that is an assessed
process with a quality mark the project has not been through.

There is no integration with any other system, deliberately. Nothing is
deposited into an organisation's record, which is the reason no data sharing
agreement or procurement is required.

### 5. Usability and accessibility

**Position: WCAG 2.2 AA is the target; it has not been audited.**

What exists: the interface is built in plain language for people with cognitive
impairment and for exhausted carers. Photographs, recorded speech and short
video are built in so that somebody who cannot read a screen can still be heard
in their own voice. An equality and health inequalities impact assessment has
been published (`docs/EHIA.md`), which identifies digital exclusion as a real
limit on who can take this up, and English-only content as an unmitigated gap.

**Gap:** an independent accessibility audit, with people who have cognitive
impairment. Requested from the Service.

---

## Summary

| DTAC section | Status |
| --- | --- |
| Clinical safety | Assessed as out of scope. Confirmation requested |
| Data protection | DPIA published. Qualified opinion requested |
| Technical security | **No independent review. Largest gap** |
| Interoperability | PRSB About Me implemented as a subset. Conformance not claimed |
| Usability and accessibility | WCAG 2.2 AA targeted, not audited. EHIA published |
| Supplier assurance, DSPT, service management | **Cannot be answered as written: there is no supplier** |

Nothing in this document should be read as a claim to meet DTAC. It is a
statement of where the project genuinely stands, written so that somebody
qualified can tell it what to do next.
