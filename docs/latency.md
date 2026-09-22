# Why joining a circle takes about ninety seconds

**Status: research, not conclusions. Nothing here has been measured yet** — see
*What would settle it* at the end. Written by Claude; the source references are
checked against the pinned versions, the explanation of the delay is a
hypothesis.

> ## ⚠️ Measured on 22 September 2026, and it is a different problem
>
> The section below is about two machines *finding* each other. A test that
> day found them finding each other quickly and then taking **about ten
> minutes for the joiner to actually be in the circle** — with a photograph
> and a two-minute video in the record.
>
> **The laptop's log says what happened**, and it is not the bootstrap
> backoff described below:
>
> - **83 gossip rounds timed out**, one roughly every twenty seconds, from
>   18:40:33 to 18:56:30. Sixteen minutes of rounds starting and failing.
> - Every one names the peer as
>   `https://dev-test-bootstrap2.holochain.org:443/…` — the address goes
>   through **Holochain's public development relay**, for two machines on the
>   same home wifi.
> - `ERROR integrate_dht_ops_consumer: database is locked` during a
>   fifty-second stall, so the workflow that files arriving data was fighting
>   for the database as well.
>
> **The cause is not that the video is large. It is that a large payload
> cannot finish inside a twenty-second gossip round over a relay**, so each
> attempt restarted instead of completing. Size and timeout together.
>
> Three levers, in the order they are worth trying:
>
> 1. **Find out why it relayed at all.** Two machines on one network should
>    connect directly. If antivirus TLS inspection or NAT prevents
>    hole-punching, that is the whole problem and the rest is unnecessary.
> 2. **Stop making a joiner take the media.** A cell can run at
>    `target_arc_factor: 0` — read without storing. A professional's laptop
>    should not be accumulating copies of somebody's family photographs in any
>    case.
> 3. **Smaller pieces.** Three megabytes per piece is fixed in the rules, so it
>    can only change in this migration batch, before release. If a round cannot
>    move 3 MB in twenty seconds, it may move 512 KB.
>
> The rest of this page stands: it is about a different part of the same
> journey, and that part behaved well in the same test.

Prompted by an observation from walking the app: after Dave pastes his
invitation, it takes roughly a minute and a half before Pam's machine knows
anything about him. The fair objection alongside it: Volla phones were making
*phone calls* over Holochain 0.6.1, so this is not a platform that is
inherently slow, and we have not tuned a single thing.

## What is actually slow

Almost certainly **peer discovery**, not messaging.

Every circle is a clone, so **every circle is its own DHT space with nobody in
it**. When Dave joins he publishes himself to the bootstrap server. Pam does
not find out because she is told — she finds out because she *asks*, and she
is not asking very often by then.

`CoreBootstrapConfig` (kitsune2_core 0.5.1, `factories/core_bootstrap.rs`):

| setting | default |
| --- | --- |
| `backoff_min_ms` | 5 seconds |
| `backoff_max_ms` | 5 minutes |

Pam's node has been sitting in an empty space finding nobody, so its poll
backoff has already grown well past five seconds by the time Dave arrives. She
is not slow to react to him; she is not looking yet. **That is the best
candidate for most of the ninety seconds.**

## What is not the problem

**Gossip cadence.** `K2GossipConfig` (kitsune2_gossip 0.5.1, `config.rs`) has a
steady-state `initiate_interval_ms` of 2 minutes, which sounds damning, but a
node doing initial sync uses `initial_initiate_interval_ms` — **1 second** —
and `initiate_burst_factor` of 3 exists precisely so a node that has just
joined can sync faster than the normal rate limit allows. Once two peers know
about each other, this is built to be quick.

**Publishing.** The publish workflow is triggered directly by
`call_zome_workflow` (holochain 0.7.0,
`core/workflow/call_zome_workflow.rs`), so writing something pushes it out at
once. There is also a 60s–5min background loop, but that is a backstop, not the
main path. The 5-minute `min_publish_interval` rate-limits *re*-publishing the
same op; it does not delay the first one.

**So once people have found each other, updates should be quick** — seconds,
not minutes. Two independent mechanisms deliver them: the author publishes to
the authorities, and this app's own reads (`GetStrategy::Network`) ask the
network directly every time it looks. Gossip is the safety net under both.

## Why the Volla comparison is not like for like

A call is between two peers who have **already** found each other and hold an
open connection; the audio never touches the DHT. What we are timing is two
strangers meeting in a space that came into existence seconds ago. A call never
pays this cost, and neither do we — but we pay it once per circle, at exactly
the moment somebody is watching to see whether their invitation worked.

## The levers, and where they live

The conductor's `advanced` network config **is** the kitsune module-config map
— our own running conductor shows `irohTransport` sitting in it. The keys are
camelCase (`#[serde(rename_all = "camelCase")]` on both config structs), so:

```jsonc
// conductor-config.yaml -> network.advanced
{
  "coreBootstrap": { "backoffMinMs": 5000, "backoffMaxMs": 30000 },
  "k2Gossip":      { "initiateIntervalMs": 15000 }
}
```

Nothing in Holochain needs patching to try this.

## Two cautions

**Measure before tuning.** The backoff explanation above is the best-supported
hypothesis, not a measured fact. Tuning against a guess is how you end up with
numbers that are worse and a story about why they should be better.

**Do not tune the demo into a lie.** Aggressive settings would make this feel
instant on one laptop and would tell us nothing about a district nurse's phone
on mobile data. Defaults exist to protect batteries and bandwidth on real
networks. If we tune, it should be an explicit demo profile *next to* a
realistic one, and any claim we make in public should quote the realistic one.

## The cost that tuning will not remove

Clone-per-circle means **every new circle pays full discovery**. That is the
price of the design that makes circles cheap to re-form — which is what the
answer to revocation rests on. It is a trade to state plainly, not a bug to
fix.

## What would settle it

A timestamped trace of one join, which the conductor logs already carry at
debug level:

1. Dave's agent info reaches the bootstrap server
2. Pam's node next polls the bootstrap server ← *the suspected wait*
3. Pam learns Dave's agent info
4. First gossip round between them
5. Dave's introduction becomes readable on Pam's machine

If the gap is between 1 and 3, it is discovery and the bootstrap backoff is the
lever. If it is between 3 and 5, it is gossip and the answer is elsewhere.
Until that trace exists, this document is a hypothesis with references.
