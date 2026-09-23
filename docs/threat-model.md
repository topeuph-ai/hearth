# Threat model

**One page, 23 September 2026 (Hearth 0.3.4).** Asked for by two outside
reviews, which each had to piece it together from a dozen other documents.
What is at risk, who might go after it, what stands in the way, and what is
left. Where the defence is a rule every device checks, it says so; where it is
only the app behaving well, it says that too. Nothing here has been reviewed by
a security specialist yet — that is the first thing this page asks for.

## What is being protected

- **The record** — seven sections about a person, often somebody who cannot
  speak for themselves. Not clinical, by design.
- **Who is in a circle** — who looks after whom is itself sensitive.
- **Photographs, sound and video** of the person.
- **The person's say over all of it** — that nobody else can rewrite their
  record or take over their circle.

## Threats

| Threat | What stands in the way | What is left | How sure |
| --- | --- | --- | --- |
| **A stranger joins a circle** | The membrane: every device refuses anybody without a signed invitation from the holder (and the second person, if the circle asks for two) | — | Rule; tested |
| **A member rewrites or deletes the person's record** | Only the author may write or delete an entry, checked by every device | — | Rule; tested |
| **A member forges an invitation, or passes theirs on** | Invitations are signed and name the one key they are for | — | Rule; tested |
| **Somebody removed reads what is written afterwards** | A new key on every removal; nothing is locked with an older key | They keep everything they had already seen. No design can take that back | Tested; one gap found in review and fixed (0.3.1) |
| **A lost, stolen or seized device** | The keystore | **The biggest practical risk.** The released app does not yet ask for a password, so holding the device is holding the keys. Decided; not built — it needs a careful test so existing installs are not locked out | Known gap |
| **Somebody reads the content as it passes through the network** | Content is locked to the circle's keys before it leaves the device | Who wrote something, and when, can be seen by peers — not what it says | Built; not independently reviewed |
| **A professional given a pass reads more than they should** | The holder's device reads the pass's terms from what it wrote itself, never from the reader; Holochain refuses a stopped pass before our code runs | A pass works for whoever holds it, like a key | Tested |
| **Somebody claims the holder has gone, to take over** | A successor must be named in advance; a waiting period the whole circle sees; the holder can say "I am still here"; a named checker confirms | Coercion of the checker or the successor | Rule; not yet tried on machines. An old claim that could come back to life was found in our own audit and fixed (0.3.4) |
| **A door flooded with knocks** | Ten knocks per key, checked by every device; a button to give the door a new address (0.3.4) | Keys cost nothing, so a determined flood can use many; the answer is to leave the door. A stranger can also send "somebody is asking to join" without knocking — held to a trickle on screen since 0.3.4 | Rule and app |
| **Unreadable entries clogging a circle** | Each entry is opened on its own; one that cannot be opened is skipped (fixed 0.3.4) | A member can write things nobody can read; they gain nothing by it | App |
| **The holder is coerced, or somebody lies about who they are** | Hearth controls access, not identity: "district nurse" is shown as a claim, never as verified | Not a software problem; stated, not solved | By design |
| **A flaw in something Hearth is built on** | Versions pinned; dependencies checked against the public advisory database on every push (0.3.4) | The check reports; a person has to act | CI |
| **An update strands or leaks a circle** | The rules are frozen and checked by hash in CI; new rules install beside old ones, and circles are carried across | Carrying an encrypted circle with media across is not yet tried on machines | Partly proven |

## Our own audit, 23 September 2026

After three outside reviews, a pass of our own over the parts they had not
looked at closely. Sound: a member cannot fake a move (the zome refuses it
before any screen sees it); the successor rules are every device's, not just
the app's; a pass reads its terms only from what the holder wrote; nothing
anybody types is ever put on screen as HTML. Found and fixed, all in 0.3.4:

- **A successor's old claim could count again** if the holder named the same
  person afresh — ready at once, its checks and waiting period long past. A new
  naming now starts succession over. Tested.
- **The zome and the app weighed checks differently.** The zome let any
  member's "she cannot carry on" count; the app, rightly, let the named
  checker's answer come first. Now both do. The app was the gate, so this was
  never open, but two layers that disagree are one change away from a hole.
- **"Somebody is asking to join" could be sent without knocking**, as often as
  anybody liked, each one reloading the holder's circle. Now at most one
  announcement per person a minute, and one reload every fifteen seconds.
- **A pass read in a loop** could push the real record of who read what off
  the end of the holder's list. Repeat reads within a minute now count as one.

One thing to test rather than fix: every write now reads the circle's keys and
removals from the network first. Offline, those reads should fall back to what
the device already has — worth confirming on the two-machine test with the
internet unplugged, since the offline edit proven on 14 September came before
encryption.

## Deliberately out of scope

- **Clinical content.** Medicines, diagnoses, care plans. Keeping them out is
  what keeps clinical safety certification out.
- **Taking back what somebody has already read.** Impossible anywhere; stated
  rather than implied.
- **Verified identity.** Hearth never makes a claim look checked.
- **Encryption that resists future quantum computers.** Not yet a practical
  concern for this kind of record, and Holochain's keystore, which Hearth's
  keys live in, does not offer it.

## What would change this page

1. **A review by a person who does this for a living** — of the rules, the
   keys, and the interface. Three AI reviews found one real gap in the keys
   and one display bug; a specialist would look where none of them can.
2. The keystore password, built and tested.
3. The two-machine tests in [what-is-proven.md](what-is-proven.md), especially
   carrying an encrypted circle across and a successor taking one over.

See also: [encryption.md](encryption.md) for the keys,
[hard-questions.md](hard-questions.md) for the questions software cannot
answer, and [what-is-proven.md](what-is-proven.md) for what has been shown.
