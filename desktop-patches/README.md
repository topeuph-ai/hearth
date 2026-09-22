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

#### What is still missing

The interface only ever receives a token for one app (`getAppToken` uses
`HAPP_APP_ID`, and `windows.ts` passes a single `INSTALLED_APP_ID` to the
renderer). So with these patches applied and nothing else, a person who updates
would see the new rules and **not** their old circles — which are still on disk,
intact, and still theirs.

**Do not release on these two patches alone.** What is needed next:

1. A token per installed app, and both passed to the interface.
2. The interface listing circles from both, with the old ones marked as
   belonging to a version that is going away.
3. The move across, using the mechanism that already exists.

Steps 3 and 4 of [docs/upgrades.md](../docs/upgrades.md).
