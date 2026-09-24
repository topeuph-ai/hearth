# Encrypting the record: design note

**Migration batch, item 6. Drafted 19 September 2026.** *Updated 23 September:
everything the design locks is now locked — the record, suggestions, media and
acknowledgement roles — in test releases 0.2.7 onwards; introductions are the
one thing still in the open. See [what is built so far](#what-is-built-so-far).*

Written before any code because it touches every kind of entry, and a mistake
in it is either a record nobody can read or a record that was never as private
as it said.

## What it is for

Today every member's device holds the record as plain text. That is fine while
everybody in the circle is trusted, and it is the weak point the moment
somebody is not. The table from [how-it-works.md](how-it-works.md#where-that-leaves-the-three-levels):

| | Today | With encryption and a new key on removal |
| --- | --- | --- |
| Ordinary use of the app | Holds | Holds |
| What is written **after** somebody is removed, read with a modified app | Does not hold | **Holds** |
| What was written **before** they were removed | Does not hold | Does not hold |
| A device found or examined later | Everything, readable | Old readable, new scrambled |
| A copy made outside the app | Does not hold | Does not hold |

**The one thing encryption adds is the second row**: after somebody is
removed, what the circle writes next reaches their device locked with a key
they were never given. That is mathematics, not an app behaving itself, and
it is the thing the ordinary removal cannot do on its own.

## What Holochain 0.7 provides

Read in `hdk-0.7.0/src/x_salsa20_poly1305.rs`, not taken from documentation:

- **`x_salsa20_poly1305_shared_secret_create_random`** — makes a random key
  inside the keystore (lair). The app gets a reference to it, never the key.
- **`x_salsa20_poly1305_encrypt` / `_decrypt`** — lock and unlock with that key.
  **Only for small things**: see the 8 KiB limit below, found the hard way.
- **`x_salsa20_poly1305_shared_secret_export`** — seals the key for one other
  person, **using X25519 encryption keys, not the agent's identity key**.
- **`x_salsa20_poly1305_shared_secret_ingest`** — the other person unseals it
  into their own keystore.
- **`create_x25519_keypair`** — makes an encryption key pair; the secret half
  never leaves the keystore.

Holochain's own notes on these, worth keeping in view: the key is only as
safe as the device; "encrypted data cannot be validated effectively by the
public DHT"; large data should be locked in chunks; and none of it is resistant
to a future quantum computer. The note about chunks turned out to be the
important one — see the next section.

## What the first build ran into: the keystore takes 8 KiB at a time

**Found on 20 September 2026, by a test with a 20 KB photograph in it**, which
failed with `FrameOverflow, 29937 > 8192`. Confirmed in the source rather than
guessed at: every request to the keystore travels down a framed channel with

```rust
/// Throw errors on the streams if a single message is > 8 KiB
const MAX_FRAME: usize = 1024 * 8; // 8 KiB
```

in `lair_keystore_api-0.7.1/src/sodium_secretstream.rs`. So
`x_salsa20_poly1305_encrypt` cannot lock anything much over about five
kilobytes — not a full-length record (nine sections of 500 words is some thirty
kilobytes), and nowhere near a photograph.

**So the design gained one step, and lost none of its point.** Each thing
written gets a key of its own, used once:

1. The keystore locks that one-use key with the circle's key. Thirty-two bytes,
   well inside the limit.
2. The content is locked with the one-use key, in the zome, with
   XChaCha20-Poly1305.

Opening it is those two steps backwards. **The circle's own key still never
leaves the keystore** — that is the property everything here rests on. What
passes through the app is the one-use key for the one thing being read or
written at that moment, which is no worse than the plain text passing through
it, as it always has.

This is the ordinary way large data is encrypted, and it is what Holochain's
own notes mean when they say to encrypt large data in chunks. The cost is one
dependency in the coordinator zome (`chacha20poly1305`, a cipher used widely
and reviewed far more than anything this project could write), and it does not
touch the frozen rules file's own logic.

## The design

### Every member publishes an encryption key

Passing a key to somebody needs *their* X25519 public key, and today nobody in
a circle has one. So on joining, each member's app makes an X25519 key pair
and writes its public half into the circle as a new entry, **`BoxKey`**, which
only its author may write. Every device checks that.

*(The alternative — sealing the key with the ed25519 identity keys, as knocks
already are — cannot work here: `_ingest` only accepts X25519 keys, and
decrypting a raw key into app memory to re-import it is exactly what
Holochain's notes warn against.)*

### Keys come in "epochs", and removal starts a new one

- When the circle is made, the holder's app makes **key 1**.
- For each member, it writes an entry **`EpochKey { epoch, for_member, sealed }`**
  — key 1 sealed to that member's `BoxKey`. Only the holder may write one.
  Every member's app unseals its own copy into its own keystore.
- Everything written in the circle is locked with the **current** key, and
  says which epoch it used.
- **When somebody is removed**, the holder's app makes **key 2** and seals it
  to everybody *except* them. From then on everything is written with key 2.
  Their device still receives it — replication does not choose person by
  person — but has no key to open it.
- **When somebody joins**, the holder's app seals the current key to them as
  she lets them in. The joiner's `BoxKey` is written on arrival; until then
  the holder cannot seal to them, so the app waits for it — for somebody who
  has only just knocked, that is seconds.
- **Somebody offline** when a key changes finds their sealed copy waiting in
  the circle, addressed to them. The same as everything else here.

### What is locked, and what stays readable

| Locked with the circle's key | Stays readable, and why |
| --- | --- |
| The record — all seven sections, the name, "supported to write this by", coded values | Who may write each thing is checked by every device, so the **author** and the **kind** of every entry stay visible. That cannot be hidden and still be checked |
| Suggestions and "why" | Appointments, answers, proposals and endorsements: the rules check whose keys and whose signatures they carry |
| The role on an acknowledgement ("district nurse") | Departures and successor entries: the rules check who they name |
| How members describe themselves | `BoxKey` and `EpochKey`: they are how the keys travel |
| Photos, sound and video — each 3 MB piece locked separately, as Holochain advises | Knocks: already sealed to the holder |

### What validation loses, and what replaces it

A device cannot check the words of something it cannot read, and must not try
to decrypt during validation even when it happens to hold the key: validation
has to reach the same answer on every device forever, and "does this device
hold key 3?" is not the same everywhere.

So for locked entries:

- **Who may write it is still checked by every device** — the holder writes
  the record, anybody may suggest, and so on. That is unchanged.
- **The 500-word limits become size limits on the locked bytes**, checked by
  every device: a locked section can be no bigger than 500 words could ever
  be. A modified app could still write nonsense of that size; it could not
  write more.
- **Everything about the content** — the words, whether a name is empty —
  is checked by the app, as knocks already are.

## What it does not do

Said plainly, so nobody reads more into it:

- **What was already read stays read.** The removed person keeps every key
  they were ever given. The HDK has no way to delete a key from a keystore.
- **Who did what, and when, stays visible** to anybody still receiving the
  circle: that somebody joined on Tuesday, that three people read the record
  at 2am. For a person who is a danger to the one the record is about, that
  can be enough — which is what moving the circle is for.
- **A device is only as safe as its keystore.** The desktop app currently
  runs with a random keystore password kept on the same device
  (`passwordMode: 'password-optional'`), so somebody holding the device holds
  the keys. Encryption protects a removed person's copy far more than a stolen
  laptop's.
- **Not resistant to a future quantum computer**, per Holochain's own notes.

## Decided by Ceri, 20 September 2026

All three as recommended below:

1. **Somebody who joins later reads the history.** The holder seals every past
   key to them, not only the current one.
2. **Hearth offers a password at startup**, with an explanation, and never
   forces one on somebody who cannot manage it.
3. **How members describe themselves is locked** along with the record.

## The questions those answers settle

1. **Does somebody who joins later read the circle's history?** If yes, the
   holder seals *every* past key to them, not only the current one — so a new
   district nurse can read how the record used to read. If no, they see the
   record as it is now and nothing earlier. *(Recommended: yes — the record's
   history is part of the record.)*
2. **Should Hearth ask for a password when it starts?** Without one, the keys
   are as safe as the device. With one, a lost laptop keeps the record locked —
   and a forgotten password loses nothing, because every other member still
   holds the keys. *(Recommended: offer it, and explain it; do not force it on
   somebody who cannot manage a password.)*
3. **How members describe themselves: locked or readable?** Locked hides from a
   removed person who has joined since. Readable is simpler, and names are
   less sensitive than the record. *(Recommended: locked.)*

## What is built so far

**20 September 2026, on the `migration-batch` branch.** The keys, and nothing
else: the record is still written as plain text, so none of the protection in
the table above is in force yet.

- `BoxKey` — each member's device publishes an X25519 public key, once. Only
  its author may write one.
- `EpochKey { epoch, for_member, sealed }` — the circle's key sealed to one
  member. Only the holder may write one, never for epoch 0, and no bigger than
  a sealed key can be.
- `keep_keys_up_to_date` — the one call the app makes whenever a circle is
  opened. It publishes this device's encryption key, takes up anything sealed
  to it, and, on the holder's device, makes key 1 if there is none and seals
  every key to everybody owed one. It writes nothing when nothing has changed,
  and reports the circle's newest key, the newest this device can use, and
  anybody whose encryption key has not arrived yet.
- `keys_i_can_use` — asked of the keystore by locking one byte with each key,
  because a keystore cannot be asked what it holds. This is what tells a
  removed device's honest answer from a hopeful one.
- Removal starts a new key (`decide_departure`); letting somebody back seals
  them the ones they missed. A key that fails to be made does not undo the
  removal — the next open tries again, and until then the screen can see that
  this device's key is older than the circle's.

And the record itself is locked:

- `AboutMe` gained `locked: Option<Locked>`. When it is there, every other
  field is empty — a record is locked or in the open, never half of each, and
  every device checks that. The words are one sealed blob, with a ceiling of
  what nine sections of 8,000 characters could be.
- The app locks on write and opens on read: `create_about_me` and
  `update_about_me` take the words as typed, and `get_current_about_me` returns
  them opened, beside `locked_out` for a record this device cannot open.
- **The word limits moved to the app.** No device can count words it cannot
  read, so the zome counts them before locking and refuses with the same
  wording the rules used to give. What every device still checks is the size of
  the sealed bytes. This is a real loss of peer-checking, stated here rather
  than glossed: a changed app could write 70 kB of nonsense where 500 words
  belong. It could not write more, and it could not write it as somebody else.
- A device that cannot open the record says so, in the app, in those words —
  it never shows an empty record instead.

Suggestions and media are locked the same way:

- A **suggestion** and its why are sealed together. Which section it is about
  stays in the open, because that is a heading and the holder's screen sorts
  by it.
- **Photos, sound and video** are locked piece by piece — which is what
  Holochain advises for anything large, and what the pieces already were — and
  so are a file's name and what it says in words. What stays readable is what a
  player needs and what every device has to check: section, kind, file type,
  length, how many pieces, how big.
- The three-megabyte limit on a piece is measured on the **file**, before
  locking, because three megabytes is what people were promised. Peers check
  the sealed size, which is a little larger by the nonce and the proof the
  bytes were not tampered with.

Eight tests, in `tests/tests/adversarial.rs`: everybody in the circle can use
the key; **a removed member is not given the next key** and holds key 1 only;
somebody who joins later is given every past key, and nothing is handed out
twice; a member cannot hand out keys; the record is written locked with nothing
in the open; a suggestion likewise, and reads back as it was offered; **a
removed member cannot open what is written next**; and **a removed member
cannot open a photo added after they went** — the two the whole design exists
for.

**Acknowledgement roles are locked too.** "District nurse", beside a name and a
date, is exactly what a removed member should stop receiving. Which version was
read, and by whom, stays in the open: that is the evidence the acknowledgement
exists to be, and it is what every device checks.

### Decided 23 September 2026

**A password, and locked self-descriptions — both yes.**

- **The released app asks for a password when it starts**, so the keys are not
  simply as safe as the device. Today the desktop app runs with
  `passwordMode: 'password-optional'` and a random password kept beside the
  keystore, which protects nobody who is holding the machine.
- **In the demo the password is bypassed**, because a demonstration that asks
  for a password three times before anything happens demonstrates nothing. The
  demo says so on screen instead, so nobody watching believes the released app
  would behave that way.
- **How members describe themselves is locked**, like the rest.

### Still in the open, and next

- **How members describe themselves**, which Ceri decided should be locked.
  Left until last on purpose: an introduction is written the moment somebody
  joins, which may be before the holder has sealed them a key, so locking it
  naively would mean a person unable to say who they are until the holder is
  next online. It wants deciding, not rushing — either the app waits and writes
  the introduction when the key arrives, or the introduction stays in the open
  until then and is rewritten locked.
- **A password when Hearth starts**, which is Ceri's decision 2 and is not
  something the keys settle on their own. Without one the keystore's password
  sits on the same device, so somebody holding the device holds the keys.

Everything in this list is a decision, not a gap in the build.

### From an outside review, 23 September 2026

An outside reader listed the ways designs like this usually fail: nonce reuse,
key substitution, stale keys after a removal, downgrade to plaintext, and the
rest. Each was checked against the code.

- **Stale keys after a removal — a real gap, fixed.** A device without the
  newest key used to lock with the newest one it had, so that a member was
  never stopped from writing. After a removal, that older key is the one the
  removed person still holds. Now nothing is locked with anything but the
  newest key: a member waits a moment instead. And a removal whose new key
  failed to be made is now retried the next time the holder opens the circle —
  the code said it was, and it was not. Coordinator only; the rules did not
  change. Test: `nothing_is_locked_with_a_key_older_than_the_newest`.
- **Nonce reuse — no.** Every entry has its own random one-use key *and* its
  own random 24-byte nonce, so a repeat would need two independent random
  collisions.
- **Key substitution — no.** Only the holder may hand out keys (every device
  checks), and a device opens a key only if it was sealed by the holder.
- **Plaintext — accepted by the rules, and should not be.** Every entry that
  can be locked may also be written in the open, because the rules allow
  either. The app always locks, and only the author can write in the open —
  nobody can downgrade somebody else. But the rules are meant to be the
  backstop that does not trust the app, and here they trust it. **For the next
  version of the rules:** a circle made under these rules refuses anything in
  the open.
- **Nothing ties a locked body to where it sits.** The lock does not use the
  author, the entry type or the epoch as associated data, so a member could
  copy another member's locked suggestion into one of their own. It gains
  them nothing they could not do by retyping it, since they can read it
  anyway. **For the next version of the rules**, cheaply: bind the author and
  the kind of entry. **Done on the `rules-3` branch, 24 September 2026** — see
  "A version on every lock" below. Not on `main` and not released.
- **Introductions are still in the open** — the item above, already known.

### From a second outside review, 23 September 2026 (of 0.3.3)

- **Locked entries are not checked against a real key.** True: a member could
  write something that claims key 999,999, and every device would accept it and
  none could open it. But checking the key number would not stop the harm it
  points at — a member can just as easily write scrambled bytes under a real
  key, and nobody could open those either. What matters is that one unreadable
  entry spoils nothing else. Checked: every list opens each entry on its own
  and carries on past one it cannot open. **One real bug found on the way:** a
  suggestion this device could not open was shown as a blank card, and would
  have been carried into a moved circle as an empty suggestion. Fixed in the
  interface.
- **A handed-out key is not checked to be for a current member.** True, and
  not worth a rule: only the holder writes those, and the holder can already
  give the key to anybody she chooses. Nothing a rule could check would stop
  her.
- **A removed member can still publish an encryption key.** True and harmless:
  the holder's device skips removed members when handing out keys, so a new
  one gets them nothing. "A removed member is given no new key" is tested;
  "even after publishing a fresh encryption key" is not separately, since it
  is the same skip in the same code.
- **Carrying a circle across, or a successor moving it, must not bring a
  removed person back.** Checked in the code: the list of who to invite leaves
  out everybody removed, and the moved circle starts with a key of its own that
  nobody from the old one holds. Not yet tried on two machines; it is in the
  test matrix in [what-is-proven.md](what-is-proven.md).

### A version on every lock, 24 September 2026 (the `rules-3` branch)

The third version of the rules, on a branch of its own. Not on `main`, not
released, and the desktop app is untouched.

**What changed.** Every locked item now says which way it was locked (a
version number, 1), and the rules refuse any other number. And the lock now
seals in what the item is and where it sits: what kind of entry it is (the
record, a suggestion, a file's name and words, a piece of a file, or the role
somebody gave when they read the record), who wrote it, which key it was
locked with, and which circle it is in. None of that is stored a second time.
The reader rebuilds it from what it can already see — above all from who
signed the entry, which nobody can choose for somebody else — and if any of it
differs, nothing opens.

**Why.** It closes the gap the first outside review found: a member copying
somebody else's locked suggestion into one of their own now gets something
that opens for nobody. The version number is what makes a change like this
safe to make again later: an item locked a newer way can be told apart
instead of being mistaken for this one. The idea comes from Mycelix-Health
(see [prior-art.md](prior-art.md)); their code was not looked at, and this was
written from scratch.

**What it costs.** It is a rules change, so it makes a new network. Circles
made under the previous rules come across by being written again, as every
rules change does; nothing written the older way is ever read under these
rules, so there is no path kept for it.

**Tests.** A copied suggestion opening for nobody cannot be shown through the
app, because the app has no way to write a copy — and it is not being given
one. So the test shows the step every reader goes through: the same locked
bytes open as the real author's and not as anybody else's, nor as another kind
of entry, key or circle. A wrong version number cannot be written through the
app either, so the rule is tested directly in the rules' own unit tests.

### One thing the design note got wrong

It said a new key on every **succession**. There is nothing to build: a
successor does not become the holder of this circle, she *moves* it, and a
moved circle is a different circle with a different identity — so it makes its
own key 1, which the old circle's members were never given. Key names include
the circle's identity for the same reason, so two circles on one device can
never reach for each other's keys.

## How it is being built, in order

1. ~~`BoxKey` and `EpochKey`~~ **done.** Then the locked forms of the record,
   suggestions, acknowledgements, introductions and media — **the frozen-file
   part**, with size limits in place of word limits.
2. ~~Making and sealing keys, unsealing on arrival~~ **done.** Then locking on
   write and unlocking on read.
3. ~~A new key on every removal~~ **done.** Succession needs none, for the
   reason above.
4. Tests: ~~a removed member is given no new key~~, ~~a new member is given the
   history~~ — **done.** Still to come: a removed member's copy of the next
   version cannot be opened; an offline member catches up; a locked entry too
   large is refused.
5. The app: a screen that says plainly what is locked and what is not, and the
   password question (decision 2), which the keys do not settle on their own.
6. The [DPIA](DPIA.md) and the table in how-it-works updated from "planned" to
   what was built.

And one check against the licence: [licensing.md](licensing.md) notes that the
Cryptographic Autonomy License forbids using keys to deny somebody control of
*their own* data. A removed member's own suggestions stay readable to them
with the keys they already hold; what they lose is the record, which is not
theirs. Worth a qualified second look before release.
