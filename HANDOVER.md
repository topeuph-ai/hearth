# Handover prompt

Paste this into a new session opened in `C:\Users\user\Desktop\aboutme`.

---

We're continuing work on **Hearth**. Read your memory first — `project_hearth.md`
has what it is and why.

**Then read these, in this order. They are the state of the project and they
replace anything you would otherwise have to infer:**

1. `docs/what-is-proven.md` — what is tested, what is built but unwatched, what
   is not built. If anything else contradicts it, it wins.
2. `docs/to-a-product.md` — the roadmap, ordered by what blocks what.
3. `README.md` — the freeze notice near the top matters more than it looks.

`docs/funding.md`, `docs/prior-art.md`, `docs/outer-ring.md`,
`docs/storyboard.md` and `docs/standard-and-gap.md` are there when they become
relevant. Don't read them up front.

## Four rules that are not negotiable

**1. The integrity zome is frozen.**
`dnas/aboutme/zomes/integrity/aboutme/src/lib.rs` must not be edited — **not
even a comment**, which was measured to change the compiled hash. A circle *is*
that hash, so any change makes every existing circle unreachable, silently. CI
fails if it moves. Everything worth doing happens in the interface and the
coordinator zome, neither of which touches the DNA.

**2. Build zomes with `node scripts/build-zomes.mjs`, never `cargo build`.**
A bare cargo build bakes the machine's home directory into the wasm and changes
the DNA. The wrapper strips it.

**3. Windows and Linux produce different DNAs and always will** — the remapped
paths keep each system's path separator. So the released `.webhapp` is the
canonical build and every platform is assembled from it. Not a bug to fix.

**4. Never probe the conductors while he is walking the demo.** Read the code
instead. If you can't tell from the code, ask him one precise question. Four
guessed fixes have cost more than any single question ever has.

## How we work

He walks the interface as a real user and reports what he sees; you fix it,
verify, and commit. That has been the most productive part of this project —
most of the real bugs were found that way and no test would have caught any of
them. When he reports something, **read the actual code before theorising**, and
say plainly when you need a fact from him rather than guessing.

Restart the demo with `cd ui && npm run demo -- 3`. Kill Electron, holochain,
lair-keystore **and whatever holds port 5273** first, or it refuses to start.

He is not a software engineer — his field is music — and the code is written
with AI assistance while the design decisions are his. Lead with the point,
strip the jargon, and never hand him one of your own opinions as though he had
arrived at it.

## Where things stand, 9 September 2026

**Released.** `v0.1.1` is published with the installer and `hearth.webhapp`.
`v0.1.0` is marked superseded — it built a different circle from its own source.

**The release installs and runs — on the machine it was built on.** Downloaded
from the release page, verified byte-identical to the local build, installed
(exit code 0) and launched: the window opens, Holochain and lair-keystore start
beside it, and the DNA inside the installed app is the released one.

One thing to know: for about a minute after downloading, the installer would
not start — "Access is denied", nothing quarantined. That was antivirus holding
a 115MB file open while it scanned. It cleared on its own. The README says
"wait and try again", which is what actually happened; an earlier version of
that note blamed Norton outright and was wrong.

**Nobody outside this machine has run it.** That is still the single most
important open item, and only somebody else's computer can settle it.

**Two machines have still never found each other.** There is one Windows machine
here; the second laptop is a Chromebook. This is a money problem, not a
technical one, and it is written up honestly in the README and the release notes
as the thing a stranger could settle in half an hour.

**Recently done:** a third front-page option so a circle needing two agreements
is made *once* rather than re-formed; a way out when the second person dies or
loses their device; several state-leak fixes (leaving a circle, opening the join
screen, the refresh reaching under an open form).

**Recently learned, and worth not re-deriving:** the second yes is a *tripwire,
not a lock*. The holder can always re-form the circle without one — what she
cannot do is drop it quietly, because everybody must re-join. Saying otherwise
is false, and the stronger claim is the tempting one.

## What to do next, in this order

1. **Get the installer onto a machine that is not this one.** Everything else is
   secondary to somebody outside this room running it.
2. **Two machines finding each other.** Cheap for anybody with two computers.
3. **Restack, deadline 3 November 2026, noon CET.** €5k–€50k, individuals may
   apply, open source required. Read the call text before writing anything —
   the scope is more infrastructure-shaped than this project is, and that
   argument has to be made deliberately. See `docs/funding.md`.

## What not to do

**Stop working on the second yes.** A whole day went into it — where it is
appointed, whether it makes a second circle, a third front-page button, then
nearly a fourth, then what happens when the second person dies. Every problem
was real; none of it was worth that share of the effort. It is an *optional*
safeguard that is not in the About Me standard and that most circles will never
use. It works, it is explicable, leave it.

The pattern to avoid is the one that produced that: finding the next real
problem inside a feature and treating "real" as the same as "worth doing now".
He noticed before I did.
