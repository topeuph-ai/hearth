# Handover prompt

Paste this into a new session opened in `C:\Users\user\Desktop\aboutme`.
Rewritten 24 September 2026; the version from 9 September is in the history.

---

We're continuing work on **Hearth**. Read your memory first —
`project_release_state.md` is where things stand, `project_hearth.md` is what
Hearth is and why.

**Then, when they become relevant:**

1. `docs/what-is-proven.md` — what has been shown on real machines and what has
   not, including the two-machine test matrix. If anything contradicts it, it
   wins.
2. `docs/threat-model.md` — what is at risk and what stands in the way; the
   findings of four outside reviews and our own audit. Start there rather than
   re-auditing.
3. `docs/holochain-roadmap.md` — what Holochain plans, and what Hearth does
   about each (next: move to the 0.7.1 program without rebuilding the zomes).
4. `docs/upgrades.md` and `desktop-patches/README.md` — how a circle survives
   a new version, and the three patches to the desktop packaging.

## Rules that are not negotiable

**1. The integrity zome is frozen between deliberate migrations.** Changing
`dnas/aboutme/zomes/integrity/aboutme/src/lib.rs` — even a comment — changes
the hash a circle *is*. A change is only ever a planned migration: new pins in
`FROZEN.sha256`, a new app id in the desktop packaging (`hearth.rules.3`, with
the current one moved to the list of earlier ones), and circles carried across.
Everything else happens in the coordinator zome and the interface.

**2. Build zomes with `node scripts/build-zomes.mjs`, never `cargo build`.**
A bare cargo build bakes the machine's paths into the wasm and changes the DNA.

**3. Windows and Linux produce different DNAs and always will.** The released
`.webhapp` is the canonical build.

**4. When a screen looks wrong, ask the running app** (the admin port is in the
log) rather than reasoning about it — one probe has beaten four guessed fixes.
But not while he is in the middle of walking the demo.

## How we work

He walks the interface as a real user and reports what he sees; read the actual
code before theorising, fix, verify, commit, and push `main`. There is only
`main` now; the `migration-batch` branch was retired on 24 September 2026 once
its work was released. A future rules change can start a branch of its own. He is not a software engineer — his field is music — and
is dyslexic: lead with the point, strip the jargon, and never hand him an
opinion as though it were his. Ideas he raises mid-task are to be written down,
not built, unless he asks.

Releases are published only when he says so. Tests take about 50 minutes in CI,
across three runners; they cannot run on Windows.
