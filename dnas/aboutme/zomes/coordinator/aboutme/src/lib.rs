//! Coordinator zome for About Me.
//!
//! Everything here runs locally, on the machine of whoever calls it.
//! There is no server. If every device in a circle is switched off, the
//! circle is simply not reachable — see the open question in README.md,
//! which is the thing to resolve before this becomes a product.

use aboutme_integrity::*;
use hdk::prelude::*;
use std::collections::BTreeSet;

const CIRCLE_ANCHOR: &str = "circle";

/*
 * Anchors are asked about locally, never over the network.
 *
 * `TypedPath::ensure` writes the anchor link if it is not there already, and
 * the "already" is a `get_links`, which by default asks the network. On a
 * cell seconds old that is a question with nobody to answer it: the call
 * blocks and then fails with
 *
 *     get_links response channel dropped: likely response timeout
 *
 * and everything the function had written is rolled back with it.
 *
 * Which is exactly what happened to the third person to arrive. They pasted
 * the address, their app made the room cell, they knocked — and the knock
 * died inside `ensure` before it was ever written, in front of a wasm error
 * naming a host function. Their conductor could read that room perfectly
 * well a minute later. It simply had no peers yet at the moment it asked.
 *
 * The network was never needed for this. The anchor is a fixed hash; a link
 * from it is found by `get_links` whether or not anybody else has written
 * one, and the duplicates this can leave are the same path-to-path links
 * every reader here already ignores. So each agent answers "is it there?"
 * about their own chain, which is the rule the rest of this file already
 * follows: never ask the network what you can know yourself.
 */
fn anchored(name: &str, link_type: LinkTypes) -> ExternResult<TypedPath> {
    Ok(Path::from(name)
        .typed(link_type)?
        .with_strategy(GetStrategy::Local))
}

fn circle_path() -> ExternResult<TypedPath> {
    anchored(CIRCLE_ANCHOR, LinkTypes::CircleToAboutMe)
}

/// Issue an invitation to join this circle.
///
/// Only meaningful when called by the founder: anyone else can call it, but the
/// signature will not verify and the invitation will not let anybody in. There
/// is no permission check here because there is nowhere to enforce one — the
/// enforcement lives in every peer's copy of the validation rules.
///
/// The result is handed to the invitee out of band (a link, a QR code, read
/// aloud over the phone) and presented as their membrane proof on joining.
/// Everything the invited person needs, in one piece.
///
/// Safe to send by any means — text message, email, read aloud. The signature
/// is over the invitee's own key, so it admits nobody else. An interceptor
/// gains a useless blob.
#[derive(Serialize, Deserialize, Debug)]
pub struct InvitationBundle {
    /// Base64, not raw bytes. This bundle is meant to be copied into a text
    /// message, so every field in it has to survive being text.
    pub founder: String,
    /// Who made this invitation, in their own words.
    ///
    /// Their own introduction to this circle, not a name the app assigned.
    /// Empty where they have not introduced themselves.
    ///
    /// It is here because of the person who has to agree to the invitation.
    /// Being asked to sign somebody in is a decision, and it was impossible to
    /// make: the screen could say which circle and which key, but not who was
    /// asking. "Somebody, possibly, wants to let this string of characters in"
    /// is not something anybody can sensibly agree to.
    ///
    /// Unverified, like every other name in this app, and the interface must
    /// say so rather than present it as established.
    #[serde(default)]
    pub inviter: String,

    /// Who this invitation is for.
    ///
    /// Carried for the person who has to agree to it. They are being asked to
    /// sign somebody into a circle, and "sign this, never mind who" is not a
    /// safeguard — it is a rubber stamp with extra steps. It is also simply
    /// what they need in order to sign at all: the signature is over this key.
    pub invitee: String,

    /// Who the person inviting says that key belongs to.
    ///
    /// A key identifies nobody. It is a long line of characters anybody can
    /// generate, and no care taken here changes that — tying a key to a named
    /// human being is what a certificate authority does, and a certificate
    /// authority is an operator, which is the thing this project does not
    /// have.
    ///
    /// So this is not identification and must never be shown as though it
    /// were. It is the holder's own claim about who she is letting in, and
    /// that turns out to be exactly what the second person needs.
    ///
    /// The case the second yes exists for is somebody being talked into
    /// admitting a stranger. Judging that means judging *her* — "she says this
    /// is Ronnie, her cousin" — against which the honest answers are "yes, I
    /// know Ronnie" and "who?". Without a name there is nothing to answer at
    /// all, and agreeing means agreeing that she asked, which safeguards
    /// nobody.
    ///
    /// **Not signed**, which is a real limit rather than an oversight. Signing
    /// it would mean putting it inside the membrane proof, and that lives in
    /// the frozen integrity zome. What a forged label cannot do is change who
    /// gets in: the signature is over the key, so altering the name misleads
    /// the reader without admitting anybody the holder had not already signed
    /// for. See `docs/to-a-product.md`.
    #[serde(default)]
    pub invitee_name: String,

    /// Whoever this circle asks to agree as well, if it asks anybody. It forms
    /// part of the DNA hash, so it has to travel with the invitation or the
    /// joiner computes a different circle and lands nowhere.
    #[serde(default)]
    pub seconder: Option<String>,

    /// Whether this circle asks two people to agree before anybody joins.
    ///
    /// **This is the part that is in the DNA hash**, and it has to travel or
    /// the person joining computes a different circle and lands in a network
    /// of one — with no error anywhere, because nothing is wrong except that
    /// they are somewhere else.
    ///
    /// `seconder` above used to do this job, back when the person was in the
    /// identity and therefore always present. Once the person moved into the
    /// circle, an invitation made before anybody had been appointed carried
    /// nobody — and the joiner quite correctly built a circle that asks
    /// nobody. Two circles, one name, both working perfectly, invisible to
    /// each other for ten minutes until somebody thought to check.
    #[serde(default)]
    pub requires_second_yes: bool,

    pub network_seed: String,
    /// Whose circle this is, so the recipient knows what they are accepting
    /// before they accept it. A label, not a claim.
    pub about: String,
    pub invitation: Invitation,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InviteInput {
    /// Their identifier as they sent it to you: base64 text beginning
    /// `uhCAk`, and not a name.
    pub invitee: String,

    /// What you call the person that identifier belongs to.
    ///
    /// Your claim, not a fact, and nothing anywhere checks it. See
    /// `invitee_name` on the bundle for why it is worth carrying anyway.
    /// Optional: an invitation with no name still works, it is just harder for
    /// the second person to judge.
    #[serde(default)]
    pub name: String,
}

#[hdk_extern]
pub fn invite(input: InviteInput) -> ExternResult<InvitationBundle> {
    let invitee = AgentPubKey::try_from(input.invitee.trim()).map_err(|_| {
        wasm_error!(
            "That does not look like somebody's identifier. It is a long line of              letters and numbers beginning uhCAk, which they can copy from their              own copy of Hearth. It is not their name."
        )
    })?;

    let me = agent_info()?.agent_initial_pubkey;
    let signature = sign(me.clone(), invitee.clone())?;

    let about = match get_circle_about_me(())?.first() {
        Some(original) => get_current_about_me(original.clone())?
            .record
            .and_then(|r| r.entry().as_option().cloned())
            .and_then(|e| AboutMe::try_from(e).ok())
            .map(|a| a.display_name)
            .unwrap_or_default(),
        None => String::new(),
    };

    let appointed = appointment_now()?;
    let seconder = appointed.as_ref().map(|(_, key)| key.to_string());

    // The rule this circle was made with. It is in the DNA hash, so it has to
    // travel with the invitation or the joiner builds a different circle.
    let asks_two = matches!(membrane()?, Membrane::Founder(_, true));

    // My own introduction, off my own chain: what I told this circle I am
    // called. Read locally because it is mine, and empty if I never said.
    let inviter = on_my_own_chain(UnitEntryTypes::Member)?
        .last()
        .and_then(|record| record.entry().to_app_option::<Member>().ok().flatten())
        .map(|m| m.name)
        .unwrap_or_default();

    Ok(InvitationBundle {
        founder: me.to_string(),
        inviter,
        invitee: invitee.to_string(),
        invitee_name: input.name.trim().to_string(),
        seconder,
        requires_second_yes: asks_two,
        network_seed: dna_info()?.modifiers.network_seed,
        about,
        invitation: Invitation {
            signature,
            // Not yet. Where somebody has been asked to agree, this
            // invitation is incomplete until they do.
            seconded: None,
            appointment: appointed.map(|(hash, _)| hash),
        },
    })
}

/// Add the second agreement to an invitation somebody else has started.
///
/// Called by whoever the circle named as its seconder, on their own machine,
/// with the invitee's identifier in front of them. They are signing the same
/// thing the holder signed — that person's key, and nothing else.
///
/// Deliberately not a "vote" or an "approval workflow". There is nothing to
/// approve and nobody to approve it to: two people sign, or the invitation
/// does not exist. And deliberately not a veto, which is what RIX Multi Me's
/// Buddy has. A veto must arrive in time to stop something already moving; a
/// signature that was never given stops nothing because nothing started.
///
/// Note this does not check who is calling. It cannot usefully: anybody may
/// sign anything, and a signature from the wrong key simply fails at the door
/// like any other. The check that matters is in the membrane, where every peer
/// makes it independently.
#[hdk_extern]
pub fn second_an_invitation(invitee: String) -> ExternResult<Signature> {
    let invitee = AgentPubKey::try_from(invitee.trim())
        .map_err(|_| wasm_error!("That is not an identifier this circle can read"))?;

    let me = agent_info()?.agent_initial_pubkey;
    sign(me, invitee)
}

/// Whether this circle asks two people to agree, and who the second is.
///
/// Read from the DNA rather than stored anywhere, because it is part of what
/// this circle *is*. An interface needs it to know whether an invitation it
/// has just made is finished or half-made.
#[hdk_extern]
pub fn who_seconds_here(_: ()) -> ExternResult<Option<AgentPubKey>> {
    Ok(appointment_now()?.map(|(_, key)| key))
}

/// Who you are in this circle, in your own words.
///
/// Nothing here is verified. "Her son" is a claim, exactly like a
/// professional's role on an acknowledgement, and the interface must not
/// dress it up as anything more.
#[hdk_extern]
pub fn introduce_myself(member: Member) -> ExternResult<Record> {
    // Before writing, so the count below does not include this one.
    let said_before = !on_my_own_chain(UnitEntryTypes::Member)?.is_empty();

    let action_hash = create_entry(EntryTypes::Member(member.clone()))?;

    let path = anchored("members", LinkTypes::CircleToMember)?;
    path.ensure()?;
    create_link(
        path.path_entry_hash()?,
        action_hash.clone(),
        LinkTypes::CircleToMember,
        (),
    )?;

    // Tell the holder her invitation was taken up. Fire and forget: the
    // introduction is written either way, and a signal that does not arrive
    // must never be the difference between somebody being in the circle and
    // not. The list on her screen is the record of who is here; this only
    // saves her going to look.
    if let Membrane::Founder(founder, _) = membrane()? {
        let me = agent_info()?.agent_initial_pubkey;
        if founder != me {
            let _ = send_remote_signal(
                Signal::Introduced {
                    by: me,
                    name: member.name,
                    relationship: member.relationship,
                    joined: !said_before,
                },
                vec![founder],
            );
        }
    }

    get(action_hash, GetOptions::default())?
        .ok_or_else(|| wasm_error!("Could not read the introduction just written"))
}

// ---------------------------------------------------------------------------
// One rule, applied everywhere below it
// ---------------------------------------------------------------------------

/*
 * What I wrote myself is never a question for the network.
 *
 * Every list in this zome is assembled from links fetched with
 * `GetStrategy::Network`. That is right for other people's contributions and
 * wrong for my own: for the first seconds after I write something the network
 * has not heard of it, so the screen tells me I never did it. That produced,
 * in turn, a "Who are you?" form on the page I had just filled in, and
 * "Nothing has been written yet" printed above "Read this over".
 *
 * They were fixed one at a time until it was obvious they were one fault. So
 * the answer is not another special case: every list here reads the network
 * for everybody, and my own chain for me, and merges the two.
 */

/// Fetch many records in one go, dropping any that cannot be found.
///
/// **This is the difference between one wait and thirty.**
///
/// Asking for records one at a time in a loop is fine on the machine of
/// somebody who holds the whole circle, because every answer is already on
/// their own disk. It is not fine for the person this design is really for: a
/// professional set up to read without storing anything holds none of it, so
/// every single `get` in a loop is a separate trip out to somebody else's
/// device, one after another, each waiting for the last.
///
/// A circle with ten suggestions and five revisions was roughly forty of those
/// trips to draw one screen — and the screen redraws every twenty seconds.
///
/// Holochain's own interface takes a whole list at once and answers them
/// together, which is what this uses. The single-hash `get` in the library is a
/// convenience wrapper over exactly the same call.
///
/// Records that come back missing are dropped rather than reported. A record
/// nobody can currently reach is the ordinary condition of a circle where
/// somebody's laptop is shut, not an error worth a screen.
fn get_many(hashes: Vec<ActionHash>) -> ExternResult<Vec<Record>> {
    if hashes.is_empty() {
        return Ok(Vec::new());
    }

    let inputs: Vec<GetInput> = hashes
        .into_iter()
        .map(|hash| GetInput::new(hash.into(), GetOptions::default()))
        .collect();

    Ok(HDK
        .with(|h| h.borrow().get(inputs))?
        .into_iter()
        .flatten()
        .collect())
}

/// Everything of one entry type on my own chain, oldest first.
fn on_my_own_chain(entry_type: UnitEntryTypes) -> ExternResult<Vec<Record>> {
    query(
        ChainQueryFilter::new()
            .entry_type(entry_type.try_into()?)
            .include_entries(true),
    )
}

/// Add anything of mine the network did not know about yet.
///
/// Appended rather than prepended: where a list is read latest-wins, what I
/// have just written should be the latest.
fn and_my_own(records: &mut Vec<Record>, mine: Vec<Record>) {
    let already: BTreeSet<ActionHash> =
        records.iter().map(|r| r.action_address().clone()).collect();

    for record in mine {
        if !already.contains(record.action_address()) {
            records.push(record);
        }
    }
}

/// Put records in an order every device agrees on: oldest first.
///
/// **The order links come back in is not promised to be the same anywhere.**
/// That is already written down against `order_versions` in the integrity
/// crate, and it applies just as much to any other list assembled from links.
///
/// It matters wherever a reader takes the last of something as the one that
/// counts. Somebody who corrects how they describe themselves has two
/// introductions in the circle, and without this, which one is shown is
/// whichever happened to arrive last — so her son could appear as "nephew" on
/// one person's screen and "son" on another's, with both of them right about
/// what they were sent.
///
/// Sorted by when it was written, with the action's own hash as the tiebreak
/// for the case where two share a timestamp. Arbitrary, but identical
/// everywhere, which is the only property being asked for.
fn oldest_first(records: &mut [Record]) {
    records.sort_by(|a, b| {
        a.action()
            .timestamp()
            .cmp(&b.action().timestamp())
            .then_with(|| a.action_address().cmp(b.action_address()))
    });
}

#[hdk_extern]
pub fn get_members(_: ()) -> ExternResult<Vec<Record>> {
    let path = anchored("members", LinkTypes::CircleToMember)?;
    let links = get_links(
        LinkQuery::try_new(path.path_entry_hash()?, LinkTypes::CircleToMember)?,
        GetStrategy::Network,
    )?;

    let mut out = get_many(
        links
            .into_iter()
            .filter_map(|l| l.target.into_action_hash())
            .collect(),
    )?;

    // Including me, so I am not a stranger in a circle I just introduced
    // myself to.
    and_my_own(&mut out, on_my_own_chain(UnitEntryTypes::Member)?);

    // A reader takes the last introduction from each person as the one that
    // counts, so the order has to be the same on every device.
    oldest_first(&mut out);
    Ok(out)
}

#[hdk_extern]
pub fn create_about_me(about_me: AboutMe) -> ExternResult<Record> {
    let action_hash = create_entry(EntryTypes::AboutMe(about_me))?;

    let path = circle_path()?;
    path.ensure()?;
    create_link(
        path.path_entry_hash()?,
        action_hash.clone(),
        LinkTypes::CircleToAboutMe,
        (),
    )?;

    get(action_hash, GetOptions::default())?
        .ok_or_else(|| wasm_error!("Could not read the About Me just created"))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateAboutMeInput {
    /// The original create, which is the stable identity of this About Me.
    pub original_action_hash: ActionHash,
    /// The version being replaced (the head as the caller last saw it).
    pub previous_action_hash: ActionHash,
    pub about_me: AboutMe,
}

#[hdk_extern]
pub fn update_about_me(input: UpdateAboutMeInput) -> ExternResult<Record> {
    let updated = update_entry(input.previous_action_hash, &input.about_me)?;

    create_link(
        input.original_action_hash,
        updated.clone(),
        LinkTypes::AboutMeUpdates,
        (),
    )?;

    get(updated, GetOptions::default())?
        .ok_or_else(|| wasm_error!("Could not read the update just written"))
}

/// The original About Me records in this circle (normally exactly one).
#[hdk_extern]
pub fn get_circle_about_me(_: ()) -> ExternResult<Vec<ActionHash>> {
    let path = circle_path()?;
    let links = get_links(
        LinkQuery::try_new(path.path_entry_hash()?, LinkTypes::CircleToAboutMe)?,
        GetStrategy::Network,
    )?;

    let mut originals: Vec<ActionHash> = links
        .into_iter()
        .filter_map(|l| l.target.into_action_hash())
        .collect();

    // Mine too, if I am the one this circle is about. The create, not the
    // updates: this is the record's identity, and it has exactly one.
    for record in on_my_own_chain(UnitEntryTypes::AboutMe)? {
        if !matches!(record.action().data, ActionData::Create(_)) {
            continue;
        }
        let hash = record.action_address().clone();
        if !originals.contains(&hash) {
            originals.push(hash);
        }
    }

    Ok(originals)
}

/// Every version of an About Me, oldest first.
///
/// There is no global clock and no total order. Two people editing while apart
/// both produce valid versions, and neither "won" — so the honest primitive is
/// the list, and anything that picks one is a display choice layered on top.
#[hdk_extern]
pub fn get_about_me_versions(original_action_hash: ActionHash) -> ExternResult<Vec<ActionHash>> {
    let links = get_links(
        LinkQuery::try_new(original_action_hash.clone(), LinkTypes::AboutMeUpdates)?,
        GetStrategy::Network,
    )?;

    // The ordering lives in the integrity crate as a pure function so it can be
    // unit tested directly — in particular the tie case, which cannot be
    // provoked through a conductor because timestamps cannot be made to collide
    // on demand. See `order_versions`.
    let mut updates: Vec<(Timestamp, ActionHash)> = links
        .into_iter()
        .filter_map(|l| l.target.into_action_hash().map(|hash| (l.timestamp, hash)))
        .collect();

    // My own edits, which the network has not necessarily heard about yet.
    // Without these, correcting a line and looking straight at it showed the
    // version before the correction.
    //
    // Followed from the original outwards rather than taken wholesale, so an
    // edit belongs to this record because it can be traced to it, not because
    // it happens to be on my chain.
    let mine = on_my_own_chain(UnitEntryTypes::AboutMe)?;
    let mut belongs: BTreeSet<ActionHash> = BTreeSet::from([original_action_hash.clone()]);
    let mut grew = true;
    while grew {
        grew = false;
        for record in &mine {
            let hash = record.action_address().clone();
            if belongs.contains(&hash) {
                continue;
            }
            let ActionData::Update(update) = &record.action().data else {
                continue;
            };
            if belongs.contains(&update.original_action_address) {
                belongs.insert(hash.clone());
                if !updates.iter().any(|(_, h)| h == &hash) {
                    updates.push((record.action().timestamp(), hash));
                }
                grew = true;
            }
        }
    }

    let mut versions = vec![original_action_hash];
    versions.extend(order_versions(updates));
    Ok(versions)
}

/// The version to show, and whether showing one is misleading.
#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentAboutMe {
    pub record: Option<Record>,
    /// How many versions have nothing written on top of them.
    ///
    /// One is the ordinary case however many times the record has been
    /// edited, because edits made one after another form a chain and only the
    /// last link of a chain is loose. More than one means the chain forked:
    /// two people wrote while their devices were apart, and neither knew about
    /// the other. That, and only that, is worth telling somebody about.
    ///
    /// This used to be the total number of versions, which meant writing a
    /// record and then correcting it — the most ordinary thing anyone does
    /// here — announced that it "was changed in 2 places while devices were
    /// apart". It was the software inventing a disagreement between a person
    /// and herself.
    pub divergent_versions: usize,
}

#[hdk_extern]
pub fn get_current_about_me(original_action_hash: ActionHash) -> ExternResult<CurrentAboutMe> {
    let versions = get_about_me_versions(original_action_hash)?;
    let newest = versions
        .last()
        .cloned()
        .ok_or_else(|| wasm_error!("An About Me always has at least its original version"))?;

    // Every version fetched together rather than one after another. This used
    // to be a loop, which meant one trip across the network per revision for
    // anybody who does not hold the circle themselves. See `get_many`.
    let fetched = get_many(versions.clone())?;

    // Every update names the version it replaced. Collect those names and the
    // loose ends are whatever is left over — the versions nothing was built
    // on top of.
    let mut replaced: BTreeSet<ActionHash> = BTreeSet::new();
    for record in &fetched {
        if let ActionData::Update(update) = &record.action().data {
            replaced.insert(update.original_action_address.clone());
        }
    }

    // A record with no updates at all has one loose end: the original.
    let divergent_versions = versions
        .iter()
        .filter(|hash| !replaced.contains(hash))
        .count()
        .max(1);

    Ok(CurrentAboutMe {
        // Already in hand from the batch above, so no second trip for it.
        record: fetched.into_iter().find(|r| r.action_address() == &newest),
        divergent_versions,
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AcknowledgeInput {
    pub about_me: ActionHash,
    pub role: String,
}

/// The whole professional workflow: one tap.
#[hdk_extern]
pub fn acknowledge(input: AcknowledgeInput) -> ExternResult<Record> {
    let role = input.role.clone();
    let ack = Acknowledgement {
        about_me: input.about_me.clone(),
        role: input.role,
    };
    let action_hash = create_entry(EntryTypes::Acknowledgement(ack))?;

    create_link(
        input.about_me.clone(),
        action_hash.clone(),
        LinkTypes::AboutMeToAcknowledgement,
        (),
    )?;

    // Tell the holder someone has read it. Fire and forget, and deliberately
    // unable to fail the write: the acknowledgement on the chain is the
    // evidence, and this is only the nudge. A family should never lose a record
    // that somebody read the notes because a phone happened to be off.
    if let Membrane::Founder(founder, _) = membrane()? {
        let me = agent_info()?.agent_initial_pubkey;
        if founder != me {
            let _ = send_remote_signal(
                Signal::Acknowledged {
                    about_me: input.about_me,
                    by: me,
                    role,
                },
                vec![founder],
            );
        }
    }

    get(action_hash, GetOptions::default())?
        .ok_or_else(|| wasm_error!("Could not read the acknowledgement just written"))
}

/// Who has read a given version of About Me.
#[hdk_extern]
pub fn get_acknowledgements(about_me: ActionHash) -> ExternResult<Vec<Record>> {
    let wanted = about_me.clone();
    let links = get_links(
        LinkQuery::try_new(about_me, LinkTypes::AboutMeToAcknowledgement)?,
        GetStrategy::Network,
    )?;

    let mut records = get_many(
        links
            .into_iter()
            .filter_map(|l| l.target.into_action_hash())
            .collect(),
    )?;

    // Including my own "I have read this", so pressing it visibly does
    // something. Only the ones about the version asked for.
    let mut mine = on_my_own_chain(UnitEntryTypes::Acknowledgement)?;
    mine.retain(|record| {
        record
            .entry()
            .to_app_option::<Acknowledgement>()
            .ok()
            .flatten()
            .is_some_and(|a| a.about_me == wanted)
    });
    and_my_own(&mut records, mine);

    // Who read it, in the order they read it, the same way on every device.
    oldest_first(&mut records);

    Ok(records)
}

/// Delete a record. Validation permits this only to the record's own author,
/// so a member cannot erase the person's About Me.
#[hdk_extern]
pub fn delete_about_me(action_hash: ActionHash) -> ExternResult<ActionHash> {
    delete_entry(action_hash)
}

// ---------------------------------------------------------------------------
// Circles: one isolated network per person
// ---------------------------------------------------------------------------
//
// This is the architecture rather than a configuration detail. A circle is a
// cloned cell whose DNA properties name the person it belongs to. Because the
// properties form part of the DNA hash, a different person means a different
// hash means a genuinely separate network. Circles cannot see one another as a
// fact about the maths, not as an access rule someone could get wrong.
//
// Creating and joining are the same operation. The only difference is that a
// joiner presents an invitation.

/// The circle's identity, which every member must compute identically.
///
/// **Everything here is in the DNA hash.** Two people who build these
/// differently are in two different networks that can never see each
/// other — and it fails in the worst way there is: invitations are made
/// and accepted, nothing errors, and nobody ever arrives.
///
/// It takes the *rule* and not the person, because that is what the
/// identity carries now. Passing the person instead was exactly the fault
/// above: the holder made a circle that asks two people, the invitation
/// carried nobody because nobody had been appointed yet, and the joiner
/// computed a circle that asks nobody. Two circles, one name, no error.
fn circle_modifiers(
    founder: &AgentPubKey,
    requires_second_yes: bool,
    network_seed: String,
) -> ExternResult<DnaModifiersOpt<YamlProperties>> {
    // Part of the DNA hash, exactly like the founder. Naming somebody who must
    // also agree is therefore a different circle, not a setting inside this
    // one — which is the point. A rule the holder can switch off alone is no
    // protection for a holder who is being leaned on.
    let properties = CircleProperties {
        founder: Some(founder.to_string()),
        lobby: false,
        // Never named here any more. A circle made this way asks two
        // people to agree; who the second is, is written in the circle and
        // can be written again.
        seconder: None,
        requires_second_yes,
        // A circle is not a waiting room. Its own room is a separate cell,
        // and it is the only thing anybody outside can reach.
        waiting_for: None,
    };

    // Clone modifiers arrive as YAML, which is why the founder is carried as a
    // base64 string rather than raw key bytes.
    let yaml = yaml_serde::to_value(&properties).map_err(|e| {
        wasm_error!(format!(
            "Could not express the circle's properties as YAML: {e}"
        ))
    })?;

    Ok(DnaModifiersOpt::none()
        .with_network_seed(network_seed)
        .with_properties(YamlProperties::new(yaml)))
}

fn this_cell() -> ExternResult<CellId> {
    Ok(CellId::new(
        dna_info()?.hash,
        agent_info()?.agent_initial_pubkey,
    ))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateCircleInput {
    /// Who will hold this circle, as base64 text. Normally the caller.
    pub founder: String,
    /// A human name for this circle, shown in the app. Not part of the DNA
    /// hash, so two people may safely use the same word.
    pub name: String,
    /// Makes this circle distinct from any other for the same person.
    pub network_seed: String,
    /// Whether this circle asks two people to agree before anybody joins.
    ///
    /// The rule, not the person — who agrees is written inside the circle
    /// afterwards. This forms part of the identity and the person does not.
    #[serde(default)]
    pub requires_second_yes: bool,
}

/// Bring a new circle into being.
#[hdk_extern]
pub fn create_circle(input: CreateCircleInput) -> ExternResult<ClonedCell> {
    let founder = AgentPubKey::try_from(input.founder.trim())
        .map_err(|_| wasm_error!("That is not a valid identifier for the holder"))?;

    create_clone_cell(CreateCloneCellInput {
        cell_id: this_cell()?,
        modifiers: circle_modifiers(&founder, input.requires_second_yes, input.network_seed)?,
        membrane_proof: None,
        name: Some(input.name),
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JoinCircleInput {
    /// The holder, as base64 text out of the invitation. Must match what the
    /// inviter used, or the DNA hash differs and you land in a different
    /// network entirely.
    pub founder: String,
    pub name: String,
    pub network_seed: String,
    /// From the founder's `invite`, signed over the joiner's own key.
    pub invitation: Invitation,
    /// Whether the circle asks two people to agree before anybody joins.
    ///
    /// Part of the DNA hash, so a joiner who gets it wrong computes a
    /// different circle and lands nowhere — silently, which is why it
    /// travels in the invitation rather than being guessed at.
    ///
    /// It used to be the second person's key doing this job, back when the
    /// person was in the identity. They are separate things now, and an
    /// invitation carrying only the person left the joiner computing a
    /// circle that asks nobody.
    #[serde(default)]
    pub requires_second_yes: bool,

    /// Who the circle had appointed when this invitation was made.
    ///
    /// **Not part of the identity, and must never be passed to
    /// `circle_modifiers`.** It is here only so that a plainly wrong
    /// invitation can be refused before a cell is built from it — see the note
    /// in `join_circle` about the dead circle called "Auntie Marge".
    ///
    /// Keeping it beside the flag is uncomfortable, because confusing the two
    /// is exactly the fault this commit fixes. The comment is the guard.
    #[serde(default)]
    pub seconder: Option<String>,
}

/// Join a circle you have been invited to.
///
/// Note what is *not* here: no request, no approval step, nobody to ask. The
/// invitation is the whole of it, and every existing member checks it
/// independently.
#[hdk_extern]
pub fn join_circle(input: JoinCircleInput) -> ExternResult<ClonedCell> {
    let founder = AgentPubKey::try_from(input.founder.trim())
        .map_err(|_| wasm_error!("That invitation is damaged: the holder is unreadable"))?;

    /*
     * Check the invitation before building anything with it.
     *
     * The membrane is the real gate and stays the real gate: every peer checks
     * it, and nothing here can be skipped by a modified client. This is about
     * what a failed attempt leaves behind on the machine that made it.
     *
     * A clone is created first and genesis runs second, so an invitation that
     * fails the membrane still leaves the cell registered — an enabled circle
     * in somebody's list that they are not a member of, cannot read, and
     * cannot join properly afterwards, because the cell id is now taken and
     * every retry collides with it. Found by trying it: joining with half an
     * invitation left a circle called "Auntie Marge" that could never work.
     *
     * So the same two signatures are verified here, where failing costs
     * nothing and can say something useful.
     */
    let me = agent_info()?.agent_initial_pubkey;

    if !verify_signature(
        founder.clone(),
        input.invitation.signature.clone(),
        me.clone(),
    )? {
        return Err(wasm_error!(
            "This invitation was not made for you, or not by the person whose \
             circle it is. Ask them to make one from your identifier."
        ));
    }

    if let Some(seconder) = input.seconder.as_deref() {
        let seconder = AgentPubKey::try_from(seconder.trim()).map_err(|_| {
            wasm_error!("That invitation is damaged: the second agreement names nobody readable")
        })?;

        // The seconder needs no second agreement to their own admission; see
        // the membrane, which is where this is actually enforced.
        if me == seconder {
            let proof = SerializedBytes::try_from(input.invitation)
                .map(MembraneProof::new)
                .map_err(|e| wasm_error!(format!("Could not read that invitation: {e:?}")))?;

            return create_clone_cell(CreateCloneCellInput {
                cell_id: this_cell()?,
                modifiers: circle_modifiers(
                    &founder,
                    input.requires_second_yes,
                    input.network_seed,
                )?,
                membrane_proof: Some(proof),
                name: Some(input.name),
            });
        }

        let Some(seconded) = input.invitation.seconded.clone() else {
            return Err(wasm_error!(
                "This invitation is not finished. This circle asks two people to \
                 agree before anybody joins, and only one of them has. Send it \
                 back to whoever invited you."
            ));
        };

        if !verify_signature(seconder, seconded, me)? {
            return Err(wasm_error!(
                "The second agreement on this invitation is not from the person \
                 this circle asks to give it."
            ));
        }
    }

    let proof = SerializedBytes::try_from(input.invitation)
        .map(MembraneProof::new)
        .map_err(|e| wasm_error!(format!("Could not read that invitation: {e:?}")))?;

    create_clone_cell(CreateCloneCellInput {
        cell_id: this_cell()?,
        modifiers: circle_modifiers(&founder, input.requires_second_yes, input.network_seed)?,
        membrane_proof: Some(proof),
        name: Some(input.name),
    })
}

/// Take a circle off this device.
///
/// Deliberately `disable_clone_cell` and not `delete_clone_cell`. Disabling
/// stops the cell running and takes it out of the app's list, which is the
/// whole of what somebody means by "get this off my screen". Deleting would
/// also throw away the local copy for good, and this is not the place to make
/// that decision on their behalf.
///
/// What this does NOT do, and what the interface must not imply it does:
/// nobody else's device is touched, nothing anyone has already read is taken
/// back, and the circle carries on existing for every other member. There is
/// no operator here to reach across and remove anything. Leaving is the only
/// honest verb.
#[hdk_extern]
pub fn leave_circle(dna_hash: DnaHash) -> ExternResult<()> {
    disable_clone_cell(DisableCloneCellInput {
        clone_cell_id: CloneCellId::DnaHash(dna_hash),
    })
}

/// Come back to a circle you took off this device.
///
/// Leaving disables the clone rather than deleting it, so the cell is still
/// there and its id is still taken. Joining again with the same invitation
/// therefore tried to build a cell that already existed and failed with
/// "Tried to create a cell with an existing id" — a wasm error, in front of
/// somebody who had done nothing wrong except change their mind.
///
/// Nothing needs rebuilding. The cell is intact, with everything that was in
/// it; it was only switched off. So this switches it back on.
#[hdk_extern]
pub fn rejoin_circle(dna_hash: DnaHash) -> ExternResult<ClonedCell> {
    enable_clone_cell(EnableCloneCellInput {
        clone_cell_id: CloneCellId::DnaHash(dna_hash),
    })
}

// ---------------------------------------------------------------------------
// Signals: telling someone their record was read, without polling or a server
// ---------------------------------------------------------------------------

/// Sent peer to peer, never stored.
///
/// A signal is not evidence. The acknowledgement written to the chain is the
/// evidence; this is only the nudge that makes it visible without the app
/// having to ask repeatedly whether anything happened.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum Signal {
    /// Somebody read a specific version of the record.
    Acknowledged {
        about_me: ActionHash,
        by: AgentPubKey,
        /// Claimed, never verified. See the note on `Acknowledgement`.
        role: String,
    },
    /// Somebody said who they are — for the first time, or corrected it.
    ///
    /// Sent only to the holder, and deliberately not to the whole circle. She
    /// sent the invitation and is the one waiting to hear whether it worked. A
    /// district nurse in thirty circles does not need telling every time
    /// somebody else's nephew arrives; that is the inbox this is built not to
    /// be.
    Introduced {
        by: AgentPubKey,
        name: String,
        relationship: String,
        /// False when they are correcting what they said before, so the
        /// interface can say "has joined" without ever saying it twice.
        joined: bool,
    },
    /// Somebody offered something for the record.
    Suggested {
        suggestion: ActionHash,
        by: AgentPubKey,
        text: String,
    },
    /// The holder has put somebody forward and is waiting on the second
    /// agreement. Sent only to the person who has to give it.
    Proposed {
        proposed: ActionHash,
        by: AgentPubKey,
        /// What the holder called them. A claim, never checked.
        name: String,
    },
    /// The second agreement has been given. Sent only to the holder, who is
    /// the one waiting on it, and whose screen can now show a finished
    /// invitation instead of a job half done.
    Endorsed {
        proposed: ActionHash,
        by: AgentPubKey,
    },
    /// Somebody is at the door of a waiting room. Sent only to the holder of
    /// the circle it serves, who is the only person who can answer.
    Knocked {
        by: AgentPubKey,
        /// What they call themselves. A claim.
        name: String,
        /// How they say they are connected. A claim.
        relationship: String,
    },
    /// A knock has been answered, so there is an invitation to collect. Sent
    /// only to the person who knocked.
    Admitted { by: AgentPubKey },
    /// The holder has asked somebody to agree to who joins. Sent only to
    /// the person being asked, who until now found out by noticing that
    /// strangers had started appearing on their screen for approval.
    Appointed {
        appointment: ActionHash,
        by: AgentPubKey,
    },
    /// They have said whether they are willing. Sent only to the holder,
    /// who is the one who has to appoint somebody else if they are not.
    Answered { by: AgentPubKey, willing: bool },
}

/// Allow other members of this circle to deliver signals to us.
///
/// This is the one place capability grants genuinely belong. They were once on
/// this project's build order as the route to revocation, which was wrong —
/// they govern who may call into *this* cell, not what somebody already holds.
///
/// `Unrestricted` sounds alarming and is not: the only agents who can reach
/// this cell at all are the ones the membrane already admitted to the circle.
#[hdk_extern]
pub fn init() -> ExternResult<InitCallbackResult> {
    let mut functions = HashSet::new();
    functions.insert((zome_info()?.name, "recv_remote_signal".into()));

    create_cap_grant(CapGrantEntry {
        tag: "circle-signals".into(),
        access: CapAccess::Unrestricted,
        functions: GrantedFunctions::Listed(functions),
    })?;

    Ok(InitCallbackResult::Pass)
}

/// Hand an incoming signal to whatever is showing the circle.
///
/// Every signal says who it is from. That field is filled in by whoever sent
/// it, so on its own it is a claim and not a fact — and the claim is checked
/// here, against who actually called.
///
/// Without this check, any member of a circle could make the holder's screen
/// say "Someone read this. They said they are: district nurse" without ever
/// reading anything, or announce that a person who is not here has joined. The
/// screen would be telling her something untrue about somebody else, which is
/// the one thing this app must never do.
///
/// A signal that does not match is dropped in silence rather than reported.
/// There is nobody to report it to, nothing was written, and a warning about a
/// message she never asked for is not information — it is worry.
#[hdk_extern]
pub fn recv_remote_signal(signal: Signal) -> ExternResult<()> {
    // Who actually made this call. For a signal arriving from another machine
    // this is the sending agent, established by Holochain rather than asserted
    // in the payload.
    let caller = call_info()?.provenance;

    let claimed = match &signal {
        Signal::Acknowledged { by, .. }
        | Signal::Introduced { by, .. }
        | Signal::Suggested { by, .. }
        | Signal::Proposed { by, .. }
        | Signal::Endorsed { by, .. }
        | Signal::Knocked { by, .. }
        | Signal::Admitted { by }
        | Signal::Appointed { by, .. }
        | Signal::Answered { by, .. } => by,
    };

    if claimed != &caller {
        return Ok(());
    }

    emit_signal(signal)
}

// ---------------------------------------------------------------------------
// Suggestions: everyone contributes, one voice remains
// ---------------------------------------------------------------------------
//
// A son remembers what his mother enjoyed. A support worker notices what
// settles her. A record only one person may write loses all of it — so anyone
// in the circle may offer something, and only the holder decides what goes in.

const SUGGESTION_ANCHOR: &str = "suggestions";

fn suggestion_path() -> ExternResult<TypedPath> {
    anchored(SUGGESTION_ANCHOR, LinkTypes::CircleToSuggestion)
}

#[hdk_extern]
pub fn suggest(suggestion: Suggestion) -> ExternResult<Record> {
    let action_hash = create_entry(EntryTypes::Suggestion(suggestion.clone()))?;

    let path = suggestion_path()?;
    path.ensure()?;
    create_link(
        path.path_entry_hash()?,
        action_hash.clone(),
        LinkTypes::CircleToSuggestion,
        (),
    )?;

    // Nudge the holder, the same way an acknowledgement does. Fire and forget:
    // the suggestion is safely written either way.
    if let Membrane::Founder(founder, _) = membrane()? {
        let me = agent_info()?.agent_initial_pubkey;
        if founder != me {
            let _ = send_remote_signal(
                Signal::Suggested {
                    suggestion: action_hash.clone(),
                    by: me,
                    text: suggestion.text,
                },
                vec![founder],
            );
        }
    }

    get(action_hash, GetOptions::default())?
        .ok_or_else(|| wasm_error!("Could not read the suggestion just written"))
}

/// A suggestion and what became of it, if anything.
#[derive(Serialize, Deserialize, Debug)]
pub struct SuggestionWithOutcome {
    pub suggestion: Record,
    /// None means the holder has not looked at it yet.
    pub outcome: Option<Record>,
}

#[hdk_extern]
pub fn get_suggestions(_: ()) -> ExternResult<Vec<SuggestionWithOutcome>> {
    let path = suggestion_path()?;
    let links = get_links(
        LinkQuery::try_new(path.path_entry_hash()?, LinkTypes::CircleToSuggestion)?,
        GetStrategy::Network,
    )?;

    let mut suggestions = get_many(
        links
            .into_iter()
            .filter_map(|l| l.target.into_action_hash())
            .collect(),
    )?;

    // Including anything I have offered myself, so a carer can see that what
    // she noticed was actually written down.
    and_my_own(
        &mut suggestions,
        on_my_own_chain(UnitEntryTypes::Suggestion)?,
    );

    // So the list reads in the order things were offered, and reads the same
    // way on everybody's screen, rather than in whatever order they arrived.
    oldest_first(&mut suggestions);

    // My own decisions too. Without this the holder accepts something, the
    // list still shows it as undecided, and the obvious thing to do is decide
    // it again.
    let my_decisions = on_my_own_chain(UnitEntryTypes::SuggestionOutcome)?;

    /*
     * Which decision belongs to which suggestion, worked out first, and then
     * every one of them fetched in a single call.
     *
     * This was a fetch per suggestion inside the loop below, so a circle where
     * a lot had been offered redrew slowly for exactly the person least able
     * to afford it. See `get_many`.
     */
    let mut wanted: Vec<ActionHash> = Vec::new();
    for suggestion in &suggestions {
        let mut decisions = get_links(
            LinkQuery::try_new(
                suggestion.action_address().clone(),
                LinkTypes::SuggestionToOutcome,
            )?,
            GetStrategy::Network,
        )?;

        /*
         * The newest decision, chosen the same way on every device.
         *
         * This used to take whichever decision happened to arrive first. The
         * interface hides the buttons once anything has been decided, so a
         * second decision is not reachable by pressing things — but "not
         * reachable today" is a poor reason to leave a list being read in an
         * order nothing promises. Two devices could show a suggestion as
         * accepted and set aside at the same time.
         *
         * Same rule as everywhere else here: by time, with the hash as the
         * tiebreak.
         */
        decisions.sort_by(|a, b| {
            a.timestamp
                .cmp(&b.timestamp)
                .then_with(|| a.target.cmp(&b.target))
        });

        if let Some(hash) = decisions
            .last()
            .and_then(|l| l.target.clone().into_action_hash())
        {
            wanted.push(hash);
        }
    }

    // What each decision was about, so they can be matched back up below.
    let decided: Vec<(ActionHash, Record)> = get_many(wanted)?
        .into_iter()
        .filter_map(|record| {
            let about = record
                .entry()
                .to_app_option::<SuggestionOutcome>()
                .ok()
                .flatten()?
                .suggestion;
            Some((about, record))
        })
        .collect();

    let mut out = Vec::new();
    for suggestion in suggestions {
        let hash = suggestion.action_address().clone();

        let mut outcome = decided
            .iter()
            .find(|(about, _)| about == &hash)
            .map(|(_, record)| record.clone());

        if outcome.is_none() {
            outcome = my_decisions
                .iter()
                // Backwards, so this is the newest of my own decisions and not
                // the first one I ever made. My chain comes back oldest first,
                // so searching forwards found a decision I had since changed
                // my mind about — the same fault as reading link order, in the
                // one place that does not depend on the network at all.
                .rev()
                .find(|record| {
                    record
                        .entry()
                        .to_app_option::<SuggestionOutcome>()
                        .ok()
                        .flatten()
                        .is_some_and(|o| o.suggestion == hash)
                })
                .cloned();
        }

        out.push(SuggestionWithOutcome {
            suggestion,
            outcome,
        });
    }
    Ok(out)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DecideInput {
    pub suggestion: ActionHash,
    pub accepted: bool,
}

/// Record what the holder decided.
///
/// Setting something aside is kept, not deleted. Somebody took the trouble to
/// notice a thing about a person; that should not vanish silently.
#[hdk_extern]
pub fn decide_on_suggestion(input: DecideInput) -> ExternResult<Record> {
    let outcome = SuggestionOutcome {
        suggestion: input.suggestion.clone(),
        accepted: input.accepted,
    };
    let action_hash = create_entry(EntryTypes::SuggestionOutcome(outcome))?;

    create_link(
        input.suggestion,
        action_hash.clone(),
        LinkTypes::SuggestionToOutcome,
        (),
    )?;

    get(action_hash, GetOptions::default())?
        .ok_or_else(|| wasm_error!("Could not read the decision just written"))
}

/// Correct your own suggestion before it has been decided.
///
/// Validation permits this only to whoever offered it, so nobody can put words
/// in another member's mouth.
#[hdk_extern]
pub fn update_suggestion(input: (ActionHash, Suggestion)) -> ExternResult<Record> {
    let (previous, suggestion) = input;
    let updated = update_entry(previous, &suggestion)?;

    get(updated, GetOptions::default())?
        .ok_or_else(|| wasm_error!("Could not read the corrected suggestion"))
}

/// Who holds this circle.
///
/// Read from the DNA properties rather than assumed. A circle you joined is
/// held by somebody else, and an interface that assumes otherwise will offer
/// you buttons that cannot work.
#[hdk_extern]
pub fn who_holds_this(_: ()) -> ExternResult<Option<AgentPubKey>> {
    Ok(match membrane()? {
        Membrane::Founder(key, _) => Some(key),
        _ => None,
    })
}

// ---------------------------------------------------------------------------
// Two agreements, without the post
// ---------------------------------------------------------------------------
//
// Where a circle asks two people to agree, both agreements are signatures, and
// they used to reach each other by hand: the holder sent half an invitation to
// the second person, who sent it back finished, who sent it on. Five
// copy-and-pastes of near-identical text between two devices that were already
// members of the same circle.
//
// They are members of the same circle, so they can simply tell each other. The
// holder writes down who she wants to let in; the second person sees it and
// agrees; the finished invitation appears on the holder's screen. One thing to
// send, to the person joining, which is the one place a message genuinely has
// to leave the circle.
//
// The membrane is untouched. Whoever joins still presents both signatures at
// the door, because somebody who has not joined cannot read any of this.

const PROPOSED_ANCHOR: &str = "proposed";

fn proposed_path() -> ExternResult<TypedPath> {
    anchored(PROPOSED_ANCHOR, LinkTypes::CircleToProposedMember)
}

/// Everything an invitation carries besides the signatures themselves.
///
/// Pulled out because it is now needed twice: once when an invitation is made
/// directly, and once when one is assembled from two agreements that arrived
/// separately. Two copies of this drifting apart would produce invitations
/// that differ in ways nobody would notice until somebody could not join.
fn bundle_around(
    invitee: &AgentPubKey,
    invitee_name: String,
    invitation: Invitation,
) -> ExternResult<InvitationBundle> {
    let me = agent_info()?.agent_initial_pubkey;

    let about = match get_circle_about_me(())?.first() {
        Some(original) => get_current_about_me(original.clone())?
            .record
            .and_then(|r| r.entry().as_option().cloned())
            .and_then(|e| AboutMe::try_from(e).ok())
            .map(|a| a.display_name)
            .unwrap_or_default(),
        None => String::new(),
    };

    let seconder = appointment_now()?.map(|(_, key)| key.to_string());
    let asks_two = matches!(membrane()?, Membrane::Founder(_, true));

    // My own introduction, off my own chain: what I told this circle I am
    // called. Read locally because it is mine, and empty if I never said.
    let inviter = on_my_own_chain(UnitEntryTypes::Member)?
        .last()
        .and_then(|record| record.entry().to_app_option::<Member>().ok().flatten())
        .map(|m| m.name)
        .unwrap_or_default();

    Ok(InvitationBundle {
        founder: match membrane()? {
            Membrane::Founder(founder, _) => founder.to_string(),
            _ => me.to_string(),
        },
        inviter,
        invitee: invitee.to_string(),
        invitee_name,
        seconder,
        requires_second_yes: asks_two,
        network_seed: dna_info()?.modifiers.network_seed,
        about,
        invitation,
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProposeInput {
    pub invitee: String,
    #[serde(default)]
    pub name: String,
}

/// Put somebody forward, where the person who has to agree will see it.
///
/// Only meaningful from the holder — validation refuses it from anybody else,
/// on every peer independently, so there is no permission check to forget
/// here.
#[hdk_extern]
pub fn propose_member(input: ProposeInput) -> ExternResult<Record> {
    let invitee = AgentPubKey::try_from(input.invitee.trim()).map_err(|_| {
        wasm_error!(
            "That does not look like somebody's identifier. It is a long line of \
             letters and numbers beginning uhCAk, which they can copy from their \
             own copy of Hearth. It is not their name."
        )
    })?;

    let me = agent_info()?.agent_initial_pubkey;
    let signature = sign(me.clone(), invitee.clone())?;
    let name = input.name.trim().to_string();

    let action_hash = create_entry(EntryTypes::ProposedMember(ProposedMember {
        invitee: invitee.clone(),
        name: name.clone(),
        signature,
    }))?;

    let path = proposed_path()?;
    path.ensure()?;
    create_link(
        path.path_entry_hash()?,
        action_hash.clone(),
        LinkTypes::CircleToProposedMember,
        (),
    )?;

    // Tell the person who has to agree, so they do not have to be watching.
    // Fire and forget, like every other signal here: the proposal is on the
    // chain either way, and they will see it whenever they next look.
    if let Some((_, seconder)) = appointment_now()? {
        if seconder != me {
            let _ = send_remote_signal(
                Signal::Proposed {
                    proposed: action_hash.clone(),
                    by: me,
                    name,
                },
                vec![seconder],
            );
        }
    }

    get(action_hash, GetOptions::default())?
        .ok_or_else(|| wasm_error!("Could not read the proposal just written"))
}

/// Give the second agreement to somebody the holder has put forward.
///
/// Called by whoever the circle names, on their own machine, having read who
/// it is. Validation refuses it from anybody else — that check is the whole
/// safeguard, and it is made by every peer rather than here.
#[hdk_extern]
pub fn endorse(proposed: ActionHash) -> ExternResult<Record> {
    let record = get(proposed.clone(), GetOptions::default())?
        .ok_or_else(|| wasm_error!("That proposal could not be found"))?;

    let entry = record
        .entry()
        .to_app_option::<ProposedMember>()
        .map_err(|e| wasm_error!(format!("{e:?}")))?
        .ok_or_else(|| wasm_error!("That is not somebody put forward to join"))?;

    let me = agent_info()?.agent_initial_pubkey;
    let signature = sign(me.clone(), entry.invitee.clone())?;

    let (appointment, _) = appointment_now()?
        .ok_or_else(|| wasm_error!("This circle has not asked anybody to agree to who joins"))?;

    let action_hash = create_entry(EntryTypes::Endorsement(Endorsement {
        proposed: proposed.clone(),
        appointment,
        signature,
    }))?;

    create_link(
        proposed.clone(),
        action_hash.clone(),
        LinkTypes::ProposedMemberToEndorsement,
        (),
    )?;

    // Tell the holder, so the finished invitation appears in front of her
    // rather than being waited for.
    if let Membrane::Founder(founder, _) = membrane()? {
        if founder != me {
            let _ = send_remote_signal(Signal::Endorsed { proposed, by: me }, vec![founder]);
        }
    }

    get(action_hash, GetOptions::default())?
        .ok_or_else(|| wasm_error!("Could not read the agreement just written"))
}

/// Somebody put forward, and how far they have got.
#[derive(Serialize, Deserialize, Debug)]
pub struct PendingMember {
    pub proposed: ActionHash,
    /// The key that would be admitted, as text.
    pub invitee: String,
    /// What the holder called them. A claim, never checked.
    pub name: String,
    /// Whether the second person has agreed yet.
    pub agreed: bool,
    /// The finished invitation, once both agreements exist.
    ///
    /// `None` until then, and that is the whole state anybody needs: nothing
    /// to assemble by hand, and nothing to send too early.
    pub invitation: Option<InvitationBundle>,
}

/// Everybody currently put forward, oldest first.
#[hdk_extern]
pub fn get_pending_members(_: ()) -> ExternResult<Vec<PendingMember>> {
    let path = proposed_path()?;
    let links = get_links(
        LinkQuery::try_new(path.path_entry_hash()?, LinkTypes::CircleToProposedMember)?,
        GetStrategy::Network,
    )?;

    let mut proposals = get_many(
        links
            .into_iter()
            .filter_map(|l| l.target.into_action_hash())
            .collect(),
    )?;

    // Mine too, so the holder sees what she just put forward without waiting
    // for the network to hear about it.
    and_my_own(
        &mut proposals,
        on_my_own_chain(UnitEntryTypes::ProposedMember)?,
    );
    oldest_first(&mut proposals);

    // My own agreements, for the same reason on the other side.
    let mine = on_my_own_chain(UnitEntryTypes::Endorsement)?;

    let mut out = Vec::new();
    for record in proposals {
        let hash = record.action_address().clone();
        let Some(proposed) = record
            .entry()
            .to_app_option::<ProposedMember>()
            .ok()
            .flatten()
        else {
            continue;
        };

        let mut endorsements = get_links(
            LinkQuery::try_new(hash.clone(), LinkTypes::ProposedMemberToEndorsement)?,
            GetStrategy::Network,
        )?;
        endorsements.sort_by(|a, b| {
            a.timestamp
                .cmp(&b.timestamp)
                .then_with(|| a.target.cmp(&b.target))
        });

        let mut endorsement = match endorsements
            .last()
            .and_then(|l| l.target.clone().into_action_hash())
        {
            Some(h) => get(h, GetOptions::default())?,
            None => None,
        };

        if endorsement.is_none() {
            endorsement = mine
                .iter()
                .rev()
                .find(|r| {
                    r.entry()
                        .to_app_option::<Endorsement>()
                        .ok()
                        .flatten()
                        .is_some_and(|e| e.proposed == hash)
                })
                .cloned();
        }

        // Both halves of the agreement: the signature, and the appointment it
        // was given under. The second matters because appointments change, and
        // the door checks against the one that was in force at the time rather
        // than whoever is appointed by the time somebody joins.
        let given =
            endorsement.and_then(|r| r.entry().to_app_option::<Endorsement>().ok().flatten());
        let under = given.as_ref().map(|e| e.appointment.clone());
        let seconded = given.map(|e| e.signature);

        // Assembled only when both halves are here. An invitation with one
        // agreement on it is not a weaker invitation; it is not one yet, and
        // handing it over would send somebody to a door that will not open.
        let invitation = match &seconded {
            Some(_) => Some(bundle_around(
                &proposed.invitee,
                proposed.name.clone(),
                Invitation {
                    signature: proposed.signature.clone(),
                    seconded: seconded.clone(),
                    // The one the agreement was actually given under, not
                    // whichever is in force now. They can differ, and the
                    // door checks against the first.
                    appointment: under.clone(),
                },
            )?),
            None => None,
        };

        out.push(PendingMember {
            proposed: hash,
            invitee: proposed.invitee.to_string(),
            name: proposed.name,
            agreed: seconded.is_some(),
            invitation,
        });
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// The waiting room
// ---------------------------------------------------------------------------
//
// Joining used to begin with "send me the long line of characters from your
// app". That is the step where this stopped being possible for somebody
// elderly, or somebody being helped: an invitation is signed over a key, so
// the key had to be collected first, one person at a time, by hand.
//
// A waiting room turns it round. The holder shares one address that never
// changes and can go to anybody — a family group, a phone call, a note on the
// fridge. Whoever has it can knock: say who they are and ask. They bring their
// own key with them by arriving.
//
// It is a separate network on purpose. A circle is closed, so somebody outside
// cannot write to it — that is the membrane doing its job, and no amount of
// interface removes it. So the room is somewhere they *can* write, holding
// nothing but questions and the answers to them.

const KNOCK_ANCHOR: &str = "knocks";

fn knock_path() -> ExternResult<TypedPath> {
    anchored(KNOCK_ANCHOR, LinkTypes::WaitingRoomToKnock)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WaitingRoomInput {
    /// Whoever holds the circle this room serves, as base64 text.
    pub holder: String,
    /// Makes this room distinct from any other. Part of its address.
    pub network_seed: String,
    /// What this device calls it. Not part of the address.
    pub name: String,
}

fn waiting_room_modifiers(
    holder: &AgentPubKey,
    network_seed: String,
) -> ExternResult<DnaModifiersOpt<YamlProperties>> {
    let properties = CircleProperties {
        founder: None,
        lobby: false,
        seconder: None,
        // A waiting room decides nothing. It carries a question in and an
        // answer out; what happens between is settled in the circle.
        requires_second_yes: false,
        waiting_for: Some(holder.to_string()),
    };

    let yaml = yaml_serde::to_value(&properties).map_err(|e| {
        wasm_error!(format!(
            "Could not express the waiting room's properties as YAML: {e}"
        ))
    })?;

    Ok(DnaModifiersOpt::none()
        .with_network_seed(network_seed)
        .with_properties(YamlProperties::new(yaml)))
}

/// Open a waiting room, or step into somebody else's.
///
/// The same call for both, because they are the same act: a waiting room is
/// open, so there is nothing to present and nobody to ask. The holder makes
/// one for her circle; whoever she gives the address to arrives in the same
/// room by computing the same one.
#[hdk_extern]
pub fn enter_waiting_room(input: WaitingRoomInput) -> ExternResult<ClonedCell> {
    let holder = AgentPubKey::try_from(input.holder.trim())
        .map_err(|_| wasm_error!("That waiting room names nobody this app can read"))?;

    create_clone_cell(CreateCloneCellInput {
        cell_id: this_cell()?,
        modifiers: waiting_room_modifiers(&holder, input.network_seed)?,
        membrane_proof: None,
        name: Some(input.name),
    })
}

/// Ask to be let in.
///
/// No permission check, deliberately. A room where you must already be known
/// in order to ask is the closed door this exists to replace — and knocking
/// admits nobody. Everything said here is a claim, and the people deciding are
/// told so.
/// Box some words so that one named person can read them.
///
/// Sender and recipient may be the same key, which is how somebody seals a
/// copy to themselves.
fn seal_for(
    words: &WhoIsKnocking,
    sender: AgentPubKey,
    recipient: AgentPubKey,
) -> ExternResult<XSalsa20Poly1305EncryptedData> {
    let bytes = ExternIO::encode(words)
        .map_err(|e| wasm_error!(format!("{e:?}")))?
        .into_vec();
    ed_25519_x_salsa20_poly1305_encrypt(sender, recipient, bytes.into())
}

/// Open a sealed knock, or give up quietly.
///
/// Quietly on purpose. Everybody in a room can read every knock in it, and
/// almost none of them are theirs to open. Failing to open one is the
/// ordinary case, not an error.
fn unseal(
    sealed: &XSalsa20Poly1305EncryptedData,
    recipient: AgentPubKey,
    sender: AgentPubKey,
) -> Option<WhoIsKnocking> {
    let opened = ed_25519_x_salsa20_poly1305_decrypt(recipient, sender, sealed.clone()).ok()?;
    ExternIO::from(opened.as_ref().to_vec()).decode().ok()
}

#[hdk_extern]
pub fn knock(words: WhoIsKnocking) -> ExternResult<Record> {
    /*
     * Asked for here rather than checked by every peer.
     *
     * It used to be a validation rule. It cannot be one now the words are
     * sealed — a peer that cannot read a thing cannot have an opinion about
     * it. Nothing was lost: an empty name was never dangerous, only useless.
     */
    if words.name.trim().is_empty() {
        return Err(wasm_error!(
            "Say what you are called, so they know who is asking"
        ));
    }

    let me = agent_info()?.agent_initial_pubkey;
    let Membrane::WaitingRoom(holder) = membrane()? else {
        return Err(wasm_error!(
            "Knocking only means something in a circle's waiting room"
        ));
    };

    let knock = Knock {
        for_the_holder: seal_for(&words, me.clone(), holder.clone())?,
        for_me: seal_for(&words, me.clone(), me.clone())?,
    };

    let action_hash = create_entry(EntryTypes::Knock(knock.clone()))?;

    let path = knock_path()?;
    path.ensure()?;
    create_link(
        path.path_entry_hash()?,
        action_hash.clone(),
        LinkTypes::WaitingRoomToKnock,
        (),
    )?;

    /*
     * Tell the holder somebody is at the door. Fire and forget, as ever: the
     * knock is written either way and she will see it when she next looks.
     *
     * The words travel in the clear here, and that is not the same as
     * writing them in the open. A remote signal goes to one named agent over
     * the encrypted transport; the entry sits in a room anybody with the
     * address can read. Only the second one needed sealing.
     */
    if holder != me {
        let _ = send_remote_signal(
            Signal::Knocked {
                by: me,
                name: words.name,
                relationship: words.relationship,
            },
            vec![holder],
        );
    }

    get(action_hash, GetOptions::default())?
        .ok_or_else(|| wasm_error!("Could not read the knock just written"))
}

/// Somebody at the door, and what has become of them.
#[derive(Serialize, Deserialize, Debug)]
pub struct Knocking {
    pub knock: ActionHash,
    /// Whoever knocked. This is the key an invitation would be made for, and
    /// it is theirs by the fact of their having written the knock — which is
    /// the whole reason nobody had to collect it from them.
    pub who: String,
    /// What they call themselves. A claim.
    ///
    /// Empty where this device cannot open the knock, which is the ordinary
    /// case for everybody but the holder and the person who wrote it.
    pub name: String,
    /// How they say they are connected. A claim.
    pub relationship: String,
    /// True once they have been answered.
    pub answered: bool,
}

/// Everybody at the door, oldest first.
#[hdk_extern]
pub fn get_knocks(_: ()) -> ExternResult<Vec<Knocking>> {
    let path = knock_path()?;
    let links = get_links(
        LinkQuery::try_new(path.path_entry_hash()?, LinkTypes::WaitingRoomToKnock)?,
        GetStrategy::Network,
    )?;

    let mut knocks = get_many(
        links
            .into_iter()
            .filter_map(|l| l.target.into_action_hash())
            .collect(),
    )?;

    and_my_own(&mut knocks, on_my_own_chain(UnitEntryTypes::Knock)?);
    oldest_first(&mut knocks);

    // Read once, before the loop, because neither changes inside it.
    let my_answers = on_my_own_chain(UnitEntryTypes::Admission)?;
    let me = agent_info()?.agent_initial_pubkey;

    let mut out = Vec::new();
    for record in knocks {
        let hash = record.action_address().clone();
        let Some(knock) = record.entry().to_app_option::<Knock>().ok().flatten() else {
            continue;
        };

        let answers = get_links(
            LinkQuery::try_new(hash.clone(), LinkTypes::KnockToAdmission)?,
            GetStrategy::Network,
        )?;

        /*
         * And my own answers, which the network has not heard about yet.
         *
         * The rule at the top of this file, broken here and found by walking
         * it: what I wrote myself is never a question for the network. She
         * pressed "Let them in", the answer was written, the person was
         * admitted and arrived in the circle — and her own screen went on
         * asking the network whether she had done it, was told no, and left
         * them sitting at the door with the button still under them.
         *
         * The worst kind of wrong, too: pressing it again would have made a
         * second invitation for somebody already inside.
         */
        let answered = !answers.is_empty()
            || my_answers.iter().any(|record| {
                record
                    .entry()
                    .to_app_option::<Admission>()
                    .ok()
                    .flatten()
                    .is_some_and(|a| a.knock == hash)
            });

        /*
         * Opened where this device is one of the two ends, and otherwise
         * left shut.
         *
         * The holder opens the copy sealed to her. The person who knocked
         * opens the copy they sealed to themselves, which is how their own
         * app can tell them back what they said after a restart. To
         * everybody else in the room these are two blobs, which is the
         * entire point of them.
         */
        let author = record.action().author().clone();
        let words = if author == me {
            unseal(&knock.for_me, me.clone(), me.clone())
        } else {
            unseal(&knock.for_the_holder, me.clone(), author.clone())
        };

        out.push(Knocking {
            knock: hash,
            who: author.to_string(),
            name: words.as_ref().map(|w| w.name.clone()).unwrap_or_default(),
            relationship: words.map(|w| w.relationship).unwrap_or_default(),
            answered,
        });
    }

    Ok(out)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AdmitInput {
    pub knock: ActionHash,
    /// The finished invitation, as the one line of text the app passes about.
    pub invitation: String,
}

/// Answer a knock by leaving the invitation where they will find it.
///
/// Safe in the open: an invitation is signed over one person's own key, so it
/// admits nobody else and is a useless blob to anyone who picks it up. That is
/// what lets the answer be left in a room anybody may enter rather than
/// carried by hand to the one person it is for.
#[hdk_extern]
pub fn admit(input: AdmitInput) -> ExternResult<Record> {
    let action_hash = create_entry(EntryTypes::Admission(Admission {
        knock: input.knock.clone(),
        invitation: input.invitation,
    }))?;

    create_link(
        input.knock.clone(),
        action_hash.clone(),
        LinkTypes::KnockToAdmission,
        (),
    )?;

    // Tell them the door is open, so they are not left refreshing.
    if let Some(record) = get(input.knock, GetOptions::default())? {
        let waiting = record.action().author().clone();
        let me = agent_info()?.agent_initial_pubkey;
        if waiting != me {
            let _ = send_remote_signal(Signal::Admitted { by: me }, vec![waiting]);
        }
    }

    get(action_hash, GetOptions::default())?
        .ok_or_else(|| wasm_error!("Could not read the answer just written"))
}

/// The invitation waiting for me here, if there is one.
///
/// Read from my own knocks outwards rather than from the whole room, so this
/// answers "have I been let in" and never "who else has".
#[hdk_extern]
pub fn my_admission(_: ()) -> ExternResult<Option<String>> {
    let mine: Vec<ActionHash> = on_my_own_chain(UnitEntryTypes::Knock)?
        .into_iter()
        .map(|r| r.action_address().clone())
        .collect();

    for knock in mine.into_iter().rev() {
        let answers = get_links(
            LinkQuery::try_new(knock, LinkTypes::KnockToAdmission)?,
            GetStrategy::Network,
        )?;

        for link in answers {
            let Some(hash) = link.target.into_action_hash() else {
                continue;
            };
            let Some(record) = get(hash, GetOptions::default())? else {
                continue;
            };
            if let Some(admission) = record.entry().to_app_option::<Admission>().ok().flatten() {
                return Ok(Some(admission.invitation));
            }
        }
    }

    Ok(None)
}

// ---------------------------------------------------------------------------
// Who has been asked to agree to who joins
// ---------------------------------------------------------------------------
//
// The person used to be written into the circle's identity, which made them
// permanent: if they died, lost the device their keys were on, or simply had
// to be replaced, the only way out was a new circle with everybody
// re-invited. For a record about somebody in declining health, one of the two
// people becoming unable to answer is not an edge case. It is the expected
// course of events.
//
// So the identity carries the rule and the circle carries the person.

const APPOINTMENT_ANCHOR: &str = "appointments";

fn appointment_path() -> ExternResult<TypedPath> {
    anchored(APPOINTMENT_ANCHOR, LinkTypes::CircleToAppointment)
}

/// Ask somebody to agree to who joins, from now on.
///
/// Writing another one later replaces it. Nothing is erased: who was trusted
/// with this, and when, stays in the circle where everybody can see it, which
/// is what the safeguard now rests on.
///
/// It is an ask, not an instruction. They are told, and they answer -- see
/// answer_appointment. Nothing here can compel anybody to agree to an
/// arrival, and nothing tries to; what this makes possible is that saying no
/// reaches the holder instead of looking exactly like not having got round
/// to it yet.
#[hdk_extern]
pub fn appoint(agrees: String) -> ExternResult<Record> {
    let agrees = AgentPubKey::try_from(agrees.trim())
        .map_err(|_| wasm_error!("That is not an identifier this circle can read"))?;

    let action_hash = create_entry(EntryTypes::Appointment(Appointment {
        agrees: agrees.clone(),
    }))?;

    let path = appointment_path()?;
    path.ensure()?;
    create_link(
        path.path_entry_hash()?,
        action_hash.clone(),
        LinkTypes::CircleToAppointment,
        (),
    )?;

    // Ask them, rather than leaving them to find out.
    let me = agent_info()?.agent_initial_pubkey;
    if agrees != me {
        let _ = send_remote_signal(
            Signal::Appointed {
                appointment: action_hash.clone(),
                by: me,
            },
            vec![agrees],
        );
    }

    get(action_hash, GetOptions::default())?
        .ok_or_else(|| wasm_error!("Could not read the appointment just written"))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnswerInput {
    pub appointment: ActionHash,
    pub willing: bool,
}

/// Say whether you are willing to be the one who agrees to who joins.
///
/// Answering again changes your mind, and the newest answer is the one that
/// counts. Nothing is erased -- the holder may have acted on what you said
/// before.
#[hdk_extern]
pub fn answer_appointment(input: AnswerInput) -> ExternResult<Record> {
    let action_hash = create_entry(EntryTypes::Consent(Consent {
        appointment: input.appointment.clone(),
        willing: input.willing,
    }))?;

    create_link(
        input.appointment,
        action_hash.clone(),
        LinkTypes::AppointmentToConsent,
        (),
    )?;

    // Tell the holder. A no she does not hear is the same to her as no answer
    // at all, and this exists precisely to tell those two apart.
    let me = agent_info()?.agent_initial_pubkey;
    if let Membrane::Founder(founder, _) = membrane()? {
        if founder != me {
            let _ = send_remote_signal(
                Signal::Answered {
                    by: me,
                    willing: input.willing,
                },
                vec![founder],
            );
        }
    }

    get(action_hash, GetOptions::default())?
        .ok_or_else(|| wasm_error!("Could not read the answer just written"))
}

/// Who has been asked to agree to who joins, and what they said.
///
/// One read for the whole state, because every screen that cares about any
/// part of it cares about all of it: a name to show, whether to put the
/// question in front of the person being asked, and whether to tell the
/// holder she needs to ask somebody else.
#[derive(Serialize, Deserialize, Debug)]
pub struct WhoAgrees {
    pub appointment: ActionHash,
    /// The person asked, as text.
    pub agrees: String,
    /// What they said, and None while they have not answered.
    ///
    /// Three states, not two. "Not answered yet" and "said no" look the same
    /// from outside and mean entirely different things to the holder.
    pub willing: Option<bool>,
}

#[hdk_extern]
pub fn who_agrees_here(_: ()) -> ExternResult<Option<WhoAgrees>> {
    let Some((appointment, agrees)) = appointment_now()? else {
        return Ok(None);
    };
    let willing = answer_to(&appointment)?;
    Ok(Some(WhoAgrees {
        appointment,
        agrees: agrees.to_string(),
        willing,
    }))
}

/// The newest answer to one appointment, if it has been answered.
fn answer_to(appointment: &ActionHash) -> ExternResult<Option<bool>> {
    let links = get_links(
        LinkQuery::try_new(appointment.clone(), LinkTypes::AppointmentToConsent)?,
        GetStrategy::Network,
    )?;

    let mut found = get_many(
        links
            .into_iter()
            .filter_map(|l| l.target.into_action_hash())
            .collect(),
    )?;

    // Mine too, so somebody who has just answered sees their own answer
    // without waiting for the network to hear about it. The same rule as
    // everywhere else here, and the one whose absence once had a holder
    // press "let them in" twice.
    and_my_own(&mut found, on_my_own_chain(UnitEntryTypes::Consent)?);
    oldest_first(&mut found);

    Ok(found.into_iter().rev().find_map(|record| {
        let consent = record.entry().to_app_option::<Consent>().ok().flatten()?;
        (&consent.appointment == appointment).then_some(consent.willing)
    }))
}

/// The appointment in force: the newest one the holder has written.
///
/// A coordinator read, where "newest" is a perfectly good question. Validation
/// could never ask it — the answer changes — which is why everything that has
/// to be checked names the appointment it relies on instead.
fn appointment_now() -> ExternResult<Option<(ActionHash, AgentPubKey)>> {
    let path = appointment_path()?;
    let links = get_links(
        LinkQuery::try_new(path.path_entry_hash()?, LinkTypes::CircleToAppointment)?,
        GetStrategy::Network,
    )?;

    let mut found = get_many(
        links
            .into_iter()
            .filter_map(|l| l.target.into_action_hash())
            .collect(),
    )?;

    // Mine too: the holder should see who she just appointed without waiting
    // for the network to hear about it.
    and_my_own(&mut found, on_my_own_chain(UnitEntryTypes::Appointment)?);
    oldest_first(&mut found);

    Ok(found.into_iter().rev().find_map(|record| {
        let appointment = record
            .entry()
            .to_app_option::<Appointment>()
            .ok()
            .flatten()?;
        Some((record.action_address().clone(), appointment.agrees))
    }))
}
