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
