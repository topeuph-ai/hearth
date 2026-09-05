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
    /// The person whose circle this is, or whoever acts for them.
    pub founder: AgentPubKey,
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
    match CircleProperties::try_from(properties) {
        Ok(p) => Ok(Some(p.founder)),
        Err(_) => Ok(None),
    }
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

fn validate_about_me(about_me: &AboutMe) -> ExternResult<ValidateCallbackResult> {
    if about_me.display_name.trim().is_empty() {
        return invalid("About Me must have a display name");
    }
    Ok(ValidateCallbackResult::Valid)
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
            EntryTypes::AboutMe(about_me) => validate_about_me(&about_me),
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
                validate_about_me(&about_me)
            }
            EntryTypes::Acknowledgement(_) => {
                invalid("Acknowledgements cannot be updated; write a new one")
            }
        },
        _ => Ok(ValidateCallbackResult::Valid),
    }
}
