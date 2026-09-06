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
#[hdk_extern]
pub fn invite(invitee: AgentPubKey) -> ExternResult<Invitation> {
    let me = agent_info()?.agent_initial_pubkey;
    let signature = sign(me, invitee)?;
    Ok(Invitation { signature })
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
