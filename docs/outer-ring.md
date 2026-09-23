# The outer ring

**Status: first version built 23 September 2026, on the `migration-batch`
branch, not yet tried on two machines.** The design below was written on
2026-09-08, before any code; [what was built](#what-was-built-23-september-2026)
is at the end, with what it does not do yet. The idea is adapted from the
capability-token sharing in the 2022 IEEE Access paper — see
[`prior-art.md`](prior-art.md).

---

## The problem it solves

Hearth has one door and one kind of membership. You are invited through the
membrane into a circle, and from then on you hold a **complete copy of
everything in it, forever**.

For Margaret's daughter that is right, and it is the whole point: she has the
record on her own machine, so it works in an ambulance, on a ward with no
signal, and after Margaret's phone is lost.

For the district nurse who visits twice, it is absurd. To read "how and when to
support me", she must be invited, join a circle, take a permanent copy of
somebody's private record onto a work laptop, and stay there afterwards because
nothing can remove her.

**So the app currently has no shape for the person it most needs to reach.**
Every professional is either a family member or a stranger, and there is nothing
in between.

---

## Three rings

The rule is one sentence: **how much of the person you carry decides which ring
you are in.**

### 1. The person

Writes. Nobody else does. Unchanged.

### 2. The circle — a membrane

Family, close carers, the people who will be there when the person cannot speak
for themselves. Admitted through the membrane, hold a full copy, work with no
signal and no server.

They carry everything, because they are the ones who will need it when nothing
else is working. Unchanged.

### 3. The outer ring — a pass

Professionals passing through. A nurse, a paramedic, a social worker, a locum.

They are **not admitted to anything**. The person grants them the right to ask
for **one part** of the record. They hold no copy. When they need it they ask,
and the answer comes from the person's own device.

They carry nothing, because they are passing through.

**This is not a technical compromise between two designs. It is how care
actually looks**, and the current app models only two thirds of it.

---

## What it gets us

- **Per-section sharing.** "How and when to support me" without "what matters to
  me". At present it is all or nothing.
- **Reading without joining.** No invitation, no membrane, no copy on a work
  laptop.
- **Revocation that works** — for this ring. Withdraw the grant and the next
  request is refused. It does not fix revocation for circle members, and nothing
  does. It means far fewer people need to be circle members in the first place,
  which is the practical answer to the same worry.
- **A professional can be a professional**, rather than being made an honorary
  member of a family.

## What it costs

**It only works while the person's device is reachable.** This is the real cost
and it must not be buried. A circle replicates to every member precisely so the
record survives a flat battery and an unconscious owner. The outer ring gives
that up: no copy anywhere means nothing to read when the phone is off.

So the outer ring is **an addition, never a replacement.** Anybody who must be
able to read this when the person cannot help belongs in the circle, carrying a
copy. The outer ring is for everybody else — which is most people, most of the
time.

**And it needs the discovery problem solved to be worth much.** A pass has to
reach the nurse somehow. That is the same unsolved question as the sticker on
the fridge in [`what-to-borrow.md`](what-to-borrow.md), and it is not made
easier by this.

---

## Naming

**The interface must never say "token".** It is a crypto word, it will be read as
a crypto word, and this project is not that. Holochain calls the underlying
thing a capability grant, and the code may as well — but the code is not what
anybody reads.

Say what it does, and avoid naming the object at all where possible:

- **"Show one part of this to somebody"** — the action.
- **"A pass"** — if it must be a noun. Plain, ordinary, and the right meaning:
  it lets somebody in for a purpose and can be taken back.
- **"Stop showing it"** — the withdrawal. Not "revoke".

The same rule already applied elsewhere here: not "delete", because there is no
operator who could; not "remove", for the same reason.

---

## Does it make the app more complicated?

**For anybody who never uses it, no** — it is a new door, not a change to the
existing one. Nothing on Margaret's screen has to move.

**For the project, yes**, and honestly:

- A second way of reaching the record is a second set of rules to write, test and
  get wrong.
- The offline caveat is hard to explain and harder to explain *at the moment it
  fails*, which is the moment somebody is standing in a hallway getting nothing.
- Every extra concept is one more thing for somebody frightened and tired to
  understand.

**The mitigation is that it stays folded away** until somebody asks for it, like
the second yes. The default walk gains nothing and loses nothing.

---

## What to do with it

*Written 2026-09-08. Overtaken: the first version is built — see the next
section.* The original order was:

1. **Publish a release.** Nothing else matters until somebody outside this room
   can run the thing. See [`what-is-proven.md`](what-is-proven.md).
2. **Propose this as the work to be funded.** It is a much better answer to
   "what would you do with the money" than more polish on what exists. It is
   concrete, it is scoped, it is genuinely novel, and it sits squarely inside
   Restack's interest in alternatives to closed online technologies — the closed
   alternative being a server that holds everybody's record and hands out views
   of it. See [`funding.md`](funding.md).
3. **Then build it**, with the discovery problem treated as part of the work
   rather than assumed away.

---

## What was built, 23 September 2026

Five decisions, taken by Ceri before any code:

1. **Only the holder makes a pass.** Being in the circle does not make the
   record yours to hand out.
2. **A week by default**, with "just today" and "until I stop it" offered.
3. **Three sections ticked by default** — how to talk with me, please do and
   please do not, how and when to support me — because those are what somebody
   meeting the person for the first time needs. The holder can tick others.
4. **The use is seen.** "Ward 7 read this" appears on the holder's screen.
5. **Words only.** No photographs, sound or video through a pass.

### How it works

**Nothing in the frozen rules changed.** The integrity hash is the same before
and after; everything is in the coordinator zome and the interface.

- **A pass is a Holochain capability grant**, made in the circle's **door** —
  the one network anybody with the address can already enter. It unlocks
  exactly one function, `read_with_a_pass`, and what it allows (which circle,
  which sections, who it is for, when it stops) is written in the grant's tag
  by the holder's own device.
- **The reader enters the door and asks the holder's device**, presenting the
  pass. Holochain checks the secret against a live grant *before our code runs
  at all*; a stopped pass is refused there.
- **The holder's device reads the chosen sections from the circle** and sends
  back those words and nothing else. It reads the terms from the grant it made,
  never from the reader, so no reader can ask for more than was given.
- **The reader's screen keeps nothing.** The words are shown and are gone when
  the screen is left.

What travels is a pass of twelve short rows starting with the word PASS —
the same self-checking letters as an address, so a typing mistake is pointed at
by row — and the same thing as a code for a camera.

### Tested

Six tests, in `tests/tests/adversarial.rs` under *Passes*: a pass reads the
chosen sections and nothing else, and the holder's screen hears it; a stopped
pass opens nothing; a pass past its time opens nothing; a guessed secret opens
nothing; a member cannot make a pass, and a pass cannot be made inside the
circle; and the function that fetches the words cannot be used by a member as a
way round the pass.

### What it does not do yet — said plainly

- **It works only while the holder's device is on, with Hearth open.** This is
  the cost described above, and it is now real rather than predicted.
- **Only the holder's own screen sees a pass being used**, and only on that
  computer. Decision 4 wanted the *circle* to see it. That needs a new kind of
  entry in the circle — a change to the frozen rules — so it belongs in the
  migration batch rather than being faked here.
- **The name on the pass is the holder's label, not a checked identity.** "Ward
  7" is what she typed. A pass works for whoever holds it, like a key.
- **A pass is shown once.** If it is lost, stop it and make another.
- **Discovery is still the hard part.** A nurse who has Hearth and is handed a
  pass can read. Getting Hearth onto a ward's machine is not solved by this.
- **Tried on two machines, 23 September 2026 (0.3.1): it works.** A pass made
  on the PC was read on the laptop, on the same home network, with the PC's
  Hearth already upgraded from 0.2.7 — so the new code did reach an existing
  circle (desktop patch 03). The first try said the PC could not be reached;
  the second worked. The laptop had just entered the door and had not yet
  found the PC, which on released Holochain takes a minute or two (see
  [`latency.md`](latency.md)). The reader's app now keeps asking for about two
  minutes, saying it is finding their device, and a refusal from the holder
  (a stopped or run-out pass) is shown as itself rather than as "could not be
  reached", which Holochain had been making it look like.

### A note on the funding answers

The NLnet answers drafted in September describe the outer ring as the work to be
funded. A first version now exists, so those answers should say so: the funded
work becomes the parts above that are not done — the circle seeing its use,
reaching a device that is off, and getting a pass onto a ward — rather than the
idea itself.

---

## A note on the researchers

The 2022 team does not appear to have continued this. Muhammad Khandaker at
Heriot-Watt now works on 6G, intelligent reflecting surfaces, physical-layer
security and UAV-enabled IoT. Shakila Zaman's later work is cross-chain swaps,
metaverse and zero-trust platforms. No Holochain follow-up from that group is
findable.

So the instinct that somebody started this and moved on was right — just about
the 2022 paper rather than the 2025 one. **The idea is not being pursued by its
authors, and nothing here is treading on live work.**

Khandaker's contact details are public at Heriot-Watt. Whether an email is worth
sending is a judgement call: the honest pitch is not "help us" but "you proposed
this in 2022, somebody built the surrounding thing, here is what happened."
