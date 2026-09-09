# About Me, held by the person

[![tests](https://github.com/topeuph-ai/hearth/actions/workflows/tests.yml/badge.svg)](https://github.com/topeuph-ai/hearth/actions/workflows/tests.yml)

A care record that belongs to the person it is about, and works across
organisations that will never share a computer system.

**There is no server and no company in the middle.** Nobody hosts it. Nobody
can switch it off.

---

## The problem this is trying to solve

Somebody with a learning disability, or dementia, or a long-term condition, is
seen by a lot of different people. A district nurse. A support worker. A GP. A
hospital ward. Their daughter.

Each of those people writes things down in their own organisation's system, and
those systems do not talk to each other. So the person explains themselves
again, every time, to everybody — how to talk to them, what frightens them, what
helps.

There is already a national standard for exactly this: the PRSB **About Me**
standard. The problem was never what to write down. It was **where to put it.**

### Why nobody has solved it

Every previous attempt built a shared place to put the record. And whoever
builds that place becomes **the operator** — the organisation legally
responsible for holding sensitive information about a vulnerable person,
contributed by seven other organisations, and still around in twenty years'
time.

Almost nobody wants that job. Microsoft tried it with HealthVault and closed it
in 2019, taking the data with it.

**So this has no operator.** The record lives on the devices of the people in
the circle, and nowhere else. A professional is not depositing information into
somebody else's system — they are sharing from their own, which is a different
thing legally as well as technically.

That is the whole idea. Everything else here is a consequence of it.

---

## What it deliberately is not

**It is not a clinical record**, and it must not become one by accident. No
medications, no diagnoses, no care plan. The About Me standard says the same
thing about itself.

That boundary is doing real work. It keeps clinical safety certification
(DCB0129), clinician liability and the heaviest data protection questions out of
scope entirely.

> **Do not add clinical fields without understanding what they drag in with
> them.**

---

> ## 🧊 One file in this repository must never be edited
>
> `dnas/aboutme/zomes/integrity/aboutme/src/lib.rs`
>
> **Why:** a circle is identified by a fingerprint taken of that file once it is
> compiled. Change the file and you change the fingerprint, which means every
> circle anybody has ever made becomes unreachable. Their records stay on their
> own hard drives and no future version of this app will ever open them again —
> with no error message, anywhere, to explain it.
>
> **Not even comments.** Adding a comment block was measured to change the
> fingerprint (`dd942cac…` became `941f0b41…`), because the compiler stores line
> numbers for its error messages. Undoing the comment restored it exactly, which
> also tells us the build is reliable and the comment really was the difference.
>
> All the work happens in the interface and the coordinator zome. Neither of
> those affects the fingerprint. CI fails the build if the file moves.
>
> If it genuinely has to change one day, that is a migration with everybody
> re-invited — see [docs/to-a-product.md](docs/to-a-product.md).

---

## Trying it

### ⬇️ [Download the Windows installer](https://github.com/topeuph-ai/hearth/releases/latest)

> **This is an early demo.** It exists to show the idea is possible, not to be
> relied on. Please do not put real information about a real person into it.

One file, about 115MB. Everything it needs is inside — no Rust, no Node, no
separate downloads, no server to run.

**Windows only for now.** For Mac or Linux, see
[docs/building-it.md](docs/building-it.md), and **do not compile it yourself** —
that produces a different fingerprint and a private network of one. Build the
app around the released `hearth.webhapp` instead.

### Three things will happen on Windows. None of them means anything is wrong.

**1. Windows will try to stop it opening.**

You will see *"Windows protected your PC"*. Click **More info**, then **Run
anyway**.

This happens to any application whose publisher has not bought a signing
certificate, which costs money this project does not have. It is not a judgement
about the file. In some NHS settings that will not be acceptable, and it is
worth saying so — it is a cost, not a fault.

**2. For the first minute after downloading, it may refuse to start.**

Nothing happens, or Windows says *"Access is denied"*. The file is not deleted
and not quarantined.

Your antivirus is holding the file open while it scans it, and 115MB takes a
moment. **Wait a minute and try again.** That is exactly what happened here on
9 September 2026, and it cleared on its own.

**3. If it works but never finds the other person, it is almost certainly your
antivirus.**

Everything on your own machine looks fine. No error appears anywhere. It simply
never sees anybody else.

Norton, Kaspersky, Avast, ESET and most workplace networks inspect secure
connections by quietly re-signing them. Your browser accepts this. Holochain
does not, and refuses to connect rather than trust something it cannot check.

**The fix is one line.** Add this address to your antivirus's list of sites to
leave alone:

```
dev-test-bootstrap2.holochain.org
```

Then **close the app and open it again** — it will not retry a connection that
has already failed, so until you restart it the fix looks like it did nothing.

Verified here with Norton 360. Everything else on your machine stays protected
exactly as it was. The longer explanation is in
[docs/building-it.md](docs/building-it.md).

---

## The one thing this project most needs

**Two computers have never run this.**

Every part needed to work across the internet is in the build, and several
copies running on one machine do find each other. But there is one Windows
machine here, the other laptop is a Chromebook, and renting a host costs money.

So "install it on two machines and they find each other" is a claim about what
the code contains. **It is not something anybody has watched happen.**

If you have two computers, that is half an hour that would tell this project
more than anything else could. See
[what is proven and what is not](docs/what-is-proven.md), which is the honest
inventory and the page to trust if anything here sounds more finished than it
is.

---

## How it works, briefly

**A circle** is a private network for one person. It is created around the
public key of whoever holds it, and that key is baked into the circle's
identity. A different holder produces a completely different network.

Circles cannot see each other. That is a fact about the mathematics, not a
setting somebody could get wrong.

**An invitation** is the holder's signature over your public key. You show it at
the door, and every existing member checks it themselves. There is nobody to ask
for permission, because there is nobody in charge.

Because it is signed over *your* key, an invitation cannot be passed on to
somebody else.

**Only the person may write their own About Me.** Being in a circle lets you
read it and confirm you have read it. It never lets you write somebody else's
account of themselves.

**Anyone may suggest something**, and only the holder decides what goes in. A
son remembers what his mother enjoyed; a support worker notices what settles
her. A record only one person may write throws all of that away.

**A professional's whole job is one tap** — confirming they read a particular
version. It costs them almost nothing, and it is the thing families currently
have no way of knowing at all.

> **What that tap proves, and what it does not.** It proves a particular key
> said it had read a particular version. The role beside it — "district nurse" —
> is typed in by that person and **nothing checks it.** The interface says
> "claimed" every single time, and it must keep doing so. A family could
> reasonably rely on a tick that means more than it does.

There is more detail, including what a red-team review found and what it missed,
in [docs/how-it-works.md](docs/how-it-works.md).

---

## What it cannot do

Said here rather than buried, because these are the honest limits.

**Nothing is encrypted where it is stored.** Contents are only reachable by
people let into a circle, but once somebody is in, they have the plain text on
their machine. For About Me — no medications, no diagnoses — that is a smaller
exposure than it sounds. It is still the largest gap.

**"Remove" does not mean what people expect.** Leaving takes a circle off your
own device. It does not reach anybody else's copy. Changing who may enter means
making a new circle and everybody joining again.

That is not evasion, it is arithmetic: once somebody has legitimately read
something, nobody can un-read it. No system anywhere can do this. Circles are
cheap to remake, which is why re-forming one is the honest answer.

**A stranger cannot find the record.** A paramedic who has never heard of this
has no way to discover it exists. Paper solved that with a sticker on a fridge
and we have not solved it at all.

**Joining can take about ninety seconds.** Understood, written up in
[docs/latency.md](docs/latency.md), not yet fixed.

**Nothing has had a full security review.** There has been one audit and its
findings are recorded. Nobody outside this project has looked.

---

## Where the code lives

| | |
|---|---|
| `dnas/aboutme/zomes/integrity/` | the rules — **frozen, see above** |
| `dnas/aboutme/zomes/coordinator/` | the functions the app calls |
| `ui/` | the interface |
| `tests/tests/adversarial.rs` | 41 tests written as attacks, run on every push |
| `docs/` | everything below |

**Built with Holochain 0.7.0** (hdk 0.7.0, hdi 0.8.0), read from the source at
tag `holochain-0.7.0` rather than from documentation, which lags badly.

---

## The documents

Read the first one before the others.

| | |
|---|---|
| [what-is-proven.md](docs/what-is-proven.md) | **Start here.** What is tested, what is built but unwatched, what is not built. If anything else disagrees with it, this page is right |
| [building-it.md](docs/building-it.md) | Building, running the demo, packaging the desktop app, and every trap already paid for |
| [how-it-works.md](docs/how-it-works.md) | The membrane, revocation, and why offline is not a failure |
| [to-a-product.md](docs/to-a-product.md) | What stands between this and something usable, in the order it blocks |
| [standard-and-gap.md](docs/standard-and-gap.md) | Field by field against the PRSB About Me standard |
| [DPIA.md](docs/DPIA.md) | Data protection assessment |
| [latency.md](docs/latency.md) | Why joining takes ninety seconds |
| [prior-art.md](docs/prior-art.md) | What has been tried before, and what became of it |
| [what-to-borrow.md](docs/what-to-borrow.md) | What paper got right that we have not |
| [outer-ring.md](docs/outer-ring.md) | Letting somebody read without joining |
| [storyboard.md](docs/storyboard.md) | The demonstration, scene by scene |
| [funding.md](docs/funding.md) | Where the money might come from |

---

## Running it yourself

Two windows on one machine, each a separate person with a separate store,
talking to each other with nothing in between:

```bash
cd ui && npm run demo
```

Full instructions, and the things that will otherwise cost you an afternoon, are
in [docs/building-it.md](docs/building-it.md).

---

## The demonstration

Two laptops and a phone. A person, their daughter, a nurse.

The daughter writes something. It appears on the nurse's screen. **Unplug the
router** — it carries on working. Take a device out of the room, change
something, bring it back — it catches up.

> There is no account, no server and no company. If I am hit by a bus tomorrow,
> this carries on working.

---

## Licence

Apache License 2.0. See [LICENSE](LICENSE).

Built by one person who is not a software engineer, with AI assistance, and the
design decisions are his. [What that means in
practice](docs/what-is-proven.md#who-is-building-this-and-with-what).
