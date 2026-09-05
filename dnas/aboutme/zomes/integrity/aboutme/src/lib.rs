//! About Me — the person's own contribution to their record.
//!
//! Deliberately scoped to the PRSB "About Me" standard, which is explicitly
//! NOT a clinical record: no medications, no diagnoses, no care plan. That
//! boundary is what keeps clinical safety certification and clinician
//! liability out of scope. Do not add clinical fields here without
//! understanding what they drag in with them.

use hdi::prelude::*;

/// Written by the person, or by whoever acts for them.
///
/// Field names track the PRSB About Me headings. They are a working subset
/// and should be checked against the published standard before v1.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct AboutMe {
    pub display_name: String,
    /// What matters to me
    pub what_matters_to_me: String,
    /// How I communicate, and how to communicate with me
    pub how_to_communicate_with_me: String,
    /// How to support me / what helps me feel at ease
    pub how_to_support_me: String,
    /// People who matter to me
    pub people_who_matter: String,
}

/// A professional's "I have read this."
///
/// This is the entire professional workflow: one tap. It is cheap for them
/// and it is the thing families currently have no way of knowing.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Acknowledgement {
    /// The exact version of About Me that was read. Not the latest —
    /// the one actually in front of them at the time.
    pub about_me: ActionHash,
    /// Free text, e.g. "district nurse". Not a verified credential.
    pub role: String,
}

#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    AboutMe(AboutMe),
    Acknowledgement(Acknowledgement),
}

#[hdk_link_types]
pub enum LinkTypes {
    /// Anchor -> AboutMe, so somebody joining the circle can find it.
    CircleToAboutMe,
    /// Original AboutMe -> its updates.
    AboutMeUpdates,
    /// AboutMe version -> acknowledgements of that version.
    AboutMeToAcknowledgement,
}

fn invalid(reason: &str) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(reason.to_string()))
}

// ---------------------------------------------------------------------------
// The membrane: who is allowed into this circle
// ---------------------------------------------------------------------------

/// Baked into the DNA at clone time, so it forms part of the DNA hash.
///
/// This is why one circle per person works: a different founder produces a
/// different DNA hash, which produces a genuinely separate network. Circles
/// cannot see each other, and that is a property of the maths rather than of
/// anyone's access control list.
#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct CircleProperties {
    /// The person whose circle this is, or whoever acts for them, as a base64
    /// agent key (`uhCAk...`).
    ///
    /// A string rather than raw key bytes for two reasons. DNA properties must
    /// be YAML-representable and a byte array is not. And in a real deployment
    /// a person or an interface writes this value, and nobody hand-writes a
    /// byte array.
    pub founder: String,
}

/// What an invited person presents when they join.
///
/// It is simply the founder's signature over the invitee's public key. Nobody
/// else can produce one, and every peer can check it without asking anyone.
#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct Invitation {
    pub signature: Signature,
}

/// The founder of this circle, if one is set.
///
/// **Development sharp edge.** A DNA with no founder property is an open
/// circle that anyone may join. That exists so the base cell is installable
/// during development, where no founder key is known ahead of time. Every real
/// circle is a clone that sets this property. This must be made to fail closed
/// before the software is put in front of anyone.
fn founder() -> ExternResult<Option<AgentPubKey>> {
    let properties = dna_info()?.modifiers.properties;
    let Ok(p) = CircleProperties::try_from(properties) else {
        return Ok(None);
    };
    // A property that is present but unreadable is an error, not an open
    // circle. Only its complete absence means development mode.
    AgentPubKey::try_from(p.founder.as_str())
        .map(Some)
        .map_err(|_| wasm_error!("This circle's founder property is not a valid agent key"))
}

fn check_membrane(
    agent: &AgentPubKey,
    membrane_proof: &Option<MembraneProof>,
) -> ExternResult<ValidateCallbackResult> {
    let Some(founder) = founder()? else {
        // Development only. See the note on `founder`.
        return Ok(ValidateCallbackResult::Valid);
    };

    // The founder needs no invitation to their own circle.
    if agent == &founder {
        return Ok(ValidateCallbackResult::Valid);
    }

    let Some(proof) = membrane_proof else {
        return invalid(
            "Joining this circle needs an invitation from the person whose circle it is",
        );
    };

    let invitation = match Invitation::try_from((**proof).clone()) {
        Ok(i) => i,
        Err(_) => return invalid("Invitation is not in a form this circle understands"),
    };

    // Signed over the invitee's own key, so an invitation cannot be passed on
    // to somebody else.
    if verify_signature(founder, invitation.signature, agent.clone())? {
        Ok(ValidateCallbackResult::Valid)
    } else {
        invalid("Invitation was not issued by the person whose circle this is")
    }
}

/// Checked locally before joining, so a bad invitation fails immediately with a
/// readable reason rather than being silently rejected by the network later.
#[hdk_extern]
pub fn genesis_self_check(data: GenesisSelfCheckData) -> ExternResult<ValidateCallbackResult> {
    check_membrane(&data.agent_key, &data.membrane_proof)
}

/// Is this agent the person whose circle this is?
///
/// In a development circle with no founder property, everyone is. See the note
/// on `founder`.
fn is_the_person(agent: &AgentPubKey) -> ExternResult<bool> {
    Ok(match founder()? {
        Some(f) => &f == agent,
        None => true,
    })
}

/// The author of `hash`, if `hash` is an About Me entry. `None` otherwise.
fn about_me_author(hash: &ActionHash) -> ExternResult<Option<AgentPubKey>> {
    let action = must_get_action(hash.clone())?;
    let Some(entry_hash) = action.action().entry_hash() else {
        return Ok(None);
    };
    let entry = must_get_entry(entry_hash.clone())?;
    if AboutMe::try_from(entry.content.clone()).is_ok() {
        Ok(Some(action.action().author().clone()))
    } else {
        Ok(None)
    }
}

fn as_action_hash(hash: &AnyLinkableHash) -> Option<ActionHash> {
    hash.clone().into_action_hash()
}

fn validate_about_me(
    about_me: &AboutMe,
    author: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    // Only the person may speak as the person. Membership of a circle lets you
    // read it and acknowledge it; it does not let you author somebody else's
    // account of themselves.
    if !is_the_person(author)? {
        return invalid("Only the person whose circle this is may write their About Me");
    }
    if about_me.display_name.trim().is_empty() {
        return invalid("About Me must have a display name");
    }
    Ok(ValidateCallbackResult::Valid)
}

/// Links carry meaning here, so they need rules of their own.
///
/// Without this, a member could create an `AboutMeUpdates` link from the
/// person's own record to an entry of their own, and every reader following the
/// update chain would be shown the impostor's content as the person's current
/// record — without ever updating the person's entry, and so without tripping
/// the update-author rule.
fn validate_create_link(
    link_type: &LinkTypes,
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    let author = action.author();

    match link_type {
        // Only the person publishes their record to the circle index.
        LinkTypes::CircleToAboutMe => {
            if !is_the_person(author)? {
                return invalid("Only the person may publish an About Me to their circle");
            }
            let Some(target) = as_action_hash(&action.target_address) else {
                return invalid("Circle index must point at an action");
            };
            match about_me_author(&target)? {
                Some(a) if &a == author => Ok(ValidateCallbackResult::Valid),
                Some(_) => invalid("Circle index must point at the linker's own About Me"),
                None => invalid("Circle index must point at an About Me"),
            }
        }

        // The update chain. Both ends must be the person's own records, and
        // only the person may extend it.
        LinkTypes::AboutMeUpdates => {
            if !is_the_person(author)? {
                return invalid("Only the person may extend their own update chain");
            }
            let (Some(base), Some(target)) = (
                as_action_hash(&action.base_address),
                as_action_hash(&action.target_address),
            ) else {
                return invalid("Update links must join two actions");
            };
            match (about_me_author(&base)?, about_me_author(&target)?) {
                (Some(b), Some(t)) if &b == author && &t == author => {
                    Ok(ValidateCallbackResult::Valid)
                }
                (Some(_), Some(_)) => {
                    invalid("An update chain may only join the person's own About Me records")
                }
                _ => invalid("Update links must join two About Me records"),
            }
        }

        // You may only attach your own acknowledgement.
        LinkTypes::AboutMeToAcknowledgement => {
            let Some(target) = as_action_hash(&action.target_address) else {
                return invalid("Acknowledgement link must point at an action");
            };
            let target_action = must_get_action(target)?;
            if target_action.action().author() != author {
                return invalid("You may only link your own acknowledgement");
            }
            let Some(entry_hash) = target_action.action().entry_hash() else {
                return invalid("Acknowledgement link must point at an entry");
            };
            let entry = must_get_entry(entry_hash.clone())?;
            if Acknowledgement::try_from(entry.content.clone()).is_err() {
                return invalid("Acknowledgement link must point at an acknowledgement");
            }
            Ok(ValidateCallbackResult::Valid)
        }
    }
}

fn validate_acknowledgement(
    ack: &Acknowledgement,
    author: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    // The acknowledged record must exist and must actually be an About Me.
    let action = must_get_action(ack.about_me.clone())?;

    // You cannot acknowledge your own About Me. An acknowledgement is
    // evidence that somebody *else* read it; self-acknowledgement would
    // make that evidence worthless.
    if action.action().author() == author {
        return invalid("An agent cannot acknowledge their own About Me");
    }

    let entry_hash = action
        .action()
        .entry_hash()
        .ok_or_else(|| wasm_error!("Acknowledged action has no entry"))?;
    let entry = must_get_entry(entry_hash.clone())?;
    if AboutMe::try_from(entry.content.clone()).is_err() {
        return invalid("Acknowledgement must reference an About Me entry");
    }

    Ok(ValidateCallbackResult::Valid)
}

#[hdk_extern]
pub fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.flattened::<EntryTypes, LinkTypes>()? {
        // The membrane, enforced by the network rather than by the joiner.
        FlatOp::CreateRecord(OpRecord::AgentValidationPkg {
            membrane_proof,
            action,
        }) => check_membrane(action.author(), &membrane_proof),

        FlatOp::CreateEntry(OpEntry::CreateEntry { app_entry, action }) => match app_entry {
            EntryTypes::AboutMe(about_me) => validate_about_me(&about_me, action.author()),
            EntryTypes::Acknowledgement(ack) => validate_acknowledgement(&ack, action.author()),
        },
        FlatOp::Update(OpUpdate::Entry { app_entry, action }) => match app_entry {
            EntryTypes::AboutMe(about_me) => {
                // Only the original author may revise an About Me.
                // Every peer holding this checks it independently, so there
                // is no server to trust and nobody to ask for permission.
                let original = must_get_action(action.original_action_address.clone())?;
                if original.action().author() != action.author() {
                    return invalid("Only the original author may update an About Me");
                }
                validate_about_me(&about_me, action.author())
            }
            EntryTypes::Acknowledgement(_) => {
                invalid("Acknowledgements cannot be updated; write a new one")
            }
        },

        FlatOp::Link(OpLink::CreateLink {
            link_type, action, ..
        }) => validate_create_link(&link_type, &action),

        // Only the agent who made a link may remove it.
        FlatOp::Link(OpLink::DeleteLink {
            original_action,
            action,
            ..
        }) => {
            if original_action.author() != action.author() {
                return invalid("Only the agent who created a link may remove it");
            }
            Ok(ValidateCallbackResult::Valid)
        }

        // Only the author of an entry may delete it. Without this, any member
        // could erase the person's own record.
        FlatOp::Delete(OpDelete { action }) => {
            let deleted = must_get_action(action.deletes_address.clone())?;
            if deleted.action().author() != action.author() {
                return invalid("Only the author of a record may delete it");
            }
            Ok(ValidateCallbackResult::Valid)
        }

        // Everything left is Holochain's own bookkeeping (chain opens and
        // closes, init markers, agent activity). Nothing app-specific rides on
        // these, so there is nothing for this app to rule on.
        _ => Ok(ValidateCallbackResult::Valid),
    }
}
