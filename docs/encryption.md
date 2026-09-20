# Encrypting the record: design note

**Migration batch, item 6. Drafted 19 September 2026. Nothing here is built.**
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
- **`x_salsa20_poly1305_shared_secret_export`** — seals the key for one other
  person, **using X25519 encryption keys, not the agent's identity key**.
- **`x_salsa20_poly1305_shared_secret_ingest`** — the other person unseals it
  into their own keystore.
- **`create_x25519_keypair`** — makes an encryption key pair; the secret half
  never leaves the keystore.

Holochain's own notes on these, worth keeping in view: the key is only as
safe as the device; "encrypted data cannot be validated effectively by the
public DHT"; large data should be locked in chunks; and none of it is resistant
to a future quantum computer.

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

## How it would be built, in order

1. `BoxKey`, `EpochKey`, and locked forms of the record, suggestions,
   acknowledgements, introductions and media — **the frozen-file part**, with
   size limits in place of word limits.
2. The circle functions: making and sealing keys, unsealing on arrival,
   locking on write, unlocking on read.
3. A new key on every removal and every succession.
4. Tests: a removed member's copy of the next version cannot be opened; a new
   member can; an offline member catches up; a locked entry too large is
   refused.
5. The [DPIA](DPIA.md) and the table in how-it-works updated from "planned" to
   what was built.

And one check against the licence: [licensing.md](licensing.md) notes that the
Cryptographic Autonomy License forbids using keys to deny somebody control of
*their own* data. A removed member's own suggestions stay readable to them
with the keys they already hold; what they lose is the record, which is not
theirs. Worth a qualified second look before release.
