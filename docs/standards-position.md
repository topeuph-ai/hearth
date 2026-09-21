# Standards and regulations: what applies, and where Hearth stands

**Hearth — a person-held About Me record.**
Self-assessment, 21 September 2026. Published openly at
`github.com/topeuph-ai/hearth`.

**Nothing here is certified.** Two of the items below are not certifications at
all, one is assessed as out of scope, and one may not be capable of applying.
This document says which is which.

---

## Summary

| Standard or regulation | Status | Evidence | What is missing |
| --- | --- | --- | --- |
| **PRSB About Me** information standard | Implemented as a **subset**. Conformance **not claimed** | `docs/standard-and-gap.md`, field by field | Conformance is an assessed process with a quality mark. Not been through it |
| **UK GDPR / Data Protection Act 2018** | Not a certification. A **DPIA is published** | `docs/DPIA.md` | A qualified opinion on lawful basis, controllership with no operator, and the limits of erasure |
| **WCAG 2.2 AA** | **Target**, not audited | Plain-language interface; photographs, speech and video for people who cannot read a screen; `docs/EHIA.md` | An independent accessibility audit with people who have cognitive impairment |
| **DCB0129** clinical safety | Assessed **out of scope** | `docs/medical-device-determination.md`; About Me holds no clinical content | Confirmation from somebody qualified that the assessment is right |
| **Data Security and Protection Toolkit** | **Applicability unclear** | — | The DSPT is completed by an organisation with staff, policies and a SIRO. There is no organisation and no central store. Whether it can apply is an open question |
| **UK MDR 2002** | Assessed as **not a medical device** | `docs/medical-device-determination.md` | Nothing, unless that determination is wrong |

---

## PRSB About Me

Hearth implements the About Me standard's headings — what matters to me, people
who matter, how to communicate with me, my wellness, please do and please do
not, how to support me, also worth knowing — together with the standard's
acknowledgement that somebody may have supported the person to write it. Coded
values are supported beside each section, and the standard's multimedia
provision is met with photographs, recorded speech and short video.

`docs/standard-and-gap.md` sets out, field by field, what is implemented and
what is not.

**Conformance is not claimed anywhere in this project's materials**, and would
be wrong to claim: it is an assessed process with a quality mark, and this has
not been through it.

## UK GDPR and the Data Protection Act 2018

A DPIA has been written and published. Its substance:

- Membership of a circle is closed and by invitation; each member holds the copy
  they were given.
- There is no central database and no operator holding data on anybody's behalf.
- **Erasure is the honest difficulty.** Entries sit on other members' devices, so
  "delete everything" is not a single action anyone can take. Removal that every
  copy honours, the removed person's own app deleting its copy, and a new
  encryption key on removal all reduce the risk substantially — and none of them
  is erasure. The DPIA says so.

## WCAG 2.2 AA

The target, because much of the intended population lives with cognitive
impairment, sensory impairment or fatigue. Design decisions made for that reason
include plain language throughout, no reminders or nagging, and media so that a
person who cannot read a screen can still be heard in their own voice.

**It has not been audited**, so conformance is not claimed. An audit is one of
two things the project cannot do for itself and cannot currently pay for.

## DCB0129 clinical safety

Assessed as out of scope because Hearth is not a clinical record: it holds no
medications, diagnoses, observations or care plans, and the About Me standard
describes itself the same way. That boundary is deliberate and load-bearing —
adding clinical content would bring clinical safety certification, clinician
indemnity and the heaviest data-protection questions back into scope.

Confirmation of this assessment has been requested. If it is wrong, it is much
cheaper to know now.

## Data Security and Protection Toolkit

Listed because somebody will ask for it, and because the honest answer is
interesting rather than evasive.

The DSPT is completed by an organisation: it asks about staff training, named
information-governance roles, policies, incident reporting and the security of
systems the organisation operates. Hearth has no organisation, no staff, and no
systems that anybody operates — the record is held by the people in the circle
on their own devices.

Whether the DSPT can apply at all to software with no operator is a question
this project cannot answer for itself, and is part of the same request made
about DTAC: a decision on how systems of this shape should be assessed.

---

## What this document is not

It is not a claim of compliance with anything. It is a statement of position,
written by the developer, published so that anybody can check it and say where
it is wrong.
