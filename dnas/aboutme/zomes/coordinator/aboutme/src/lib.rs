//! Coordinator zome for About Me.
//!
//! Everything here runs locally, on the machine of whoever calls it.
//! There is no server. If every device in a circle is switched off, the
//! circle is simply not reachable — see the open question in README.md,
//! which is the thing to resolve before this becomes a product.

use aboutme_integrity::*;
use hdk::prelude::*;

const CIRCLE_ANCHOR: &str = "circle";

fn circle_path() -> ExternResult<TypedPath> {
    Path::from(CIRCLE_ANCHOR).typed(LinkTypes::CircleToAboutMe)
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
    pub founder: AgentPubKey,
    pub network_seed: String,
    /// Whose circle this is, so the recipient knows what they are accepting
    /// before they accept it. A label, not a claim.
    pub about: String,
    pub invitation: Invitation,
}

#[hdk_extern]
pub fn invite(invitee: AgentPubKey) -> ExternResult<InvitationBundle> {
    let me = agent_info()?.agent_initial_pubkey;
    let signature = sign(me.clone(), invitee)?;

    let about = match get_circle_about_me(())?.first() {
        Some(original) => get_current_about_me(original.clone())?
            .record
            .and_then(|r| r.entry().as_option().cloned())
            .and_then(|e| AboutMe::try_from(e).ok())
            .map(|a| a.display_name)
            .unwrap_or_default(),
        None => String::new(),
    };

    Ok(InvitationBundle {
        founder: me,
        network_seed: dna_info()?.modifiers.network_seed,
        about,
        invitation: Invitation { signature },
    })
}

/// Who you are in this circle, in your own words.
///
/// Nothing here is verified. "Her son" is a claim, exactly like a
/// professional's role on an acknowledgement, and the interface must not
/// dress it up as anything more.
#[hdk_extern]
pub fn introduce_myself(member: Member) -> ExternResult<Record> {
    let action_hash = create_entry(EntryTypes::Member(member))?;

    let path = Path::from("members").typed(LinkTypes::CircleToMember)?;
    path.ensure()?;
    create_link(
        path.path_entry_hash()?,
        action_hash.clone(),
        LinkTypes::CircleToMember,
        (),
    )?;

    get(action_hash, GetOptions::default())?
        .ok_or_else(|| wasm_error!("Could not read the introduction just written"))
}

#[hdk_extern]
pub fn get_members(_: ()) -> ExternResult<Vec<Record>> {
    let path = Path::from("members").typed(LinkTypes::CircleToMember)?;
    let links = get_links(
        LinkQuery::try_new(path.path_entry_hash()?, LinkTypes::CircleToMember)?,
        GetStrategy::Network,
    )?;

    let mut out = Vec::new();
    for link in links {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(record) = get(hash, GetOptions::default())? {
                out.push(record);
            }
        }
    }
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
    Ok(links
        .into_iter()
        .filter_map(|l| l.target.into_action_hash())
        .collect())
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
    let updates: Vec<(Timestamp, ActionHash)> = links
        .into_iter()
        .filter_map(|l| l.target.into_action_hash().map(|hash| (l.timestamp, hash)))
        .collect();

    let mut versions = vec![original_action_hash];
    versions.extend(order_versions(updates));
    Ok(versions)
}

/// The version to show, and whether showing one is misleading.
#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentAboutMe {
    pub record: Option<Record>,
    /// How many versions exist in total. More than one means edits were made
    /// while devices were apart, and the interface should say so rather than
    /// quietly picking a winner.
    pub version_count: usize,
}

#[hdk_extern]
pub fn get_current_about_me(original_action_hash: ActionHash) -> ExternResult<CurrentAboutMe> {
    let versions = get_about_me_versions(original_action_hash)?;
    let version_count = versions.len();
    let newest = versions
        .last()
        .cloned()
        .ok_or_else(|| wasm_error!("An About Me always has at least its original version"))?;

    Ok(CurrentAboutMe {
        record: get(newest, GetOptions::default())?,
        version_count,
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
    if let Membrane::Founder(founder) = membrane()? {
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
    let links = get_links(
        LinkQuery::try_new(about_me, LinkTypes::AboutMeToAcknowledgement)?,
        GetStrategy::Network,
    )?;

    let mut records = Vec::new();
    for link in links {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(record) = get(hash, GetOptions::default())? {
                records.push(record);
            }
        }
    }
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

fn circle_modifiers(
    founder: &AgentPubKey,
    network_seed: String,
) -> ExternResult<DnaModifiersOpt<YamlProperties>> {
    let properties = CircleProperties {
        founder: Some(founder.to_string()),
        lobby: false,
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
    /// The person this circle is about. Normally the caller, but a proxy may
    /// hold the keys for someone who cannot.
    pub founder: AgentPubKey,
    /// A human name for this circle, shown in the app. Not part of the DNA
    /// hash, so two people may safely use the same word.
    pub name: String,
    /// Makes this circle distinct from any other for the same person.
    pub network_seed: String,
}

/// Bring a new circle into being.
#[hdk_extern]
pub fn create_circle(input: CreateCircleInput) -> ExternResult<ClonedCell> {
    create_clone_cell(CreateCloneCellInput {
        cell_id: this_cell()?,
        modifiers: circle_modifiers(&input.founder, input.network_seed)?,
        membrane_proof: None,
        name: Some(input.name),
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JoinCircleInput {
    /// The person whose circle this is. Must match what the inviter used, or
    /// the DNA hash differs and you land in a different network entirely.
    pub founder: AgentPubKey,
    pub name: String,
    pub network_seed: String,
    /// From the founder's `invite`, signed over the joiner's own key.
    pub invitation: Invitation,
}

/// Join a circle you have been invited to.
///
/// Note what is *not* here: no request, no approval step, nobody to ask. The
/// invitation is the whole of it, and every existing member checks it
/// independently.
#[hdk_extern]
pub fn join_circle(input: JoinCircleInput) -> ExternResult<ClonedCell> {
    let proof = SerializedBytes::try_from(input.invitation)
        .map(MembraneProof::new)
        .map_err(|e| wasm_error!(format!("Could not read that invitation: {e:?}")))?;

    create_clone_cell(CreateCloneCellInput {
        cell_id: this_cell()?,
        modifiers: circle_modifiers(&input.founder, input.network_seed)?,
        membrane_proof: Some(proof),
        name: Some(input.name),
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
    /// Somebody offered something for the record.
    Suggested {
        suggestion: ActionHash,
        by: AgentPubKey,
        text: String,
    },
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
#[hdk_extern]
pub fn recv_remote_signal(signal: Signal) -> ExternResult<()> {
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
    Path::from(SUGGESTION_ANCHOR).typed(LinkTypes::CircleToSuggestion)
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
    if let Membrane::Founder(founder) = membrane()? {
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

    let mut out = Vec::new();
    for link in links {
        let Some(hash) = link.target.into_action_hash() else {
            continue;
        };
        let Some(suggestion) = get(hash.clone(), GetOptions::default())? else {
            continue;
        };

        let decisions = get_links(
            LinkQuery::try_new(hash, LinkTypes::SuggestionToOutcome)?,
            GetStrategy::Network,
        )?;
        let outcome = match decisions
            .first()
            .and_then(|l| l.target.clone().into_action_hash())
        {
            Some(h) => get(h, GetOptions::default())?,
            None => None,
        };

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
        Membrane::Founder(key) => Some(key),
        _ => None,
    })
}
