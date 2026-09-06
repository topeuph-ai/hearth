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
    pub founder: Option<String>,

    /// Mark this cell as a lobby: anyone may join, **nobody may write**.
    ///
    /// The provisioned cell of the app is a lobby. It exists only so the app
    /// is installable and can then clone real circles from it. Everyone who
    /// installs the app shares it, so it must hold nothing: joining is
    /// unrestricted precisely because there is nothing there to reach.
    ///
    /// It must be *stated*. A DNA with no properties at all, or with a typo
    /// where the founder should be, is closed to everybody rather than open to
    /// everybody. Absence of configuration must never mean absence of a
    /// membrane.
    #[serde(default)]
    pub lobby: bool,
}

/// What an invited person presents when they join.
///
/// It is simply the founder's signature over the invitee's public key. Nobody
/// else can produce one, and every peer can check it without asking anyone.
#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct Invitation {
    pub signature: Signature,
}

/// How this circle decides who belongs.
pub enum Membrane {
    /// A real circle, closed around one person.
    Founder(AgentPubKey),
    /// A shared launching point. Anyone may join it; nobody may write in it.
    /// See `lobby`.
    Lobby,
    /// No usable configuration. Nobody may join and nobody may write.
    Misconfigured,
}

/// Read the membrane from the DNA properties.
///
/// **Fails closed.** No properties, unreadable properties, or a founder that
/// is not a valid agent key all produce `Misconfigured`, which admits nobody.
/// The only way to obtain an open circle is to ask for one in writing.
pub fn membrane() -> ExternResult<Membrane> {
    let properties = dna_info()?.modifiers.properties;
    let Ok(p) = CircleProperties::try_from(properties) else {
        return Ok(Membrane::Misconfigured);
    };

    match p.founder {
        Some(founder) => match AgentPubKey::try_from(founder.as_str()) {
            Ok(key) => Ok(Membrane::Founder(key)),
            Err(_) => Ok(Membrane::Misconfigured),
        },
        None if p.lobby => Ok(Membrane::Lobby),
        None => Ok(Membrane::Misconfigured),
    }
}

fn check_membrane(
    agent: &AgentPubKey,
    membrane_proof: &Option<MembraneProof>,
) -> ExternResult<ValidateCallbackResult> {
    let founder = match membrane()? {
        Membrane::Founder(key) => key,
        Membrane::Lobby => return Ok(ValidateCallbackResult::Valid),
        Membrane::Misconfigured => {
            return invalid(
                "This circle has no founder configured, so nobody may join it. \
                 Set `founder`, or `lobby: true` if that is what you meant.",
            )
        }
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
/// Only a real circle has such a person. In a lobby the answer is nobody,
/// which is what makes a lobby unwritable.
fn is_the_person(agent: &AgentPubKey) -> ExternResult<bool> {
    Ok(match membrane()? {
        Membrane::Founder(f) => &f == agent,
        // A lobby holds nothing and accepts nothing. It exists so the app can
        // be installed and can then clone real circles from it.
        Membrane::Lobby => false,
        // Unreachable in practice, since nobody can join a misconfigured
        // circle. Written as a refusal anyway: the default answer to "may
        // this agent speak as the person" is no.
        Membrane::Misconfigured => false,
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
            // `Path::ensure` builds the anchor tree with links of this same
            // type, and those point at Path entries rather than at records.
            // Rejecting them broke every write, including the person's own.
            // They are structural, and readers ignore anything that is not an
            // action hash.
            let Some(target) = as_action_hash(&action.target_address) else {
                return Ok(ValidateCallbackResult::Valid);
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

// ---------------------------------------------------------------------------
// Deterministic ordering of versions
// ---------------------------------------------------------------------------

/// Order update links so that every peer agrees which version is newest.
///
/// Sorting by timestamp alone is not enough. Two links can carry the same
/// timestamp, and `get_links` makes no promise that peers receive links in the
/// same order — so without a tiebreak two devices could show different versions
/// of the same person's record and both believe they were current.
///
/// The action hash is the tiebreak: arbitrary, but identical everywhere.
pub fn order_versions(mut versions: Vec<(Timestamp, ActionHash)>) -> Vec<ActionHash> {
    versions.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    versions.into_iter().map(|(_, hash)| hash).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(byte: u8) -> ActionHash {
        ActionHash::from_raw_36(vec![byte; 36])
    }

    /// The case that would otherwise be invisible: identical timestamps,
    /// different arrival order. Every peer must still agree.
    #[test]
    fn identical_timestamps_still_order_identically() {
        let t = Timestamp::from_micros(1_000);
        let one_peer = order_versions(vec![(t, hash(1)), (t, hash(2)), (t, hash(3))]);
        let another = order_versions(vec![(t, hash(3)), (t, hash(1)), (t, hash(2))]);
        let a_third = order_versions(vec![(t, hash(2)), (t, hash(3)), (t, hash(1))]);

        assert_eq!(one_peer, another);
        assert_eq!(another, a_third);
        assert_eq!(one_peer.last(), Some(&hash(3)));
    }

    #[test]
    fn later_timestamps_win_regardless_of_hash() {
        let earlier = Timestamp::from_micros(1_000);
        let later = Timestamp::from_micros(2_000);

        // The later version has the lower hash, so a hash-only sort would
        // put it first.
        let ordered = order_versions(vec![(earlier, hash(9)), (later, hash(1))]);
        assert_eq!(ordered.last(), Some(&hash(1)));
    }

    #[test]
    fn a_single_version_is_returned_unchanged() {
        let ordered = order_versions(vec![(Timestamp::from_micros(1), hash(7))]);
        assert_eq!(ordered, vec![hash(7)]);
    }
}
