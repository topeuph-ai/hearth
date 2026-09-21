# Medical device determination and intended purpose

**Hearth — a person-held About Me record.**
Determination made 21 September 2026 by the developer. Published openly at
`github.com/topeuph-ai/hearth`.

---

## Determination

**Hearth is not a medical device**, and no intended purpose is claimed under the
UK Medical Device Regulations 2002.

It is a communication tool: it holds and shows what one person wants the people
looking after them to know.

---

## Why

Under UK MDR 2002, a medical device is an instrument, apparatus or software
intended by its manufacturer to be used for a medical purpose — diagnosis,
prevention, monitoring, treatment or alleviation of disease, injury or
disability, investigation or modification of anatomy or a physiological process,
or control of conception.

Hearth does none of these:

| Function | Does Hearth do it? |
| --- | --- |
| Diagnose, or aid diagnosis | No |
| Predict or calculate risk | No |
| Monitor a condition, treatment or physiological parameter | No. It takes no measurements and connects to no sensors |
| Provide or control treatment or therapy | No |
| Interpret, score, summarise, triage or advise on any information | **No. This is the decisive one** |
| Alter what it is given | No. It shows what a person wrote, unchanged |

It holds free text, photographs, recorded speech and short video, written by the
person whose record it is — or by a family member or case manager acting for
them — and displays them to the people that person has chosen. It contains no
clinical content, no medications, no diagnoses and no care plan, by design: the
PRSB **About Me** standard it implements says the same of itself.

---

## Intended purpose statement

Provided for completeness, using the four elements the MHRA asks for, so that
the determination can be checked rather than taken on trust.

**Structure and function.** Software running on a person's own device. It stores
a record written by or on behalf of the person — what matters to them, how they
communicate, what to please do and please not do — and shares it with people the
person invites, whose devices each hold a copy. It displays that record
unchanged. It performs no measurement, calculation, interpretation or advice.

**Intended population.** People supported by more than one organisation:
typically older people, people living with dementia, people with a learning
disability, autistic people, people with neurological conditions, and people
with multiple long-term conditions.

**Intended users.** The person themselves; a family member or case manager
acting for them; and the care workers, nurses and other professionals the person
invites.

**Intended use environment.** The person's own home, care settings, and any
setting where somebody unfamiliar is looking after them — including hospital
admission and urgent care, where the record may be shown by a family member.

---

## What would change this determination

Stated in advance, so the line is visible rather than argued about later.

Hearth would need reassessment **before release**, not after, if it began to:

- interpret, score, summarise or flag anything in the record;
- give advice, prompts or recommendations based on what is recorded;
- hold clinical content — medications, diagnoses, observations, care plans;
- take any measurement, or connect to any device that does;
- be described, labelled or promoted for any medical purpose.

Two things that do **not** cross that line, and are already built: adding
photographs, recorded speech and video, which are the person's own words in
another form; and, in future, speech-to-text on the device, which is a way of
writing rather than a way of interpreting.

The MHRA determines intended purpose from labelling, instructions for use and
promotional materials. All of this project's materials are public, and are
written to keep that boundary: nothing describes Hearth as clinical, diagnostic
or as supporting clinical decisions.

---

## Related documents

- `docs/standard-and-gap.md` — field by field against the PRSB About Me standard
- `docs/DPIA.md` — data protection impact assessment
- `docs/EHIA.md` — equality and health inequalities impact assessment
- `docs/hard-questions.md` — the questions a sceptical reviewer would ask

This determination has not been reviewed by a regulatory professional. It is the
developer's assessment, published so that it can be challenged.
