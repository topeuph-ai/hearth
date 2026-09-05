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
    let mut links = get_links(
        LinkQuery::try_new(original_action_hash.clone(), LinkTypes::AboutMeUpdates)?,
        GetStrategy::Network,
    )?;

    // Sort by timestamp, then by target hash. The hash tiebreak is not
    // decoration: two links can carry the same timestamp, and `get_links` makes
    // no promise that peers see links in the same order. Without a tiebreaker,
    // two devices could disagree about which version is current.
    links.sort_by(|a, b| {
        a.timestamp
            .cmp(&b.timestamp)
            .then_with(|| a.target.cmp(&b.target))
    });

    let mut versions = vec![original_action_hash];
    versions.extend(
        links
            .into_iter()
            .filter_map(|l| l.target.into_action_hash()),
    );
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
    let ack = Acknowledgement {
        about_me: input.about_me.clone(),
        role: input.role,
    };
    let action_hash = create_entry(EntryTypes::Acknowledgement(ack))?;

    create_link(
        input.about_me,
        action_hash.clone(),
        LinkTypes::AboutMeToAcknowledgement,
        (),
    )?;

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
