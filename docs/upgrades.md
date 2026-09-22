# Upgrades: how a circle survives a new version

**Written 22 September 2026, from the packaging source rather than from
assumption.** This is the work that blocks the migration batch, and the reason
it is harder than "change the rules and release".

[to-a-product.md](to-a-product.md#0-the-one-nobody-has-written-down-upgrades)
says the freeze exists because changing the rules strands every circle. That is
true and it is only half of it. The other half is in the desktop packaging, and
it is worse.

---

## Three facts, read in Kangaroo's source

The desktop app is built with Kangaroo. Three things it does decide everything
here.

**1. The data directory is named after the version.** `filesystem.ts` builds
every path under `breakingVersion(app.getVersion())`, which for a 0.x release is
`0.<minor>.x`. So 0.2.0 through 0.2.9 share one directory, and **0.3.0 starts a
fresh one**: a new conductor, a new keystore, a new agent key, and not one
existing circle in sight. The old data is still on disk under `0.2.x` and
nothing ever looks at it again.

**2. Within a minor version, the hApp is installed once and never updated.**

```ts
const installedApps = await this.adminWebsocket.listApps({});
if (installedApps.map((appInfo) => appInfo.installed_app_id).includes(HAPP_APP_ID)) return;
```

If the app id is already installed, `installHappIfNecessary` returns. So
releasing the migration batch as 0.2.5 would ship a new `.webhapp` that the
conductor never installs. Everybody would keep running the old rules and nothing
would appear to happen.

**3. Every install generates a new agent key.** `generateAgentPubKey()` on each
install, so a fresh install is a different person to everybody who knew them.

### What those three mean together

| Release as | What happens |
| --- | --- |
| **0.2.5** (patch) | Same data directory. New hApp ignored. **Nothing changes.** |
| **0.3.0** (minor) | New data directory, new keystore, new agent. **Every circle disappears and the person becomes a stranger.** |

**Neither path upgrades the rules and keeps the circles.** That is the whole
problem, stated plainly, and no amount of work inside the zomes fixes it.

---

## What has to be built

Four pieces. The first two are packaging and are not optional; the third is the
one that carries a circle; the fourth is the honest fallback.

### 1. Stop the data directory moving

The version-keyed directory is a sensible default for an app whose data can be
thrown away. It is wrong for this one.

The change is small — pin the directory rather than deriving it from the version
— and it has to happen **before** any further release, because every release
made under the current scheme adds people who will be stranded by the next minor
bump. Existing users live under `0.2.x`, so the pinned value must be that, or
the first run of the new version must move the old directory to the new name and
say so in the log.

### 2. Install a new version alongside the old, as the same person

Two changes to the install step:

- **A new app id per rules version** (`hearth.v2` beside `hearth.v1`), so the
  conductor installs the new hApp instead of returning early. Both apps then run
  in one conductor, against one keystore.
- **Reuse the existing agent key** rather than generating one. The person stays
  the same person, which is what makes a move across versions possible at all:
  the new circle recognises them, and their invitations still mean something.

The interface then needs a token for each installed app, lists circles from
both, and marks the old ones as belonging to a version that is going away.

### 3. Carry a circle across, using the mechanism that already exists

This part is built and tested: **moving a circle**. The holder founds a new
circle, the history is carried as history, the members' own apps follow the move
when they see it, and the app takes the old circle off the device when it is
done.

Moving across versions is the same act with one difference: the new circle is
founded in the **new app** rather than the old one. Everything else — the
invitations, the signal that tells the others, the note explaining what happened
— is already written.

What it cannot do, and this is not a bug to be fixed later:

> **Entries are signed by their authors.** Whoever performs the move cannot
> re-sign somebody else's entry and should not be able to. So the words come
> across; the signatures do not. What lands in the new circle is *a record of
> what was said*, written by the person doing the moving, rather than the signed
> original. The interface must show that difference rather than hide it.

### 4. Export and import, for when even that is impossible

If somebody has already updated past a minor version bump — or restores a backup
onto a machine whose app has moved on — there is no live old circle to move
from. All that is left is the data on disk.

So: a plain file the old app can write and the new app can read, containing the
record, the media, and the history as history. Crude, not automatic, and the
only thing that works when the two versions cannot be running at the same time.

It is also the most reusable part. A documented export format is something
another project can implement; a clone-to-clone move is specific to Holochain.

---

## The order to do it in

1. **Pin the data directory** — cheapest, most urgent, blocks nothing else but
   is made worse by every release that happens without it.
2. **Install alongside, keeping the agent key.**
3. **Move across versions**, reusing the circle-move mechanism.
4. **Export and import**, and write the format down.

Nothing in the migration batch may be released until 1 and 2 are done, and no
circle holding a real record should be upgraded until 3 works. That is the
promise the freeze was made to keep.

---

## What this changes about the release plan

The batch was going to be released as a version bump. It cannot be, in either
direction, until the packaging changes above are made. This is the honest state
of it:

- **0.2.4 is the last release under the old scheme**, and anybody using it will
  need to be walked across by hand or by export.
- The next release has to carry the packaging fixes, and should say in its notes
  exactly what it does to existing circles.
- **Test it with two machines before it goes out**, with a circle made on the
  old version and carried to the new one, because this is the one change where
  being wrong destroys somebody's record silently.
