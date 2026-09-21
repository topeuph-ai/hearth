# Equality and Health Inequalities Impact Assessment (EHIA)

**Hearth — a person-held About Me record, shared across organisations, with no
operator.**

**Status: a self-assessment by the developer, written 21 September 2026.** It
follows the structure NHS England publishes for an EHIA. It has not been
through any decision-maker, any organisation, or anybody from the populations
described in it — which is itself one of the findings, and is recorded as such
in sections 5 and 9.

Where evidence is asserted, the source is named. Where there is no evidence,
this document says so rather than filling the space. The most important finding
is in section 4 and it is **adverse**: Hearth could widen the inequality it is
meant to narrow, for the people least able to use a device.

---

## 1. Name of the proposal

Hearth: an application that lets a person keep and share what they want the
people looking after them to know — what matters to them, how they communicate,
what to please do and please not do — across the organisations supporting them.
Implements the PRSB **About Me** information standard. Open source, no charge,
and with no central database and no operating organisation.

---

## 2. Brief summary of the proposal

A person, or a family member or case manager acting for them, writes their own
account of themselves and chooses who may see it: relatives, a district nurse, a
support worker, a care agency. Each person in that circle holds a copy on their
own device; when the record is updated, every copy updates. Anybody who reads it
can record that they have, so the family can see it has been read. Members may
suggest additions, which only the person decides whether to accept.

There is no central database and no company holding the record. Nothing is
deposited into an organisation's system; a professional reads what the person
has shared with them. There is consequently nothing to procure, host or
decommission, and no supplier whose closure takes the records away.

It is **not a clinical record** and holds no clinical content.

---

## 3. Potential impact for protected characteristic groups

| Protected characteristic | Main potential positive or adverse impact | Main recommendation |
| --- | --- | --- |
| **Age**: older people; middle years; early years; children and young people | **Positive, and strongly weighted to older people**, who are the likeliest to be supported by several organisations at once and to be admitted somewhere unfamiliar. **Adverse**: older people are also over-represented among those who do not use a smartphone or computer, or who rely on somebody else to do so. An older person without a device depends entirely on a relative or worker holding their circle for them, which is a weaker form of "held by the person". Children and young people are not the current focus; the standard implemented is the adult About Me. | Support a proxy holder explicitly and visibly, so it is always clear on screen whose account this is and who is keeping it. Never require the person themselves to own a device. |
| **Disability**: physical, sensory and learning impairment; mental health condition; long-term conditions | **The largest positive impact, and the reason the project exists.** People with a learning disability, autistic people, people living with dementia and people with communication impairment are those whose needs are most often unknown to whoever is in front of them. Photographs, recorded speech and short video are built in so that a person who cannot read a screen, or cannot write, can still be heard in their own voice. **Adverse**: the interface has not been through an independent accessibility audit; WCAG 2.2 AA is the target, not a demonstrated fact. Sensory impairment (in particular deafblindness) is not specifically provided for beyond standard accessibility. | Commission an independent accessibility review against WCAG 2.2 AA with attention to cognitive impairment. Do not claim conformance until it has been done. Treat the media features as core, not as an extra. |
| **Gender reassignment and/or people who identify as transgender** | **Potentially positive and specifically so.** The record is written in the person's own words and shown unchanged, so a person's name, pronouns and how they wish to be addressed travel with them and do not have to be re-established with each new worker or ward. Being repeatedly misnamed by people providing intimate care is a known harm. **Adverse**: the record is visible to everybody in the circle, so a person who is out to some of the people around them and not to others cannot presently share selectively. | Note the "everyone sees everything" limitation as a real constraint (it is documented in hard-questions.md). Consider whether any future partial-visibility mechanism is worth its complexity, with this group's needs as a primary test case. |
| **Marriage and civil partnership** | No identified positive or adverse impact. | Not applicable. |
| **Pregnancy and maternity** | No specific impact identified. Maternity is not a target setting, and About Me is not a maternity record. | Not applicable. |
| **Race and ethnicity** | **Adverse, and currently unmitigated: the application exists only in English.** A person whose first language is not English, or whose family speak another language at home, cannot write their account in their own words in their own language, which is precisely what the record is for. Cultural and religious requirements around food, washing, modesty and family involvement are exactly the sort of thing About Me should carry, and are among the things most often unknown to a new worker. | Treat translation and multilingual entry as an equity requirement rather than an internationalisation nicety. Recorded speech partly mitigates in the meantime: a person may speak in their own language even where they cannot type in it. |
| **Religion and belief** | **Positive.** Requirements about diet, washing, prayer, modesty, and who may be present during personal care are the kind of thing that is carried badly between organisations and matters daily. The record is free text written by the person, so nothing constrains what may be said. | None specific. Ensure examples used in the interface and in any training material include religious and cultural requirements, so people know the record is a place for them. |
| **Sex** | No differential impact identified in the software. Note that unpaid carers, who are heavily affected by whether this works, are disproportionately women (see section 4). | Not applicable to the software; relevant to how benefits are described. |
| **Sexual orientation** | As for gender reassignment: the record travels in the person's own words, which can prevent a partner being repeatedly mistaken for a friend or relative. **Adverse**: the same all-or-nothing visibility applies — what is said to one member of the circle is said to all. | As above. |

---

## 4. Potential impact for people who experience health inequalities

| Group | Main potential positive or adverse impact | Main recommendation |
| --- | --- | --- |
| **Looked after children and young people** | Not a current focus: the standard implemented is the adult About Me. The pattern — a child known by several organisations and by none of them jointly, whose history is retold by professionals rather than by them — is recognisably the same, and the children and young people's version of About Me exists. | Do not claim benefit for this group until the children and young people's standard has actually been implemented and tested. |
| **Carers: unpaid, family members** | **Among the largest positive impacts.** Repeating the same information to every new worker is one of the most commonly described burdens of unpaid caring. A written record that travels removes some of that, and the acknowledgement feature lets a family see that what they wrote has actually been read, which is currently invisible to them. **Adverse**: where the cared-for person cannot hold the record, the work of keeping it falls on the carer, who is often exhausted and often the person with least time. | Keep the writing burden minimal and never nag. The app deliberately has no reminders, no completeness scores and no "your record is out of date" messages. Hold that line. |
| **Homeless people** | **Adverse and significant.** Device ownership, charging, connectivity and device loss are all harder or impossible. A record held on a device is a poor fit for somebody with nowhere to keep one, and this group has high need — they are repeatedly assessed by services that do not share information. | Do not present Hearth as a solution for this group in its current form. Any future work would need somebody else in the circle — a hostel worker, an outreach nurse — to hold it, which changes who controls it and needs thinking about on its own terms. |
| **People in contact with the criminal justice system** | Neutral to adverse. Devices are restricted or prohibited in custodial settings. The same proxy-holder question arises as for homelessness. | None at present. Do not claim benefit. |
| **People with addictions or substance misuse issues** | Neutral. No specific mechanism either way, beyond the general benefit of not having to retell one's history to each new service. | None at present. |
| **People or families on a low income** | **Mixed, and the mix matters.** Positive: Hearth is free, has no licence or subscription, and runs on a device the household already owns — unlike any hosted alternative, whose cost is ultimately borne by somebody. Adverse: "a device the household already owns" is an assumption, and a shared or old phone, a pay-as-you-go connection or a data cap are real constraints. | Keep it free and keep it light. Measure what the app actually costs in data, and publish the figure. Never require the newest device. |
| **People with poor literacy or health literacy** | **Mixed, and deliberately worked on.** The record is text-first, which disadvantages exactly the people it is meant to serve. Against that: it is written in the person's own words rather than in professional language, and photographs, recorded speech and short video are built in so that somebody can say what matters rather than write it. "What this says in words" is suggested beside media, never required. | Continue to treat plain language as a requirement. Consider on-device speech-to-text so that speaking is a way of writing, not only a way of recording. Keep explanatory text short — it is currently longer than it should be, which is a known fault. |
| **People living in deprived areas** | **This is the central adverse finding.** Digital exclusion is concentrated in deprived areas, and a record that lives on devices is least available to the households least likely to have a spare, working, connected device and the confidence to use it. **Hearth could therefore widen the very inequality it is intended to narrow**, by making sure that people who are already better served have their needs known, while those who are not, do not. | State this openly in every conversation about adoption. Hearth should be offered **alongside** paper, never as a replacement for it. Any service adopting it must have an answer for the people who cannot use it, and should be asked for that answer before adopting. |
| **People living in remote, rural and island locations** | **Mixed, and partly positive in an unusual way.** Most shared-record systems fail without connectivity. Hearth does not require the internet to be working: two devices in the same place can find each other and share directly, demonstrated in a field test on 19 September 2026 in which two machines with no internet connection found each other in about a minute. For a rural care worker with no signal in a farmhouse, that is the difference between a record and a blank screen. Adverse: reaching somebody in another town still needs connectivity at some point. | Prioritise the local-discovery capability for release rather than treating it as an experiment. It is an equity feature, not a technical curiosity. |
| **Refugees, asylum seekers and people experiencing modern slavery** | **Adverse, on two counts.** English-only, as above. And the record's contents are visible to everybody in the circle, which is a poor fit for somebody whose safety depends on information not travelling. More positively, there is no central database to be required to disclose, and no operator who can be compelled to hand over records, which is a genuine protection for people who have reason to fear data sharing. | Be honest that the absence of an operator cuts both ways: nobody can be compelled to disclose the record, and nobody can help if a device is taken. Do not promote to this group without advice from organisations who work with them. |
| **Other: people subject to coercion or abuse within their own circle** | **Adverse, and not solvable in software.** Whoever holds a circle controls who is in it. Where the holder is a family member who is also the source of harm, the tool could strengthen their control over the account given of the person. Mitigations exist — a second person can be required to agree to who joins, removal is honoured by every copy, and a circle can be moved away from somebody entirely — but none of them detects coercion. | Say so plainly, in the documentation and in any training. This is a safeguarding matter with a software component, not a software problem. Seek advice from safeguarding practitioners before any real-world use with people at risk. |

---

## 5. Engagement and consultation

**a. Have any engagement or consultative activities been undertaken?**

**No.**

**b. Detail**

None. Nobody from any of the populations described above has been consulted,
and nobody outside the project has used the software. The developer is a music
teacher without professional contacts in health or social care, and the project
has proceeded by building and publishing rather than by engagement, because
engagement was not available to it.

This is the single largest weakness of this assessment, and of the project. Every
statement of benefit above is reasoned rather than observed.

---

## 6. Key sources of evidence, and gaps

| Evidence type | Key sources used | Key gaps |
| --- | --- | --- |
| Published evidence | The Core20PLUS5 approach (NHS England), which names people with a learning disability, autistic people and people with multiple long-term conditions among expected PLUS populations. Reviews of the deaths of people with a learning disability (the LeDeR programme) have repeatedly reported communication needs being misinterpreted and care poorly co-ordinated between services. **The LeDeR findings here are taken from secondary summaries and must be read at source before being quoted in any submission.** | No figures of any kind on: how often an About Me document is lost between settings; how often it is read; whether having one changes what happens to the person. Digital exclusion figures by deprivation have not been read at source and are not quoted here for that reason. |
| Consultation and involvement findings | None. See section 5. | Everything. |
| Research | None conducted. No academic partner. | No evaluation design exists. |
| Participant or expert knowledge | None. The developer has no professional background in health or social care and states this openly throughout the project's documentation. | No clinical, safeguarding, accessibility or information-governance expertise has been applied to this assessment. |
| The project's own testing | 94 automated adversarial tests; two-machine testing over the internet, through an internet outage, and with no internet at all (19 September 2026). See `what-is-proven.md`. | All of it is technical. None of it involves a person who needs this. |

---

## 7. Public Sector Equality Duty

| | Tackling discrimination | Advancing equality of opportunity | Fostering good relations |
| --- | --- | --- | --- |
| The proposal will support | | | |
| The proposal **may** support | **x** | **x** | |
| Uncertain whether the proposal will support | | | **x** |

**May support**, not will. The mechanism is plausible and unevidenced: a record
that carries a person's communication needs to whoever is in front of them
should reduce the disadvantage experienced by people whose needs are otherwise
guessed at. Nothing has been measured.

---

## 8. Reducing health inequalities faced by patients

| | Reducing inequalities in access to health care | Reducing inequalities in health outcomes |
| --- | --- | --- |
| The proposal will support | | |
| The proposal **may** support | | **x** |
| **Uncertain** if the proposal will support | **x** | |

**Access is marked uncertain deliberately.** A digital tool can widen access
inequality even while improving outcomes for those who can use it. Until
Hearth has been tried by people in the groups named in section 4 — and
specifically by people with no device, poor connectivity or low digital
confidence — an honest assessment cannot claim it narrows access inequality,
and might have to conclude the opposite.

---

## 9. Outstanding key issues, in priority order

| Key issue or question | What would address it |
| --- | --- |
| **1. Does this widen digital exclusion?** If the people who most need their needs known are the least able to hold a device, adoption could benefit the better-off and leave everyone else where they were. | Trying it with people in the groups named — including through a proxy holder — and asking honestly whether it helped or excluded. Advice from organisations working on digital inclusion. A firm position that Hearth sits alongside paper rather than replacing it. |
| **2. Nobody from the affected population has been consulted or has used it.** Every benefit claimed is reasoned, not observed. | A small pilot with a care provider, hospice or third sector organisation: a handful of people for a few weeks. This is the project's stated first ask of the NHS Innovation Service. |
| **3. Coercion and capacity.** Whoever holds a circle controls the account given of a person who may be unable to object. | Advice from safeguarding practitioners, and from organisations representing people who lack capacity, before any use with people at risk. |

---

## 10. Summary assessment

Hearth is aimed squarely at a population named in the Core20PLUS5 PLUS groups —
people with a learning disability, autistic people, and people with multiple
long-term conditions — together with people living with dementia and unpaid
carers. For those people the intended benefit is real and specific: what they
need the people around them to know travels with them, in their own words,
across organisations that do not share systems, and they can see that it has
been read.

Three features exist directly because of equity considerations rather than as
product decisions: **photographs, recorded speech and video** so that somebody
who cannot read or write a screen can still be heard; **no cost, no licence and
no hosting**, so that no household or provider is priced out; and **working
without the internet**, so that a rural worker with no signal is not left with
a blank screen.

Against that stand three adverse findings, which are not incidental:

1. **Digital exclusion.** A record held on devices is least available to the
   households least likely to have one. Hearth may widen access inequality even
   where it improves outcomes for the people who can use it. It should be
   offered alongside paper and never as a replacement.
2. **English only.** The record's whole purpose is a person's own words; today
   it cannot hold them in the person's own language.
3. **Coercion within the circle.** The person who holds the circle controls it,
   and software cannot tell care from control.

And one finding about this assessment itself: **it is reasoning, not evidence.**
Nobody in any of these groups has used the software, and nobody has been asked.
Until that changes, every positive judgement here should be read as a hypothesis
that the project is asking for help to test.

---

## 11. Contact details

| | |
| --- | --- |
| Author | Ceri John, developer of Hearth |
| Organisation | None — an individual, not a company or charity |
| Project | `topeuph-ai/hearth`, open source, Apache-2.0 |
| Date agreed | Not agreed by any decision-making body; published as a self-assessment |
| Date published | 21 September 2026 |
