# Holochain and health: what other people have already published

**Status: research, 2026-09-08.** The 2021/2022 paper has now been **read in
full**, not summarised. The 2025 one has not, and carries a warning — see the
end.

---

## The short version

The paper exists. It is better than remembered — a real IEEE journal article
with UK research-council funding behind it — and **there is exactly one idea in
it worth taking.**

It is not the architecture, which pushes storage and computation to cloud
servers and would undo the whole point of this project. It is **capability
tokens**: a way for the person to hand a specific professional access to a
specific part of their record, without that professional being in the circle at
all.

---

## The paper

**Thinking Out of the Blocks: Holochain for Distributed Security in IoT
Healthcare** — Shakila Zaman (BRAC University, Dhaka), Muhammad R. A. Khandaker
(Heriot-Watt), Risala T. Khan (Jahangirnagar), Faisal Tariq (Glasgow),
Kai-Kit Wong (UCL).

**Published in IEEE Access**, volume 10, 30 March 2022 —
[10.1109/ACCESS.2022.3163580](https://doi.org/10.1109/ACCESS.2022.3163580). The
arXiv preprint is from March 2021, which is the version usually found first.

**Supported by EPSRC grant EP/T015985/1.** That is UK research-council money,
which makes this considerably more citable than a preprint.

It is **analysis, not implementation.** They model resource use, latency and
storage growth against blockchain approaches. Nothing was built.

[IEEE Access](https://doi.org/10.1109/ACCESS.2022.3163580) ·
[arXiv preprint](https://arxiv.org/abs/2103.01322) ·
[Glasgow copy (PDF)](https://eprints.gla.ac.uk/272795/1/272795.pdf)

---

## What is worth taking: capability tokens, and an outer ring

Their sharing model, in their own words:

> "When a doctor wants to monitor his patient remotely using the IoT network,
> (s)he requests a set of health reports. The patient generates a capability
> grant or token for the particular reports or medical data that (s)he wants to
> share and stores the new transaction or stories as the new entity of the
> holochain. Moreover, the patient also shares the hash of the grant entry with
> the authorized doctor that will be used as a capability token. On the other
> hand, the doctor preserves this token as the new entry on his private source
> chain and uses it whenever needed to access that particular patient's data."

**This is a topology Hearth does not have.** Everything here works one way: you
are admitted through the membrane into a circle, and once inside you hold a copy
of everything in it. There is no such thing as being shown one section. There is
no such thing as somebody reading the record without joining.

Capability tokens are a different shape. The data is **not published to a shared
space at all** — it stays on the person's own chain, and a holder of the token
calls in and asks for it. Which would give this project four things it has
written down as gaps and has no answer to:

- **Per-section sharing.** A district nurse gets "how and when to support me".
  She does not get everything.
- **Reading without joining.** No invitation, no membrane, no copy of the record
  living on a stranger's laptop afterwards.
- **Revocation that actually works.** A capability grant can be withdrawn, and
  the next call is refused. See [`what-is-proven.md`](what-is-proven.md), where
  "nothing is revocable in the ordinary sense" is listed as a gap. This does not
  fix that for people already in a circle — nothing does — but it means not
  everybody has to be in one.
- **A shape for the professional case**, which is the one this project keeps
  running into and has no room for: somebody who needs to read this once, does
  not want an app, and should not become a permanent member of a family's
  private circle.

### Why this is not simply a good idea to adopt tomorrow

**It breaks the offline promise.** A capability-gated call needs the person's
device to be reachable. The whole reason a circle replicates to every member is
so the record survives a flat battery, a lost phone, or a person in an
ambulance — see the offline design principle in the README. A token-based outer
ring works only while somebody is online, which is precisely not the moment this
is most needed.

So it is an **addition, not a replacement**: an outer ring around the circle, for
people who should see a slice and nothing more, while the circle itself stays as
it is for the people who need it to work when nothing is working.

**And this project has already rejected capability grants once, correctly.**
From the coordinator zome:

> "They were once on this project's build order as the route to revocation, which
> was wrong — they govern who may call into *this* cell, not what somebody
> already holds."

That rejection stands. It was about revoking what a member has already got, and
capabilities cannot do that. This is a different use: governing what somebody
**who was never given a copy** may ask for. The distinction is the whole idea and
should be written down clearly before any of it is built, or the same wrong turn
gets taken twice in opposite directions.

---

## What is not worth taking

**The cloud layer.** They propose moving storage and computation to cloud servers
because IoT sensors cannot manage either:

> "it is important to shift all these capability-demanding activities to the
> cloud servers"

That is the correct answer to their problem and the wrong answer to this one. It
reintroduces exactly the operator this project exists without. Their agents are
thermometers; ours are people with phones and laptops, which are not
resource-constrained in the way a sensor is.

**Holo Fuel.** Their model assumes the currency for hosting and accounting. Not
relevant, and a reviewer who sees cryptocurrency near a care record will stop
reading.

**The reputation "experience matrix".** Agents scoring each other's behaviour and
gossiping the scores. Interesting, largely aspirational in the paper, and a bad
fit for a circle of six people who already know each other.

**Their future work.** Cryptocurrency processing, ML threat detection, load
balancing at IoT scale. None of it is this project's problem.

---

## ⚠️ On the 2025 paper: unresolved

**Among the DLTs: Holochain for the Security of IoT Distributed Networks — A
Review and Conceptual Framework** — Ismail, Mehannaoui, Hunde, Reza. *Sensors*,
21 June 2025. It proposes "HoloSec" and reports building a three-node Holochain
healthcare network in Rust.

**The project lead has read it and judges it to be AI-generated**, produced from
a long context in which unrelated subjects — drones and a board game — bled into
the text.

**A targeted check of the PubMed Central full text for exactly that did not find
it**, turning up only one drone-related citation that fits its context. That
check was a single automated pass over one version of the article and is weak
evidence; a person reading the PDF is stronger evidence than a machine scanning
the HTML.

**So this is unresolved, and the practical answer does not depend on resolving
it: do not cite it.** A citation is a claim that you have read something and
vouch for it. Citing a paper whose provenance is in question, in an application
that is otherwise scrupulous about what it can and cannot prove, risks the one
thing this project is trading on.

The 2022 IEEE Access paper is peer-reviewed, EPSRC-funded, readable end to end,
and enough on its own to establish that Holochain for health data is taken
seriously by people who are not us. **Cite that one.**

The claim built on the 2025 paper — that "the right to be forgotten while
preserving decentralisation" is named as open research — should be treated as
**withdrawn until it can be supported from a source that has been read**. The
2022 paper does not make it.

---

## Still worth doing

- **Search for person-centred rather than IoT-centred Holochain health work.**
  This search was aimed at the paper being remembered and found the IoT thread.
  The absence of person-centred work above is a result of that aim, not proof
  that none exists.
- **Design the outer ring properly before building it**, including what it costs
  when the person's device is off.
