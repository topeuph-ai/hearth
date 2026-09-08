# Holochain and health: what other people have already published

**Status: research, 2026-09-08.** Found by searching for a paper the project lead
half-remembered. Abstracts and article summaries were read; **the full PDFs have
not been read line by line**, and the performance figures below should be checked
in the papers themselves before being quoted anywhere that matters.

---

## The short version

The paper exists, it is from **2021 rather than 2020**, and **the thread was not
abandoned** — it produced peer-reviewed work as recently as June 2025, including
somebody actually building a Holochain healthcare network and measuring it.

That is better news than "we can take over an abandoned idea". It means this
project is not alone in the claim that Holochain is a reasonable substrate for
health data, and it does not have to make that argument from first principles to
a funder or a reviewer.

---

## 1. The paper you were thinking of

**Thinking Out of the Blocks: Holochain for Distributed Security in IoT
Healthcare** — Shakila Zaman, Muhammad R. A. Khandaker, Risala T. Khan, Faisal
Tariq, Kai-Kit Wong. Submitted to arXiv **1 March 2021**.

Copies are hosted by **Heriot-Watt University** and the **University of
Glasgow**, so this is mainstream UK academic work rather than a crypto white
paper.

The argument is the one you would expect, made carefully: blockchain's storage
and computation grow with the network, which is fatal for resource-constrained
devices, and Holochain's agent-centric model does not have that property. They
propose a framework and do performance and security analysis against blockchain
approaches.

**What it is not.** It treats healthcare as *IoT telemetry* — devices,
monitoring, sensor data — and its subject is *security of the network*. It is
not about a person's own account of themselves, not about who is allowed into
somebody's circle, and not about care crossing organisational boundaries.

[arXiv:2103.01322](https://arxiv.org/abs/2103.01322) ·
[Glasgow copy (PDF)](https://eprints.gla.ac.uk/272795/1/272795.pdf)

## 2. And it kept going — including a real implementation

**Among the DLTs: Holochain for the Security of IoT Distributed Networks — A
Review and Conceptual Framework** — Shereen Ismail (Merit Network / University of
Michigan), Raouf Mehannaoui (Batna 2, Algeria), Eden Teshome Hunde (Vrije
Universiteit Brussel / Jimma), Hassan Reza (University of North Dakota).
Published in ***Sensors*, 21 June 2025**. Peer-reviewed and open access.

They propose **HoloSec**, and — the part that matters here — **they built
something**: a Holochain-based healthcare IoT network with three simulated nodes
monitoring heart rate, blood pressure and glucose, with custom DNA and zome logic
written in Rust. Then they measured it against an Ethereum-based comparison.

Reported figures, **to be verified in the paper before use**: 50 ms to publish
and 30 ms to retrieve, against 200 ms and 100 ms for blockchain; 20 transactions
per second on a single node against 10; and at ten nodes, Holochain holding
15 TPS while the blockchain comparison fell to 3.

[Sensors article](https://doi.org/10.3390/s25133864) ·
[PubMed Central full text](https://pmc.ncbi.nlm.nih.gov/articles/PMC12251913/)

---

## What this is worth to Hearth

### It answers a question we would otherwise have to argue alone

"Is Holochain a serious choice for health data, or is this a hobbyist picking
something unusual?" That question will be asked, politely, by every reviewer.
Being able to point at a 2021 UK university paper and a 2025 peer-reviewed
journal article with a working implementation is a much better answer than
anything this project could write about itself.

### It confirms our hardest open problem is genuinely hard

Their future-work section names, as an unsolved research problem:

> "Developing mechanisms to support the right to be forgotten while preserving
> decentralization principles."

That is **exactly** the thing [`what-is-proven.md`](what-is-proven.md) lists
under "Nothing is revocable in the ordinary sense". Four academics across four
universities name it as open research. So our honest admission is not a
confession of incompetence — it is the field's open problem, and saying so
plainly puts this project on the same page as the literature rather than behind
it.

Their other named gaps — tooling and SDK immaturity, DHT performance under load,
decentralised authentication — are all things this project has hit and written
up independently. See [`latency.md`](latency.md).

### But it does not do what we do, and that is the opening

Both papers are about **securing device data on a network**. Hearth is about
**a person's own words, and who they let read them**. The unit is a relationship,
not a sensor. Nothing found in this search addresses:

- a person controlling the membrane of their own circle
- informal carers, family and advocates as first-class participants
- conformance to a published care standard that professionals already recognise
- an interface any of this is meant to be used through by somebody who is not a
  developer

**So this is not an abandoned idea to take over. It is an adjacent, live,
credible research thread that establishes the substrate — and leaves the person
entirely unaddressed.** That is a considerably stronger position than inheriting
somebody's abandoned work, and it should be said that way round.

---

## Worth doing

- **Read both papers properly.** This note is built from abstracts and article
  summaries, and the performance figures are second-hand.
- **Consider writing to the 2025 authors.** They built a Holochain health
  prototype and named the right-to-be-forgotten problem as open. This project
  has hit the same wall from the other direction, and has a working application
  rather than a simulation. That is a real exchange, not a favour being asked.
- **Search once more, properly, for person-centred rather than IoT-centred
  Holochain health work.** This search was aimed at the paper being remembered
  and found the IoT thread; the absence of person-centred work above is a result
  of that aim, not proof that none exists.
