# Hearth

### A care record that belongs to the person it is about, and works across organisations that will never share a computer system.

**Free. Open source. No company in the middle.**
Download: **github.com/topeuph-ai/hearth/releases/latest**

---

## What it is

Hearth holds what one person wants the people looking after them to know: what
matters to them, how they communicate best, what to please do and what to
please not do. It follows the PRSB **About Me** information standard.

The person — or a family member or case manager acting for them — writes it in
their own words and chooses who may see it. A daughter. A district nurse. A
support worker. The care agency. Each of them sees it in their own copy of the
app, and when it changes, everyone's copy changes.

It is not a clinical record. It is the person's own account of themselves, and
it travels with them.

---

## The gap it fills

Somebody with dementia, a learning disability or a long-term condition is seen
by many different people. Each writes things down in their own organisation's
system, and those systems do not talk to each other. So the person explains
themselves again, every time, to everybody — how to talk to them, what
frightens them, what helps.

The standard for what to write down already exists. **The problem was never
what to write. It was where to put it.**

Every previous attempt built a shared place to put the record — and whoever
builds that place becomes **the operator**: the organisation legally
responsible for holding sensitive information about a vulnerable person,
contributed by several other organisations, and still there in twenty years.
Almost nobody wants that job. Microsoft tried it with HealthVault and closed it
in 2019, taking the data with it.

**So Hearth has no operator.** The record lives on the devices of the people in
the circle. There is nothing in the middle to buy, host, procure, assure or
switch off — and nothing whose closure takes the record away.

---

## What makes it different

| | Where the record lives | What happens when it ends |
| --- | --- | --- |
| **Paper** — "This is me", hospital passports | In a drawer, or a bag | Lost, or out of date, and only ever in one place |
| **Provider systems** — care records, case management | In the organisation | You change provider, and it does not follow you |
| **Patient portals and personal health records** | On a platform with an owner | Paid for, procured, and only as permanent as its owner |
| **Personal data pods** | With whoever hosts the pod — in Flanders, a state-owned company created to host pods for 6.5 million citizens | Depends on that institution continuing |
| **Hearth** | **Only on the devices of the people in the circle** | **Nothing to end. No operator to fail** |

Hearth keeps paper's one great property — nobody owns it — and adds the things
paper cannot do: being in several places at once, being updated by a daughter
in another town, showing who has read it, and stopping being readable by
somebody who has been removed.

---

## What you can do with it

**Write the record, in the person's own words.** Seven sections from the About
Me standard — what matters to me, people who matter, how to communicate with
me, my wellness, please do and please do not, how to support me, also worth
knowing — plus who supported the person to write it.

**Say it out loud instead of typing it.** Add photographs, recorded speech and
short video beside any section. Somebody who cannot read a screen can still
hear a familiar voice explaining what matters. Video is kept to two minutes and
shrunk automatically, so an ordinary phone recording works.

**Invite people simply.** Share the circle's address, or let them scan a code
from your screen. They ask to join; **you decide**. The address lets somebody
ask — it does not let them in.

**Ask a second person to agree.** Optionally, no one joins unless two people
agree: the person and somebody they trust. A safeguard against a circle being
opened up by one person having a bad day.

**Let people suggest things.** Anybody in the circle can offer something — "she
likes the radio on in the afternoon" — and only the person decides whether it
goes in. Nothing offered is ever silently thrown away.

**See that it has been read.** Anyone reading the record can record that they
have, with the role they say they hold. Families currently have no way of
knowing whether anybody read anything.

**Remove somebody.** Every copy of the app honours it: they drop out of the
lists, what they write afterwards is not shown, and their own app takes the
circle off their device.

**Plan for what happens next.** The person can name a successor to hold the
circle, and somebody else to check on them first — with a waiting period
everybody can see, and the ability to say "I'm still here".

**Use it when the internet is down.** Two devices in the same place can share
directly. In a field test on 19 September 2026, two machines with no internet
connection at all found each other in about a minute.

---

## How to get it

1. Go to **github.com/topeuph-ai/hearth/releases/latest**
2. Download **uk.topeuph.hearth-…-setup.exe** and run it.
3. Windows will say *"Windows protected your PC"* — because the installer is
   not code-signed, which costs money this project does not have. Click **More
   info**, then **Run anyway**.
4. Open Hearth, make a circle, and share its address with somebody.

**Windows only at the moment.** The Mac and Linux builds are assembled from the
same released file; see the repository for how.

Nothing to register for. No account, no sign-up, no email address, no licence
key.

---

## How it works, briefly

Each circle is its own small private network, and its identity is fixed by the
rules every device checks. Two circles cannot see each other — that is
mathematics, not a setting somebody could get wrong.

Every entry is signed by the person who wrote it, and **every device checks the
rules independently**: only the person may write their own record, only they
may accept a suggestion, nobody may acknowledge their own record, and nobody
may put words in another member's mouth. A modified copy of the app cannot
break those rules, because everybody else's copy checks.

Devices do use public servers to **find** each other across the internet. Those
servers carry addresses, never records.

---

## Where it has got to

**Working and tested.** An installable Windows app, released publicly. 94
automated tests exist specifically to break the project's own claims. Two
machines have shared a record over the internet, kept sharing through the
internet being cut mid-session, and — with a local-discovery build — found each
other with no internet at all.

**Coming in the next release.** The record locked with a key that changes
whenever somebody is removed, so that what the circle writes afterwards cannot
be read on their device even by modified software. Built and tested; it is
waiting on the upgrade path that carries existing circles across to the new
version.

**Not there yet, said plainly.** Nobody outside the project has used it. There
has been no independent security review and no accessibility audit. It exists
only in English. And because it lives on devices, it is least available to
people without one — which is why it should sit alongside paper rather than
replace it.

All of that is written up in the open, including the parts that are
unflattering:

- **what-is-proven.md** — what is tested, what is built but unwatched, what is not built
- **hard-questions.md** — the questions a sceptical reviewer would ask, answered
- **DPIA.md** — data protection, including the limits of erasure
- **EHIA.md** — equality and health inequalities, including who this could leave out

---

## Who it is for

People supported by more than one organisation, and the people around them:
older people, people living with dementia, people with a learning disability,
autistic people, people with neurological conditions, and anybody whose
communication needs are not obvious to a stranger.

And for the professionals who arrive not knowing them: a minute's reading
before you knock on the door.

---

**github.com/topeuph-ai/hearth** — free, open source, Apache-2.0.
Built in Wales by one person, and published in full so that anybody can check
every claim on this page.
