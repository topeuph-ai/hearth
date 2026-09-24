# Holochain's roadmap, as it touches Hearth

**Read 24 September 2026** from Holochain's public project board,
[github.com/orgs/holochain/projects/11](https://github.com/orgs/holochain/projects/11).
The board groups work by release and gives **no dates**, so what follows is an
order, not a calendar. Issue numbers are in `holochain/holochain` unless said
otherwise. Check the board again before relying on any of it: plans move.

---

## Holochain 0.7.1 — a security release, next

The first test build (0.7.1-rc.1) came out on 2 September; one more is planned
before the release itself (#5932, #5933).

What it fixes that matters here:

- **#5781, marked critical.** Receipts saying "I have stored this" are accepted
  without checking the signature on them. A device can be fooled into thinking
  what it wrote has spread when it has not — for Hearth, a change to the record
  that never reaches the rest of the circle.
- **#5994.** An entry is not checked against its own hash before it is stored,
  so a made-up entry can get into a device's store.
- **#5995.** Changes how the permission records called capability grants are
  looked up. **Passes are built on those**, so passes must be tried again on
  0.7.1.
- Probably the `rustls` flaw the dependency check found (see
  [threat-model.md](threat-model.md)); not confirmed.

**What Hearth does, decided 24 September 2026:** move to the 0.7.1 *program*
and keep Hearth's own code exactly as it is. The fixes are in the program, not
in the toolkit Hearth's code is built with, so nothing needs rebuilding and no
circle is split off. Then a two-machine check of passes. Rebuilding Hearth's
code against 0.7.1 would change every circle's identity for no gain — the same
reason Lightningrod Labs gave for keeping their Moss tools on 0.7.0.

Rebuild when there is a reason to, such as 0.8 below — and use that moment to
test "Carry this circle to the new version" on two machines, which a rebuild
makes necessary anyway.

The LAN build stays on 0.7.0 until Lightningrod Labs rebuild their version on
0.7.1. That is theirs to do.

## Holochain 0.8 — "stabilize the HDK and support app upgrades"

The release aimed squarely at the problem [upgrades.md](upgrades.md) describes.

- **Official code updates** — `update_app` (#4570), a manifest refactor
  (#4404), and a callback that runs after an update (#5971). The proper
  replacement for desktop patch 03, which swaps new code into every circle by
  hand on each launch.
- **Capability tokens and remote calls across a code update** (#4911, #4912).
  Passes are exactly this: a capability grant and a call from another device.
  Watch these when moving to 0.8.
- **"What is needed for DNA migration"** (#4396), being scoped. The same
  problem as carrying a circle across to new rules, which Hearth solves today
  in the app by moving the circle.

## Holochain 0.9

Forty-odd items; the ones that touch Hearth:

- **Sharding** (#4176, #4348, #5372): each device storing only a slice of the
  network's data. **This is about scale, not security** — it changes who
  *stores* what, not who can *read* it; what protects Hearth's data is the
  encryption. The small win for Hearth: a care worker's laptop need not keep
  copies of a family's photographs. That is already possible on 0.7 (see
  [latency.md](latency.md)); 0.9 makes it adjustable while running.
- **A "per-app network infrastructure" workstream** — only one item of it
  seen, a transport detail (#397 on the board). If it means apps running
  their own introduction and relay servers, it is the route off Holochain's
  public test server, which every Hearth install uses today (see
  [DPIA.md](DPIA.md)). Worth reading properly before relying on it.
- **Matchable error types instead of strings** (#4270): would let the app tell
  "their device refused" from "their device could not be reached" properly,
  instead of reading the words, as passes do today.

## Holochain 1.0

- **Local discovery: only "re-evaluate mDNS"** (#4527), in a "Local First"
  workstream, with no commitment. The design issue for it
  (holochain/kitsune2 #497) has not moved since 24 August 2026. **The
  Lightningrod Labs build remains the only route to devices finding each other
  on the same network, for a long time yet.** It does not talk to ordinary
  Holochain at all — the two form separate networks — so a normal and a LAN
  install can never share a circle.
- **Moving an agent and a DNA during install** (#4126, #4128), part of
  Holochain's identity work (Deepkey). Would make version upgrades cleaner.
- **Conductor migrations** (#4397), being scoped.

## The Android service — Holochain on phones

A Holochain runtime running as a service on Android and hosting several apps,
built for Volla's phones and tablets. **The route to Hearth on a phone.**

Version 0.2 plans **exporting and importing a person's keys** — the start of an
answer to "can a person recover their identity?", one of the open
[hard questions](hard-questions.md) — and **updating app code**.

## Not for Hearth: Peerkit

A separate experiment from the Holochain Foundation
([holochain/peerkit](https://github.com/holochain/peerkit)): peer-to-peer data
sync in TypeScript, with "deep validation layered on top rather than built in".
Hearth's safety rests on every device checking every write against the rules,
so it is not a replacement for what Hearth is built on.

---

## Why this matters beyond Hearth

The things Hearth has had to build for itself — carrying circles across a
change of rules, and swapping new code into circles in place — are the things
Holochain has planned for 0.8 and 1.0. Hearth is a working example of the
problem, in a setting where losing somebody's record is not an option.
