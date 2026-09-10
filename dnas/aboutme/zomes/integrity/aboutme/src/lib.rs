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
/// The seven sections of the PRSB About Me standard, v2.0.1 (May 2025), in the
/// order the standard lists them, plus the one piece of its metadata that says
/// something this app could not otherwise say.
///
/// Free text only. The standard also allows a coded value and multi-media
/// against each section, and neither is here — so this is a subset of About
/// Me, not an implementation of it, and must never be described as conformant.
/// Conformance is an assessed process with a quality mark, which this has not
/// been through. See `docs/standard-and-gap.md`.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct AboutMe {
    pub display_name: String,
    /// What is most important to me
    pub what_matters_to_me: String,
    /// People who are important to me
    pub people_who_matter: String,
    /// How I communicate and how to communicate with me
    pub how_to_communicate_with_me: String,
    /// My wellness
    pub my_wellness: String,
    /// Please do and please do not
    pub please_do_and_please_do_not: String,
    /// How and when to support me
    pub how_to_support_me: String,
    /// Also worth knowing about me
    pub also_worth_knowing: String,
    /// Supported to write this by.
    ///
    /// The standard's own acknowledgement that the person whose record this is
    /// may not be the person typing. This project already believed that —
    /// where somebody cannot hold their own circle, a daughter or a case
    /// manager holds it — but the record never said so on its own face. Signed
    /// authorship proves who wrote each version cryptographically; this says it
    /// in words, in the record, where somebody reading it will see it.
    ///
    /// Self-declared like everything else here, and never inferred.
    pub supported_to_write_this_by: String,
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

/// Who somebody in the circle is, in their own words.
///
/// Each member writes their own, and only their own. Nobody is labelled by
/// anyone else. "Her son", "district nurse", "support worker, Tuesdays" —
/// these are how people describe themselves, not roles an administrator
/// assigned them.
///
/// Not a credential. Nothing here is checked, and the interface must never
/// imply it was.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Member {
    pub name: String,
    /// How they relate to the person this circle is about.
    pub relationship: String,
}

/// Somebody the holder wants to let in, waiting on the second agreement.
///
/// **Why this exists at all.** Where a circle asks two people to agree, both
/// agreements are signatures, and they used to reach each other by hand: the
/// holder sent half an invitation to the second person, who sent it back
/// finished, who sent it on. Five copy-and-pastes of near-identical text
/// between two devices that were already members of the same circle and
/// perfectly able to talk to each other. People pasted the wrong line into the
/// wrong box, which is not a mistake anybody should be given the chance to
/// make about who may read a vulnerable person's record.
///
/// So the agreements travel in the circle instead. This is the first half,
/// written by the holder where the second person will see it.
///
/// **The membrane is untouched by this.** Whoever joins still presents both
/// signatures as their membrane proof, because somebody who has not joined yet
/// cannot read anything here. This changes how the two signatures reach one
/// another, and nothing about what the door checks.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct ProposedMember {
    /// The key that would be admitted.
    pub invitee: AgentPubKey,

    /// Who the holder says that key belongs to.
    ///
    /// A claim, exactly like the role on an acknowledgement, and nothing here
    /// or anywhere else checks it. It is carried because the person being
    /// asked to agree cannot answer "should this key be let in?" and can
    /// answer "should Ronnie, her cousin, be let in?".
    pub name: String,

    /// The holder's own agreement: her signature over `invitee`.
    ///
    /// Kept here so the second person's device can build a finished invitation
    /// without anybody copying anything. Validation checks it really is hers
    /// and really is over that key, so a proposal cannot carry a signature
    /// that would fail at the door later — a promise that breaks quietly the
    /// day somebody tries to use it.
    pub signature: Signature,
}

/// Somebody asking to be let into a circle they cannot see.
///
/// Written in a waiting room, which is an open network holding nothing but
/// these and the answers to them. The person knocking brings their own key by
/// arriving, which is the whole point: nobody has to collect an identifier
/// from them first, and that was the step where this stopped being possible
/// for anybody elderly or being helped.
///
/// Everything here is a claim, checked by nobody. It is what the people
/// deciding have to go on, and the interface must present it as somebody's
/// word rather than as a fact — the same rule as every other name in this
/// app.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Knock {
    /// What they call themselves.
    pub name: String,
    /// How they say they are connected. "Her cousin", "district nurse".
    pub relationship: String,
}

/// The answer to a knock, left where the person who knocked can collect it.
///
/// **Safe in the open, and that is a property rather than a hope.** What this
/// carries is an invitation, and an invitation is signed over one person's own
/// key — so it admits nobody else and is a useless blob to anybody who picks
/// it up. That is what lets the answer be left lying in a room anyone may
/// enter, instead of having to be carried by hand to the one person it is for.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Admission {
    /// Which knock this answers.
    pub knock: ActionHash,

    /// The invitation, as the one line of text the app passes around anyway.
    ///
    /// Opaque here on purpose. This room does not know what a circle is, what
    /// its rules are, or who its members are; it carries a question in and an
    /// answer out. Whether the answer opens anything is settled at the
    /// circle's own door, by every peer there, exactly as before.
    pub invitation: String,
}

/// The second agreement, given in the circle rather than by hand.
///
/// Only the person this circle names may write one, and every peer checks that
/// independently — the same rule the membrane applies, applied here so that a
/// second yes cannot be manufactured by anybody else inside the circle.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Endorsement {
    /// The proposal being agreed to.
    pub proposed: ActionHash,

    /// The seconder's signature over the same key the holder signed.
    pub signature: Signature,
}

/// Which part of the record a suggestion is about.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AboutMeField {
    WhatMattersToMe,
    PeopleWhoMatter,
    HowToCommunicateWithMe,
    MyWellness,
    PleaseDoAndPleaseDoNot,
    HowToSupportMe,
    AlsoWorthKnowing,
}

/// Something a member of the circle thinks should be in the record.
///
/// **Anyone in the circle may write one of these, and only the holder may
/// accept it.** That split is the point. A son remembers what his mother
/// enjoyed; a support worker notices what settles her. A record that only one
/// person may write loses all of it.
///
/// The record keeps one voice. The knowledge is allowed in from everywhere.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Suggestion {
    pub field: AboutMeField,
    pub text: String,
    /// Optional: why they think so. "She talked about the allotment all
    /// summer." Often the more useful half.
    pub because: String,
}

/// What the holder decided about a suggestion.
///
/// Kept as its own record rather than by deleting the suggestion, so the trail
/// survives: who offered something, and what became of it. Somebody who takes
/// the trouble to notice a thing about a person deserves better than for it to
/// vanish silently.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct SuggestionOutcome {
    pub suggestion: ActionHash,
    /// True when it went into the record. False when it was set aside — which
    /// is not a judgement on the person who offered it.
    pub accepted: bool,
}

#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    AboutMe(AboutMe),
    Acknowledgement(Acknowledgement),
    Suggestion(Suggestion),
    SuggestionOutcome(SuggestionOutcome),
    Member(Member),
    // Appended, so every existing entry type keeps the index it already had.
    ProposedMember(ProposedMember),
    Endorsement(Endorsement),
    // The waiting room's whole vocabulary: ask, and be answered.
    Knock(Knock),
    Admission(Admission),
}

#[hdk_link_types]
pub enum LinkTypes {
    /// Anchor -> AboutMe, so somebody joining the circle can find it.
    CircleToAboutMe,
    /// Original AboutMe -> its updates.
    AboutMeUpdates,
    /// AboutMe version -> acknowledgements of that version.
    AboutMeToAcknowledgement,
    /// Anchor -> Suggestion, so the holder can find what has been offered.
    CircleToSuggestion,
    /// Suggestion -> what the holder decided about it.
    SuggestionToOutcome,
    /// Anchor -> Member, so the circle can put names to people.
    CircleToMember,
    /// Anchor -> ProposedMember, so the person who has to agree finds what is
    /// waiting for them rather than being sent it.
    CircleToProposedMember,
    /// ProposedMember -> the second agreement on it.
    ProposedMemberToEndorsement,
    /// Anchor -> Knock, so the people deciding find who is asking.
    WaitingRoomToKnock,
    /// Knock -> the answer to it, where the person who knocked will look.
    KnockToAdmission,
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

    /// Somebody whose agreement is also needed before anyone may join, as a
    /// base64 agent key. Optional: most circles will not have one.
    ///
    /// The safeguarding case this exists for is not a stranger breaking in.
    /// It is somebody being *talked into* letting a person in — a plausible
    /// caller, a new "friend", a relative nobody trusts. The holder is the
    /// person under that pressure, so a rule the holder can waive alone is not
    /// a safeguard at all.
    ///
    /// So it lives here, in the properties, where it forms part of the DNA
    /// hash and every peer enforces it independently. It cannot be turned off
    /// under pressure, because turning it off would be a different circle.
    /// The cost of that is honest and worth stating: changing who the seconder
    /// is means re-forming the circle. Circles are clones, so that is cheap,
    /// and it is the same answer this project gives to revocation.
    ///
    /// Compare RIX Multi Me's "Buddy", who can *veto* a share. This is the
    /// other way round, and stronger: nothing happens unless the seconder
    /// actively agrees. A veto has to arrive in time to stop something; a
    /// second signature simply does not exist until it is given.
    pub seconder: Option<String>,

    /// The circle this waiting room serves, as a base64 agent key.
    ///
    /// **A waiting room is how somebody gets in without anybody collecting
    /// their identifier first.** A circle is closed, so a person outside it
    /// cannot write to it — which is the membrane working, and also why
    /// joining used to begin with "send me the long line of characters from
    /// your app". For somebody elderly, or somebody being helped, that is the
    /// step where it stops being possible.
    ///
    /// So the holder shares one address, which never changes and can go to
    /// anybody: a family group, a phone call, a note. It leads to an open
    /// network where the only thing anybody can do is knock — say who they
    /// are and ask. They bring their own key with them by arriving.
    ///
    /// Whoever knocks is admitted or not by the circle's own rules, which
    /// this room knows nothing about. All it does is carry the question in
    /// and the answer out.
    ///
    /// Nothing about anybody's record is here, and nothing can be: the
    /// entries this room permits are a knock and an answer to one, and every
    /// other write is refused exactly as in the plain lobby.
    #[serde(default)]
    pub waiting_for: Option<String>,
}

/// What an invited person presents when they join.
///
/// The founder's signature over the invitee's public key, and — where the
/// circle names a seconder — that person's signature over the same key. Nobody
/// else can produce either, and every peer can check both without asking
/// anyone.
#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct Invitation {
    pub signature: Signature,

    /// The seconder's signature over the same invitee key.
    ///
    /// `None` is the ordinary case for a circle with no seconder. In a circle
    /// that has one, an invitation without this is simply incomplete — it is
    /// not a weaker invitation, it is not one yet.
    #[serde(default)]
    pub seconded: Option<Signature>,
}

/// How this circle decides who belongs.
pub enum Membrane {
    /// A real circle, closed around one person.
    ///
    /// The second key, where there is one, is somebody who must also agree
    /// before anybody joins.
    Founder(AgentPubKey, Option<AgentPubKey>),
    /// A shared launching point. Anyone may join it; nobody may write in it.
    /// See `lobby`.
    Lobby,
    /// A waiting room for one circle, holding the key of whoever holds that
    /// circle. Anyone may join and knock; only that person may answer.
    WaitingRoom(AgentPubKey),
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

    // A seconder that cannot be read is not a seconder that can be ignored.
    // Naming one and getting it wrong closes the circle, exactly as a
    // mistyped founder does: absence of configuration must never mean absence
    // of a membrane, and neither must a typo in it.
    let seconder = match p.seconder.as_deref() {
        None => None,
        Some(text) => match AgentPubKey::try_from(text.trim()) {
            Ok(key) => Some(key),
            Err(_) => return Ok(Membrane::Misconfigured),
        },
    };

    // A waiting room names the circle it serves. Unreadable is closed, for
    // the same reason a mistyped founder is: a room nobody can be admitted
    // from is visibly broken, and one that admits on a typo is not.
    let waiting_for = match p.waiting_for.as_deref() {
        None => None,
        Some(text) => match AgentPubKey::try_from(text.trim()) {
            Ok(key) => Some(key),
            Err(_) => return Ok(Membrane::Misconfigured),
        },
    };

    match p.founder {
        Some(founder) => match AgentPubKey::try_from(founder.as_str()) {
            Ok(key) => Ok(Membrane::Founder(key, seconder)),
            Err(_) => Ok(Membrane::Misconfigured),
        },
        // Checked before the plain lobby, because a room that serves a circle
        // is a different thing from the empty room the app is installed with,
        // and only one of them lets anybody write anything.
        None => match waiting_for {
            Some(holder) => Ok(Membrane::WaitingRoom(holder)),
            None if p.lobby => Ok(Membrane::Lobby),
            None => Ok(Membrane::Misconfigured),
        },
    }
}

fn check_membrane(
    agent: &AgentPubKey,
    membrane_proof: &Option<MembraneProof>,
) -> ExternResult<ValidateCallbackResult> {
    let (founder, seconder) = match membrane()? {
        Membrane::Founder(key, seconder) => (key, seconder),
        Membrane::Lobby => return Ok(ValidateCallbackResult::Valid),
        // A waiting room is open on purpose. Being in it is not being in
        // anything — the only thing it holds is people asking, and the
        // answers to them, neither of which is anybody's record.
        Membrane::WaitingRoom(_) => return Ok(ValidateCallbackResult::Valid),
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
    if !verify_signature(founder, invitation.signature, agent.clone())? {
        return invalid("Invitation was not issued by the person whose circle this is");
    }

    // Where this circle names somebody who must also agree, their signature is
    // over the same key, and it is checked here by every peer rather than
    // anywhere it could be skipped.
    if let Some(seconder) = seconder {
        /*
         * The seconder needs no second agreement to their own admission.
         *
         * They were chosen by the holder, and asking them to countersign
         * their own way in adds nothing anybody could check. It also removes
         * a bootstrap problem that made the whole feature awkward: without
         * this, the one person who must agree to every arrival cannot get in
         * without agreeing to themselves from outside a circle they are not
         * in yet. They come in first, on the holder's invitation alone, and
         * from then on nobody else arrives without them.
         */
        if agent == &seconder {
            return Ok(ValidateCallbackResult::Valid);
        }

        let Some(seconded) = invitation.seconded else {
            return invalid(
                "This circle asks two people to agree before anybody joins, and \
                 this invitation only has one of them",
            );
        };
        if !verify_signature(seconder, seconded, agent.clone())? {
            return invalid(
                "The second agreement on this invitation is not from the person \
                 this circle asks to give it",
            );
        }
    }

    Ok(ValidateCallbackResult::Valid)
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
        Membrane::Founder(f, _) => &f == agent,
        // A lobby holds nothing and accepts nothing. It exists so the app can
        // be installed and can then clone real circles from it.
        Membrane::Lobby => false,
        // Nor in a waiting room. Nobody's record is there to speak as, and
        // the holder being named in its properties does not make it hers to
        // write in — it is a doorstep, not a room in the house.
        Membrane::WaitingRoom(_) => false,
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

        // Anyone in the circle may offer a suggestion, but only their own.
        LinkTypes::CircleToSuggestion => {
            let Some(target) = as_action_hash(&action.target_address) else {
                // Path anchor scaffolding. See the note under CircleToAboutMe.
                return Ok(ValidateCallbackResult::Valid);
            };
            let target_action = must_get_action(target)?;
            if target_action.action().author() != author {
                return invalid("You may only post your own suggestion");
            }
            Ok(ValidateCallbackResult::Valid)
        }

        // Anyone may say who they are, and only about themselves.
        LinkTypes::CircleToMember => {
            let Some(target) = as_action_hash(&action.target_address) else {
                return Ok(ValidateCallbackResult::Valid);
            };
            let target_action = must_get_action(target)?;
            if target_action.action().author() != author {
                return invalid("You may only introduce yourself");
            }
            Ok(ValidateCallbackResult::Valid)
        }

        // Only the holder records what they decided.
        LinkTypes::SuggestionToOutcome => {
            if !is_the_person(author)? {
                return invalid("Only the person whose circle this is may decide on a suggestion");
            }
            Ok(ValidateCallbackResult::Valid)
        }

        // Only the holder puts somebody forward, and only her own proposals.
        LinkTypes::CircleToProposedMember => {
            if !is_the_person(author)? {
                return invalid("Only the person whose circle this is may propose somebody");
            }
            // Path anchor scaffolding. See the note under CircleToAboutMe.
            let Some(target) = as_action_hash(&action.target_address) else {
                return Ok(ValidateCallbackResult::Valid);
            };
            let target_action = must_get_action(target)?;
            if target_action.action().author() != author {
                return invalid("The circle's list must point at the holder's own proposal");
            }
            Ok(ValidateCallbackResult::Valid)
        }

        // Only the person this circle asks, and only their own agreement.
        //
        // Without the second half of this a member could link somebody else's
        // entry here and have it read as an agreement; the entry rule would
        // still hold, but the list is what everyone actually reads.
        LinkTypes::ProposedMemberToEndorsement => {
            if !is_the_seconder(author)? {
                return invalid(
                    "Only the person this circle asks to agree may record an agreement",
                );
            }
            let Some(target) = as_action_hash(&action.target_address) else {
                return invalid("An agreement link must point at an action");
            };
            let target_action = must_get_action(target)?;
            if target_action.action().author() != author {
                return invalid("You may only link your own agreement");
            }
            Ok(ValidateCallbackResult::Valid)
        }

        // Anybody may knock, and only about themselves.
        LinkTypes::WaitingRoomToKnock => {
            if who_this_room_serves()?.is_none() {
                return invalid("Knocking only means something in a circle's waiting room");
            }
            // Path anchor scaffolding. See the note under CircleToAboutMe.
            let Some(target) = as_action_hash(&action.target_address) else {
                return Ok(ValidateCallbackResult::Valid);
            };
            let target_action = must_get_action(target)?;
            if target_action.action().author() != author {
                return invalid("You may only knock for yourself");
            }
            Ok(ValidateCallbackResult::Valid)
        }

        // Only the holder answers, and only with their own answer.
        LinkTypes::KnockToAdmission => {
            match who_this_room_serves()? {
                Some(holder) if &holder == author => {}
                _ => return invalid("Only the person whose circle this is may answer a knock"),
            }
            let Some(target) = as_action_hash(&action.target_address) else {
                return invalid("An answer link must point at an action");
            };
            let target_action = must_get_action(target)?;
            if target_action.action().author() != author {
                return invalid("You may only link your own answer");
            }
            Ok(ValidateCallbackResult::Valid)
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

/// Whoever holds the circle a waiting room serves, if this is one.
fn who_this_room_serves() -> ExternResult<Option<AgentPubKey>> {
    Ok(match membrane()? {
        Membrane::WaitingRoom(holder) => Some(holder),
        _ => None,
    })
}

/// Anybody may knock, and only in a room built for it.
///
/// No check on who the author is, deliberately — a room where you must
/// already be known in order to ask is not a waiting room, it is the closed
/// door it was meant to replace.
///
/// What is checked is that this is a waiting room at all. Without that, these
/// entries would be writable in the plain lobby every installation shares,
/// which would put "somebody wants to join Margaret Smythe's circle" in front
/// of every person who ever installs this app.
fn validate_knock(knock: &Knock) -> ExternResult<ValidateCallbackResult> {
    if who_this_room_serves()?.is_none() {
        return invalid("Knocking only means something in a circle's waiting room");
    }
    if knock.name.trim().is_empty() {
        return invalid("Say what you are called, so they know who is asking");
    }
    Ok(ValidateCallbackResult::Valid)
}

/// Only the holder of the circle answers knocks at its door.
///
/// Without this, anybody in the room could answer one — and an answer is an
/// invitation. It would not admit them anywhere, because a forged invitation
/// carries no valid signature and the circle's own door still checks that. But
/// it would let a stranger hand somebody a thing that looks like a welcome and
/// silently is not, which is its own kind of cruelty.
fn validate_admission(
    admission: &Admission,
    author: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    let Some(holder) = who_this_room_serves()? else {
        return invalid("There is nothing to answer outside a circle's waiting room");
    };
    if &holder != author {
        return invalid("Only the person whose circle this is may answer a knock");
    }

    // It must answer a real knock, so an admission cannot be left floating and
    // later read as an answer to somebody.
    let action = must_get_action(admission.knock.clone())?;
    let Some(entry_hash) = action.action().entry_hash() else {
        return invalid("An answer must refer to a knock");
    };
    let entry = must_get_entry(entry_hash.clone())?;
    if Knock::try_from(entry.content.clone()).is_err() {
        return invalid("An answer must refer to a knock");
    }

    Ok(ValidateCallbackResult::Valid)
}

/// Is this agent the one this circle asks to agree as well?
///
/// Read from the circle's own identity, exactly as the membrane reads it, so
/// the rule inside the circle and the rule at the door cannot drift apart. A
/// circle with nobody named asks nobody, and nobody can therefore endorse.
fn is_the_seconder(agent: &AgentPubKey) -> ExternResult<bool> {
    Ok(match membrane()? {
        Membrane::Founder(_, Some(seconder)) => &seconder == agent,
        Membrane::Founder(_, None) => false,
        Membrane::Lobby => false,
        // A waiting room asks nobody to agree. What happens to a knock is
        // settled in the circle, which is where both agreements live.
        Membrane::WaitingRoom(_) => false,
        Membrane::Misconfigured => false,
    })
}

/// The holder's half of a two-person admission.
///
/// Two rules, and the second is the one that matters. Anybody could otherwise
/// write a proposal carrying a signature that is not really the holder's, or
/// not really over that key — and nothing would notice until the person it
/// names tried to join and was turned away by the door for reasons nobody
/// could see. A promise that breaks silently, later, is worse than a refusal
/// now.
fn validate_proposed_member(
    proposed: &ProposedMember,
    author: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    if !is_the_person(author)? {
        return invalid("Only the person whose circle this is may propose somebody");
    }

    if !verify_signature(
        author.clone(),
        proposed.signature.clone(),
        proposed.invitee.clone(),
    )? {
        return invalid(
            "A proposal must carry the holder's own agreement, over the key it names",
        );
    }

    Ok(ValidateCallbackResult::Valid)
}

/// The second agreement, checked by every peer rather than taken on trust.
///
/// This is the rule that makes the whole safeguard worth having. Without it
/// any member could write an endorsement and the holder's device would
/// assemble an invitation from it — a second yes given by somebody who was
/// never asked for one, which is precisely the thing this feature exists to
/// prevent.
fn validate_endorsement(
    endorsement: &Endorsement,
    author: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    if !is_the_seconder(author)? {
        return invalid(
            "Only the person this circle asks to agree may give the second agreement",
        );
    }

    // It must be about a real proposal, so an endorsement cannot be attached
    // to something else and later read as agreement to somebody.
    let action = must_get_action(endorsement.proposed.clone())?;
    let Some(entry_hash) = action.action().entry_hash() else {
        return invalid("An agreement must refer to a proposed member");
    };
    let entry = must_get_entry(entry_hash.clone())?;
    let Ok(proposed) = ProposedMember::try_from(entry.content.clone()) else {
        return invalid("An agreement must refer to a proposed member");
    };

    // Over the same key the holder signed, and nothing else.
    if !verify_signature(
        author.clone(),
        endorsement.signature.clone(),
        proposed.invitee.clone(),
    )? {
        return invalid("The second agreement must be over the key the proposal names");
    }

    Ok(ValidateCallbackResult::Valid)
}

fn validate_member(member: &Member) -> ExternResult<ValidateCallbackResult> {
    // Deliberately no check on who the author is: everyone describes
    // themselves. The only rule is that they say something.
    if member.name.trim().is_empty() {
        return invalid("Tell the circle what you are called");
    }
    Ok(ValidateCallbackResult::Valid)
}

fn validate_suggestion(suggestion: &Suggestion) -> ExternResult<ValidateCallbackResult> {
    // Deliberately no check on who the author is. Any member of the circle may
    // offer something; the holder decides what goes in.
    if suggestion.text.trim().is_empty() {
        return invalid("A suggestion needs something in it");
    }
    Ok(ValidateCallbackResult::Valid)
}

fn validate_outcome(
    outcome: &SuggestionOutcome,
    author: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    if !is_the_person(author)? {
        return invalid(
            "Only the person whose circle this is may accept or set aside a suggestion",
        );
    }

    // It must actually be a suggestion, so an outcome cannot be used to
    // silently mark something else as settled.
    let action = must_get_action(outcome.suggestion.clone())?;
    let Some(entry_hash) = action.action().entry_hash() else {
        return invalid("An outcome must refer to a suggestion");
    };
    let entry = must_get_entry(entry_hash.clone())?;
    if Suggestion::try_from(entry.content.clone()).is_err() {
        return invalid("An outcome must refer to a suggestion");
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
            EntryTypes::Suggestion(s) => validate_suggestion(&s),
            EntryTypes::SuggestionOutcome(o) => validate_outcome(&o, action.author()),
            EntryTypes::Member(m) => validate_member(&m),
            EntryTypes::ProposedMember(p) => validate_proposed_member(&p, action.author()),
            EntryTypes::Endorsement(e) => validate_endorsement(&e, action.author()),
            EntryTypes::Knock(k) => validate_knock(&k),
            EntryTypes::Admission(a) => validate_admission(&a, action.author()),
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
            // A member may correct their own suggestion before it is decided.
            EntryTypes::Suggestion(s) => {
                let original = must_get_action(action.original_action_address.clone())?;
                if original.action().author() != action.author() {
                    return invalid("Only the person who offered a suggestion may change it");
                }
                validate_suggestion(&s)
            }
            EntryTypes::SuggestionOutcome(_) => {
                invalid("A decision cannot be edited; make a new one")
            }
            // You may correct how you describe yourself, and only your own.
            EntryTypes::Member(m) => {
                let original = must_get_action(action.original_action_address.clone())?;
                if original.action().author() != action.author() {
                    return invalid("Only you may change how you are described");
                }
                validate_member(&m)
            }
            /*
             * Neither of these can be edited, and both refusals are the same
             * refusal: an agreement is a thing that was given at a moment, and
             * a record of it that can be rewritten afterwards is not evidence
             * of anything.
             *
             * Changing your mind about who may join is possible and costs
             * nothing — propose somebody else, or simply never agree. What is
             * not possible is altering what was already agreed to.
             */
            EntryTypes::ProposedMember(_) => {
                invalid("A proposal cannot be changed; make another one")
            }
            EntryTypes::Endorsement(_) => {
                invalid("An agreement cannot be changed once it is given")
            }
            // You may knock again; you may not rewrite the knock somebody has
            // already read and is deciding about.
            EntryTypes::Knock(_) => invalid("A knock cannot be changed; knock again"),
            EntryTypes::Admission(_) => invalid("An answer cannot be changed"),
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
