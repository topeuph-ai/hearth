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

/// Follow the update chain and return the current version.
#[hdk_extern]
pub fn get_latest_about_me(original_action_hash: ActionHash) -> ExternResult<Option<Record>> {
    let links = get_links(
        LinkQuery::try_new(original_action_hash.clone(), LinkTypes::AboutMeUpdates)?,
        GetStrategy::Network,
    )?;

    // Latest by link timestamp. Note there is no global clock and no total
    // order here: two people editing while apart can both be valid. This
    // picks one deterministically rather than pretending there was a winner.
    let latest = links
        .into_iter()
        .max_by(|a, b| a.timestamp.cmp(&b.timestamp))
        .and_then(|l| l.target.into_action_hash())
        .unwrap_or(original_action_hash);

    get(latest, GetOptions::default())
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
