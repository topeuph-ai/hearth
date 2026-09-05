# Data Protection Impact Assessment — About Me

**Status: DRAFT for review.** Written by Claude, to be checked and owned by the project lead. Anywhere marked **[DECISION]** is a judgement only a person can make.

---

## In one paragraph

About Me lets a person record what matters to them, how to communicate with them and how to support them, and share it with the people who care for them — family and professionals from different organisations. It runs peer-to-peer with **no server and no operator**. Each participant's device holds their own copy. Nothing is uploaded to anyone.

---

## 1. Why a DPIA at all

Not because the law necessarily demands one yet. Because the honest answer to "who is responsible for this data?" is unusual, and writing it down before anyone asks is worth more than being able to answer on the spot.

Two things do point towards needing one under UK GDPR Article 35: the people described may be vulnerable, and the architecture is novel. Both are listed triggers.

---

## 2. What the system actually does

**Information held**

- A person's name, what matters to them, how to communicate with them, how to support them, and who is important to them.
- A record that a named professional read a specific version.

**Information deliberately NOT held**

- No diagnoses. No medications. No care plan. No clinical notes.

This exclusion is a design decision, not an oversight. It tracks the PRSB *About Me* standard, which states it is not a person-held clinical record. It keeps the system outside the definition of a medical device and outside clinical risk.

**Is it special category data?** Probably yes in places. "How to support me" may reveal a disability or condition by implication even without naming one. This assessment assumes **it is** special category data, which is the safer assumption. **[DECISION: confirm rather than assume.]**

**How it moves**

Directly between the devices of circle members. There is no cloud service, no database, and no company holding a copy. Members' devices hold copies of each other's entries so the circle still works when someone is offline.

**Who is in it**

Only people the person, or whoever acts for them, has admitted. Membership is not open.

---

## 3. Who is responsible for what

This is the section everything else depends on.

| Who | Role | Responsible for |
|---|---|---|
| The person, or whoever acts for them | Controller | Their own entries; deciding who joins |
| A family carer | Controller (possibly exempt — see below) | What they write |
| A professional's employing organisation | Controller | That professional's contribution and acknowledgements |
| A circle member whose device stores a copy | Processor | Storing and passing on data faithfully; deciding nothing |
| **The project / the developer** | **Neither** | **Nothing. Determines no purpose, sees no data, operates no service** |

**Reasoning, and the published sources it rests on:**

- NHS guidance on identifying controllers gives, as a worked example, *a group of health and care providers inputting into a shared care record* — these are **joint controllers**. That is this arrangement.
- NHS developer guidance (*Step 3: Determine if you are a data controller or processor*) turns the question on **who decides what data is processed and how**. The developer here decides nothing: no collection, no retention, no access, no sharing.
- ICO guidance on distributed ledger technologies distinguishes participants who **create** entries (likely controllers) from those who merely **hold and pass on** entries (likely processors). Applied here: writing into a circle is a controller act; storing a neighbour's copy is a processor act.

**Caveat to state openly rather than hide:** the ICO guidance is written for distributed ledgers, and this is not a ledger. The reasoning is functional — it asks what role a participant plays, not what the technology is called — so it transfers. But the distinction should be raised by us, not discovered by an assessor.

**Open question for NHS England (`england.dtac@nhs.net`):** the DTAC form assumes a supplier who hosts a service and holds data. There is no box for a system with no operator. How should it be completed?

**Possible exemption, unverified.** A family member keeping notes about their own relative may fall under the household-purposes exemption and outside UK GDPR entirely. Once professionals join, the circle is probably no longer purely domestic. **[DECISION: ask, do not assume. Do not rely on this.]**

---

## 4. Is it necessary and proportionate

**The need.** A person's care spans organisations that do not share systems. What matters to them is lost at every handover. This is not a claim about efficiency; it is about a person being met as themselves by a stranger.

**Why not a conventional service?** Because that is the thing that has repeatedly failed. It requires an operator to hold the data, and no organisation will accept that role on behalf of the others. Microsoft HealthVault took the role and then shut down in 2019, taking the data with it. Removing the operator is the point, not a technical preference.

**Data minimisation.** Five fields. No clinical data. No analytics, no telemetry, no tracking of any kind — there is no operator to receive it.

**Lawful basis.** **[DECISION]** Likely explicit consent for the person's own entries; professionals' contributions rest on their own employer's existing basis. Needs a qualified view.

---

## 5. Risks

| Risk | Likelihood | Severity | Notes |
|---|---|---|---|
| **Data unavailable when needed** — small circle, all devices asleep, information needed at 3am | Medium | High | The most serious risk. See below |
| A member's device is lost or stolen and readable | Medium | High | Device encryption; keep the local store encrypted at rest |
| A member holding a copy reads content not meant for them | Medium | Medium | Encrypt contents to circle members at application level |
| Someone is admitted who should not have been | Low | High | Only the person or their proxy can admit |
| Data cannot be erased once distributed to peers | Medium | Medium | Genuine tension with the right to erasure — see below |
| A person lacks capacity to decide who joins | Medium | High | Proxy arrangements needed. **[DECISION]** |
| Content is wrong or out of date and someone relies on it | Medium | Medium | Acknowledgements record which version was read |

### The availability risk

If every device in a six-person circle is off, the circle is unreachable. Holochain's redundancy assumes enough peers are online, and a small circle on sleeping phones is the hardest case for that assumption. The documentation does not address it.

This is unresolved, and it is the question that decides whether this becomes a product. Adding an always-on device per circle would fix it — but that device would be able to read everything unless contents are encrypted to circle members first.

**Nothing should be built on top of this until it is answered.**

### The erasure tension

Entries are held on other members' devices, so "delete everything" is not a single action anyone can take. This is a real limitation and must not be glossed over. Two partial mitigations: hold as little as possible in the first place, and encrypt contents so that withdrawing access renders remaining copies unreadable.

**[DECISION: get a qualified opinion. Do not claim this is solved.]**

---

## 6. Measures

**Already built in**

- Clinical data excluded by design
- Only the original author may revise their own entry — enforced independently by every participant's device, not by a server
- Nobody can acknowledge their own record
- Closed membership by invitation
- No operator, therefore no central store to breach and no company whose failure destroys the data

**To do, in order**

1. Resolve the availability question with Holochain's developers
2. Encrypt entry contents to circle members, so a device holding a copy cannot read it
3. Build to WCAG 2.2 AA from the first screen — these users include people with cognitive impairment and exhausted carers
4. Write down how proxy decision-making works where someone lacks capacity
5. Ask NHS England how a no-operator system should be assessed under DTAC
6. Get a qualified view on lawful basis and on erasure

**Not needed yet:** Cyber Essentials and clinical safety certification are procurement gates. They apply when an organisation is buying something. That is years away, and neither costs anything today.

---

## 7. Sign-off

Unsigned. This is a working document for a prototype, not an assurance document for a live service. It will need a qualified reviewer before it is relied on.

**Sources:** NHS England — *Identifying controllers and processors in health and care*; NHS AI and Digital Regulations Service — *Step 3: Determine if you are a data controller or processor*; ICO — *Distributed ledger technologies*; PRSB — *About Me* standard.
