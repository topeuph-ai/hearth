# What licence Hearth is under, and what shipping Holochain asks of us

**Read 17 September 2026, from the licence texts themselves rather than from
summaries. Nobody here is a lawyer and this is not legal advice** — it is what
the words say, written down so that somebody qualified can check it quickly,
and so that the obligations are met rather than discovered.

## The short version

- **Hearth's own code is Apache-2.0**, and stays that way.
- **Holochain is under the Cryptographic Autonomy License 1.0 (CAL-1.0)**, an
  OSI-approved open-source licence written for peer-to-peer software. So are
  the HDK and HDI, the libraries Hearth's rules are built with.
- **That is a good fit for this project.** CAL's central demand — that you must
  not stand between a person and their own data — is the thing Hearth exists to
  do. It costs us nothing we were not already doing.
- **Two things are missing today**, both small: the built app carries no
  Holochain notices, and nothing tells people where to get Holochain's source.

## What CAL actually requires

CAL's conditions bite when the Work "is distributed, communicated, made
available, or made perceptible to a non-Affiliate third party (a 'Recipient')"
(§4). Publishing an installer is exactly that. The conditions:

| Condition | What it says | Where Hearth stands |
| --- | --- | --- |
| **§4.1 Access to Source Code** | Give each recipient the source, or free network access to it — and it must stay available for a year after you stop distributing | Holochain's source is public on GitHub, which §4.1.1 explicitly allows ("by You or by a third party, such as a public software repository"). **Nothing in Hearth points at it.** Fix: a line in the README, the release notes and the app |
| **§4.2 Maintain User Autonomy** | You must not use your permissions to stop somebody using their own copy with their own **User Data** | Met by design. Hearth has no operator, holds nobody's data and has no lock |
| **§4.2.1 No withholding User Data** | If you provide services to somebody via the Work, give them a copy of their data on request | We provide no service to anybody. Every copy is theirs |
| **§4.2.2 No technical measures that limit access** | No keys, hashes or protection measures that deny somebody control of their own data | The keystore passphrase protects the person's own keys on their own device, which is the opposite of limiting them. **Worth re-reading before encryption is built**, so key rotation never locks a member out of their own copy |
| **§4.3 Notices and attribution** | Keep all licensing and authorship notices and give them to each recipient, "together with a statement acknowledging the use of the Work" | **Not met.** The installer ships Electron and Chromium notices and nothing for Holochain or lair-keystore |

## The part that decides how far it reaches: the Combined Work Exception

CAL comes in two forms. Files marked "with Combined Work Exception" may be
combined into a larger work licensed however you like (§4.5). Files marked
plain `CAL-1.0` have no such exception.

**Checked at the `holochain-0.7.0` tag:**

| Crate | Licence |
| --- | --- |
| `holochain` (the conductor) | `CAL-1.0` |
| `hdk` | `CAL-1.0` |
| `hdi` | `CAL-1.0` |
| `holochain_integrity_types` | `Apache-2.0` |

**No Combined Work Exception anywhere in the parts Hearth uses.** Two
consequences:

1. **The installer** ships the `holochain` and `lair-keystore` programs
   unchanged. That is distribution of the Work, so §4.1 and §4.3 apply:
   notices, and somewhere to get the source.
2. **Hearth's rules are compiled with the HDK and HDI**, so the wasm files
   inside `hearth.webhapp` contain CAL-licensed code. Distributing them is
   distributing part of the Work. Our own source must be available to anybody
   who gets it, under CAL or a "Compatible Open Source License" (§4.1.2) — a
   licence OSI accepts that allows the two to be distributed as one work.
   **Apache-2.0 qualifies**, and Hearth's repository is public, so this is met
   in substance. It should still be said out loud rather than left to be
   inferred.

The practical effect, and it is worth understanding rather than fearing:
**nobody can take Hearth's built app closed-source.** The interface is
Apache-2.0 and could be taken proprietary on its own, but anything shipping
Holochain or built with the HDK carries CAL's conditions with it. For a care
record meant never to have an operator, that is a feature.

## What to do about it

Small, and none of it changes how Hearth works:

1. **Say it in the README**: Hearth's code is Apache-2.0; the built app
   includes Holochain and lair-keystore under CAL-1.0, with a link to their
   source. *(Done, 17 September 2026.)*
2. **Ship a NOTICES file in the installer**, listing Holochain, lair-keystore,
   Electron and Chromium with their licences and source links, and reachable
   from inside the app. *(Not done — needs a change to the desktop app.)*
3. **Put the same links in each release's notes.** *(Not done.)*
4. **Re-read §4.2.2 when encryption is built**, so that changing a key on
   removal never becomes a technical measure denying somebody their own data.
   Removal deletes the removed person's local copy by their own app's
   choice, which is a different thing — but it is close enough to be worth
   checking deliberately.

## Kangaroo, which the installer is built from

`holochain/kangaroo-electron` has **no licence file at all**, and GitHub
reports none. Its loading screen says "Licensed under the Cryptographic
Autonomy License v1.0", which is presumably about the Holochain inside it. A
repository with no licence grants no rights by default, so **the terms on which
Hearth's desktop app may be built from Kangaroo and distributed are not
established.** It is plainly published for people to do exactly that, and
Holochain's own documentation tells you to. That is an argument about intent,
not a licence.

**Worth asking them for a licence file.** It costs them a minute and settles
it. Until then, this is written down rather than assumed.

## Sources

- The licence text shipped in Holochain's repository: [`LICENSE`](https://github.com/holochain/holochain/blob/develop/LICENSE)
- The licence at OSI: <https://opensource.org/license/cal-1-0>
- Crate licences read at the [`holochain-0.7.0`](https://github.com/holochain/holochain/tree/holochain-0.7.0) tag
