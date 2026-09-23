# Patches to the desktop packaging

Hearth's desktop app is assembled with [Kangaroo](https://github.com/holochain/kangaroo-electron),
which is cloned separately and is not part of this repository. These are the
changes Hearth needs on top of it.

**They live here because a fresh clone of Kangaroo does not have them, and
because the first of them is the difference between an upgrade and silent data
loss.** See [docs/upgrades.md](../docs/upgrades.md) for why.

## Applying them

From the Kangaroo clone (`hearth-desktop` on the build machine):

```bash
git apply /path/to/hearth/desktop-patches/01-pin-the-data-directory.patch
```

Then check it took:

```bash
grep -n "HEARTH_DATA_DIRECTORY" src/main/filesystem.ts
```

## The patches

### 01 — pin the data directory

**Without this, releasing 0.3.0 destroys every circle.**

Kangaroo names its data directory after the app version: `breakingVersion()`
returns `0.2.x` for any 0.2 release, and `0.3.x` for any 0.3 release. The
directory holds the conductor, the keystore and every circle. So the first
0.3.0 release starts a fresh conductor with a fresh keystore and a fresh agent
key, the old data sits on disk untouched, and the person has become a stranger
to everybody who knew them — with no error message anywhere.

This patch fixes the directory at `0.2.x`, which is where every installation of
0.2.0 to 0.2.4 already keeps its data. The name is now only a name.

Upgrading the rules is done instead by installing the new version alongside the
old one in the same directory, and carrying circles across — the rest of
[docs/upgrades.md](../docs/upgrades.md).

**Applied to the build machine's clone on 22 September 2026. Not yet in any
release.**

### 02 — install new rules alongside the old, as the same person

Two changes, and neither is finished work on its own: see *What is still
missing* below before building a release with this.

**The app id now carries the rules version.** `HAPP_APP_ID` was
`kangaroo.happ`, and Kangaroo installs that id once and never again. Shipping
new rules under the same id means nobody ever runs them. It is now
`hearth.rules.2`, and it changes when — and only when — the integrity zome
changes. `HEARTH_PREVIOUS_APP_IDS` lists what came before, so the app knows
where the older circles are.

**The agent key is reused.** Kangaroo generates a new one on every install. Beside
an existing installation that would make the person a stranger: invitations
would mean nothing, nobody would recognise them, and the circles they hold could
not be carried across. So if the conductor already holds an earlier version of
Hearth, its key is reused. Same keystore, same person, new rules.

**A token for every installed version.** `getEarlierAppTokens()` issues one per
earlier version of Hearth that is actually installed, `launch.ts` asks for them,
and `windows.ts` puts them in the page as `window.__HEARTH_EARLIER_APPS__`
beside the launcher's own environment. Empty on a first install.

The interface side is in this repository rather than in this patch: it connects
to each earlier version at startup, lists those circles beside the current ones
marked *"made with an older version"*, and routes every question about a circle
to the app that can answer it.

#### What is still missing

**The move across versions** — step 3 of
[docs/upgrades.md](../docs/upgrades.md). An older circle can be opened and read;
it cannot yet be carried to the new rules. The mechanism it will use (moving a
circle) is built and tested; what is missing is founding the new circle in the
*new* app rather than the old one.

**And it has never been run.** These patches type-check and the interface
passes its own checks, but no build has been made and no machine has been
through an actual upgrade. Nothing here should be trusted until a circle made
on 0.2.4 has been opened by a later version on two machines.

*Since overtaken: the upgrade was proven on two machines on 22 September, and
the move across versions was built on 23 September. See docs/upgrades.md.*

### 03 — new code under the same rules

**Without this, a release that changes only the code is ignored** on every
machine that already has the current rules installed.

Patch 02 makes the app id change only when the rules change. That is right for
the rules, and it leaves a gap: Kangaroo installs an id once and never again, so
0.3.0 — new code (passes), same rules — would show the new screens on top of the
old functions. Pressing *Make a pass* would fail with a function not found.

Holochain can swap a cell's code (its coordinator zome) in place, keeping all
its data: `update_coordinators`. On every launch, if the current app id is
already installed, this patch reads the coordinator out of the bundled hApp and
swaps it into every cell of that app — the lobby, every circle and every door,
since each clone has a DNA of its own. Once per launch rather than once ever, so
there is nothing to record and nothing to fall out of step.

**The rules are never swapped this way.** A change to the integrity zome is a
new app id and a circle carried across, as in 02.

**Written 23 September 2026, built into 0.3.0, not yet run on a machine that
had 0.2.7.** The first launch of 0.3.0 over 0.2.7 is the test: the log should
say *Brought the code up to date in N cell(s)*, and *Make a pass* should work.
