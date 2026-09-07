# Handover prompt

Paste this into a new session opened in `C:\Users\user\Desktop\aboutme`.

---

We're continuing work on Hearth. Read your memory first — `project_hearth.md`
has what it is, why, and the four things not to re-derive.

## Where we are

The app works end to end and is running. 25 integration tests plus 3 unit tests
pass in CI. There's an installable Windows desktop build. The repo is
`topeuph-ai/hearth`, public, Apache-2.0.

I'm walking through the interface as a real user and finding things. That has
been by far the most productive part of this project — seven issues so far that
no test could catch, all on the boundary between the software and a person:

- a name typed into a field expecting a public key
- an invitation that couldn't survive being copied
- a copy button with no visible feedback
- an error screen with no way out
- "what you call them" — them being ambiguous
- a first screen showing a form instead of asking a question
- the same question asked twice

Keep doing that with me. When I report something, fix it, **load the page and
verify it in the browser**, then commit.

## To start the two windows

```bash
cd ui && npm run demo
```

One command. Packs a fresh hApp, starts vite, opens two Electron windows, and
clears its own leftovers. If it hangs, check the log rather than guessing.

## Things that cost real time — don't rediscover them

- **`npm run build` succeeding proves nothing.** Bundling doesn't execute the
  module. A `ReferenceError` at import time builds cleanly and leaves the app
  stuck on "Starting up." forever. **Load the page and read the console after
  every change.**
- **Never slice this file by start/end markers.** A patch that cut from
  `loadCircle` to `loadCircles` deleted five hundred lines including `start()`,
  because `loadCircles` had been appended much later. Use targeted replacements
  and assert landmarks are still present afterwards.
- **Watch CI.** It was red for five commits while I worked on the interface,
  because a zome signature change broke the test helpers.
- **Kangaroo installs the hApp on first run only.** After changing zomes, move
  `~/AppData/Roaming/uk.topeuph.hearth/0.1.x/default` aside or the desktop app
  silently runs the old ones.
- Holochain's docs lag its releases badly. Read the source at the tag.

## Design rules that are load-bearing

- **Offline is not a failure state.** No sync spinners, no staleness warnings,
  nothing implying somebody has fallen behind.
- **A list of people, never an inbox.** A district nurse could be in thirty
  circles. No unread counts, no badges.
- **Never claim what the code can't know.** "Role claimed: district nurse", not
  a verified credential. A name *changed*, not *corrected* — software can't tell
  a typo from a marriage.
- **The record speaks in her voice; forms ask whoever is typing.** "What matters
  to me" on the page, "What matters to Margaret" on the form.
- **Never a dead end.** Every screen has a way back.
- Mistakes are fine when everyone can see them. Don't over-engineer prevention.

## What's next

1. Finish walking the flow: create → write → review → invite → join →
   introduce → suggest → accept → acknowledge.
2. **The NLnet application. Deadline 3 November 2026.** I write it; you draft
   and I humanise. Their Open Internet Stack funds individuals building
   open-source decentralised infrastructure — no company or revenue needed.
3. Free NHS routes, now worth using with a working demo: the NHS Innovation
   Service, Life Sciences Hub Wales (`hello@lshubwales.com`), and
   `england.dtac@nhs.net` for how a no-operator system should be assessed.

## Still open

- **Encrypting entry contents.** Biggest governance gap; every circle member's
  device holds readable copies. Must land before real use.
- Thirty to fifty cloned cells per conductor on a mid-range phone — unknown,
  and a good question for Holochain's developers.
- QR codes for identifiers. 53 characters of base64 is not something anyone
  reads down a phone.
