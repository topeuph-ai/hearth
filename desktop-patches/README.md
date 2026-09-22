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
