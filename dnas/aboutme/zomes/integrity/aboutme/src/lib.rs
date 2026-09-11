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

/// Who the holder has asked to agree to who joins.
///
/// Written in the circle rather than baked into its identity, which is the
/// whole point: **it can be written again.** If the person appointed dies,
/// loses the device their keys were on, or simply has to be replaced, the
/// holder appoints somebody else and the circle carries on. Before this, that
/// situation had no way out but a new circle and everybody re-invited.
///
/// Append-only, like every other agreement here. Appointing somebody new does
/// not erase who was appointed before, and the record of who was trusted with
/// this, and when, stays in the circle where everybody can see it. That
/// visibility is what the safeguard actually rests on now.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Appointment {
    /// The person asked to agree. Never the holder herself — a safeguard the
    /// person under pressure can satisfy alone is not one.
    pub agrees: AgentPubKey,
}

/// An answer to being asked to agree to who joins.
///
/// Being appointed used to be something done *to* somebody. The holder wrote
/// their key into the circle and that was that: they might not know, might
/// not want it, and might never have been asked. The circle would then sit
/// waiting on a person who had not agreed to anything, and nothing on any
/// screen would say so.
///
/// Nobody can be made to agree to an arrival — refusing is always possible
/// by simply never writing an endorsement. What this adds is that refusing
/// can be *said*, out loud, where the holder sees it and can appoint
/// somebody else. A silence and a no look identical until one of them is
/// written down.
///
/// Write another to change your mind. The newest one is the answer, and the
/// older ones stay where everybody can see them, like everything else here.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Consent {
    /// The appointment being answered.
    ///
    /// Named rather than looked up, for the reason that runs through all of
    /// this: who is appointed changes, and validation must reach the same
    /// answer on every machine forever.
    pub appointment: ActionHash,

    /// Whether they are willing.
    pub willing: bool,
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
///
/// **And it is sealed, because the room is not private.** Anybody with a
/// circle's address can read everything written in its waiting room. Once
/// the room is the only way in, that means every arrival is announced in
/// the open: "Ronnie Smythe, her cousin", readable by everybody who has
/// ever been given the address. Who visits somebody is itself sensitive —
/// a psychiatrist, a substance misuse worker, a domestic abuse advocate.
///
/// So the words are boxed to the holder. What stays in the open is the key
/// that wrote the knock, which cannot be hidden: it is the action's author,
/// and it is the whole reason nobody had to collect it by hand.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Knock {
    /// The words, readable by the holder of the circle this room serves.
    pub for_the_holder: XSalsa20Poly1305EncryptedData,

    /// The same words, readable by whoever wrote them.
    ///
    /// Boxing is between two keys and opened with the recipient's secret,
    /// so sealing to the holder alone would leave the sender unable to read
    /// their own knock. That matters: somebody let in after a restart used
    /// to arrive nameless, in a circle that then asked them who they were
    /// when they had already said. Their app reads it back from here.
    pub for_me: XSalsa20Poly1305EncryptedData,
}

/// What is inside a sealed knock.
///
/// Not an entry: it never touches the DHT unencrypted. It is the shape both
/// ends agree on, encoded, boxed, and unboxed again.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WhoIsKnocking {
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

    /// The appointment this agreement is given under.
    ///
    /// Named for the same reason the invitation names it: who is appointed can
    /// change, and validation must reach the same answer on every machine
    /// forever. A fixed hash is a question with one answer.
    pub appointment: ActionHash,

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
    Appointment(Appointment),
    Consent(Consent),
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
    /// Anchor -> Appointment, so the circle can see who has been asked
    /// to agree to who joins, and when.
    CircleToAppointment,
    /// Appointment -> the answer to it, so the holder finds out whether the
    /// person she asked is willing without having to go and ask them again.
    AppointmentToConsent,
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

    /// Whether this circle asks two people to agree before anybody joins.
    ///
    /// **The rule lives here; the person does not.** Naming the person here
    /// made them permanent, because this forms part of the circle's identity
    /// and identity cannot be edited. If they died, lost their device, or
    /// simply had to be replaced, the only way out was a new circle with
    /// everybody re-invited — and for a record about somebody in declining
    /// health, one of the two people becoming unable to answer is not an edge
    /// case. It is the expected course of events.
    ///
    /// So who agrees is an entry the holder writes, and can write again. She
    /// can appoint somebody the day she meets them, and appoint somebody else
    /// the day the first person is past helping.
    ///
    /// **What this costs, stated plainly.** When the person was named here,
    /// the network refused to admit anybody without their signature. Now the
    /// holder can issue an invitation citing no appointment at all, and it
    /// will be accepted — because "has she appointed anybody yet?" is a
    /// question whose answer changes, and validation must give the same
    /// answer on every machine forever.
    ///
    /// So this is enforced by **being visible**, not by being impossible.
    /// Every admission records whether it was seconded and by whom, where the
    /// whole circle can see it. That is a real reduction from what came
    /// before and it is deliberate: it is also exactly what this project has
    /// always claimed the second yes to be. A tripwire, not a lock. The
    /// holder could always have made a circle without one; what she cannot do
    /// is drop it quietly.
    #[serde(default)]
    pub requires_second_yes: bool,

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

    /// Which appointment the second signature was given under.
    ///
    /// **This is what lets the second person change without re-forming the
    /// circle.** Their key used to be in the circle's identity, which made it
    /// permanent: if they died, lost their device, or simply had to be
    /// replaced, the only way out was a new circle with everybody re-invited.
    /// For a record about somebody in declining health that is not an edge
    /// case, it is the expected course of events.
    ///
    /// So the circle's identity carries the *rule* — that two people must
    /// agree — and who the second person is becomes an entry the holder
    /// writes. This names which one was in force, so every peer can check the
    /// signature against the right key without having to know what the holder
    /// has done since. A fixed hash is a question with one answer everywhere,
    /// which is what validation needs and what "who is appointed right now"
    /// could never be.
    ///
    /// `None` means the holder admitted this person on her own signature.
    /// That is permitted, and it is **seen**: see the note on
    /// `requires_second_yes`, which explains why this is a tripwire rather
    /// than a lock, and what that does and does not buy.
    #[serde(default)]
    pub appointment: Option<ActionHash>,
}

/// How this circle decides who belongs.
pub enum Membrane {
    /// A real circle, closed around one person.
    ///
    /// The flag says whether this circle asks two people to agree. Who
    /// the second person is lives in the circle, not here, so that it can
    /// change without the circle having to be made again.
    Founder(AgentPubKey, bool),
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

    /*
     * A circle asks two people to agree if it says so, or if it names one.
     *
     * `seconder` is the older way of saying it, when the person was written
     * into the circle's identity and could never be changed. It is still read
     * so that a circle made that way still asks for two agreements — and, as
     * before, a named seconder that cannot be read closes the circle rather
     * than being quietly ignored. Absence of configuration must never mean
     * absence of a membrane, and neither must a typo in it.
     *
     * New circles set the flag and name nobody here. Who agrees is an entry
     * they write, and can write again.
     */
    let named_in_identity = match p.seconder.as_deref() {
        None => false,
        Some(text) => match AgentPubKey::try_from(text.trim()) {
            Ok(_) => true,
            Err(_) => return Ok(Membrane::Misconfigured),
        },
    };
    let asks_two = p.requires_second_yes || named_in_identity;

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
            Ok(key) => Ok(Membrane::Founder(key, asks_two)),
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

/// The membrane, checked as far as the caller is allowed to look.
///
/// `may_read_the_circle` is false during `genesis_self_check`, which runs on
/// the joiner's own machine before they have joined anything and therefore has
/// no network to ask. It is true in `validate`, which runs on peers who are
/// already here and can.
///
/// This split is not a workaround; it is what the two callbacks are for. The
/// local pass catches a damaged or plainly wrong invitation immediately, with
/// a sentence somebody can act on. The real gate is the network's, and it
/// checks everything.
fn check_membrane_as_far_as(
    agent: &AgentPubKey,
    membrane_proof: &Option<MembraneProof>,
    may_read_the_circle: bool,
) -> ExternResult<ValidateCallbackResult> {
    let (founder, asks_two) = match membrane()? {
        Membrane::Founder(key, asks_two) => (key, asks_two),
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
    if !verify_signature(founder.clone(), invitation.signature, agent.clone())? {
        return invalid("Invitation was not issued by the person whose circle this is");
    }

    /*
     * The second agreement, checked against the appointment it was given
     * under.
     *
     * Who agrees is no longer part of the circle's identity, so this cannot be
     * read off the properties any more. The invitation names which appointment
     * it relies on, and that is a fixed hash — a question with the same answer
     * on every machine forever, which is what validation requires and what
     * "who is appointed right now" could never be.
     *
     * An invitation naming no appointment carries only the holder's signature,
     * and is accepted. That is the tripwire and not a lock: see
     * `requires_second_yes`, which sets out exactly what that buys and what it
     * does not. `asks_two` is therefore not consulted here — it shapes what
     * the app offers and what the circle can see, not what the door refuses.
     */
    let _ = asks_two;

    if let Some(appointment_hash) = invitation.appointment {
        /*
         * As far as this can be taken without the circle to ask.
         *
         * Before joining there is no network, so the appointment cannot be
         * fetched and the signature cannot be checked against it. What can be
         * said is that an invitation naming an appointment and carrying no
         * second agreement is unfinished — which is the common case worth
         * catching early, and it costs the person a readable sentence rather
         * than a silent refusal from strangers later.
         */
        if !may_read_the_circle {
            let Some(seconded) = &invitation.seconded else {
                return invalid(
                    "This invitation is not finished. This circle asks two people \
                     to agree before anybody joins, and only one of them has.",
                );
            };

            /*
             * One forgery *can* be caught here, without knowing who was
             * appointed: the holder signing twice.
             *
             * Whoever is appointed, it is never her — an appointment naming the
             * holder is refused when it is written. So a second agreement that
             * verifies against her own key is a forgery no matter what the
             * appointment says, and that can be settled with nothing but the
             * founder key, which is in the circle's identity and needs no
             * network to read.
             *
             * It matters because it is the attack this whole feature exists
             * for: the holder under pressure, waiving her own safeguard. Left
             * to the network it would still be refused, but only after the
             * person had apparently joined — and a door that opens and then
             * quietly stops working is worse than one that says no.
             */
            if verify_signature(founder.clone(), seconded.clone(), agent.clone())? {
                return invalid(
                    "The second agreement on this invitation is from the person \
                     whose circle it is. It has to be somebody else — that is the \
                     whole of what it is for.",
                );
            }

            return Ok(ValidateCallbackResult::Valid);
        }

        let action = must_get_action(appointment_hash)?;
        let Some(entry_hash) = action.action().entry_hash() else {
            return invalid("This invitation names something that is not an appointment");
        };
        let entry = must_get_entry(entry_hash.clone())?;
        let Ok(appointment) = Appointment::try_from(entry.content.clone()) else {
            return invalid("This invitation names something that is not an appointment");
        };

        // Only the holder appoints. Without this, anybody could write an
        // appointment naming themselves and second their own way in.
        if action.action().author() != &founder {
            return invalid(
                "The appointment this invitation relies on was not made by the \
                 person whose circle this is",
            );
        }

        /*
         * The appointed person needs no second agreement to their own
         * admission.
         *
         * Asking them to countersign their own way in adds nothing anybody
         * could check, and it removes a knot that cannot otherwise be untied:
         * the one person who must agree to every arrival cannot get in
         * without agreeing to themselves from outside a circle they are not
         * in yet.
         */
        if agent == &appointment.agrees {
            return Ok(ValidateCallbackResult::Valid);
        }

        let Some(seconded) = invitation.seconded else {
            return invalid(
                "This circle asks two people to agree before anybody joins, and \
                 this invitation only has one of them",
            );
        };
        if !verify_signature(appointment.agrees, seconded, agent.clone())? {
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
    // No network here, so as far as it can be taken alone.
    check_membrane_as_far_as(&data.agent_key, &data.membrane_proof, false)
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
            // The entry itself names the appointment, and the entry rule has
            // already checked it. Here the link is only tied to its own
            // author, so nobody can file somebody else's agreement.
            let Some(target) = as_action_hash(&action.target_address) else {
                return invalid("An agreement link must point at an action");
            };
            let target_action = must_get_action(target)?;
            if target_action.action().author() != author {
                return invalid("You may only link your own agreement");
            }
            Ok(ValidateCallbackResult::Valid)
        }

        // Only the holder puts somebody forward to agree, and only her own.
        LinkTypes::CircleToAppointment => {
            if !is_the_person(author)? {
                return invalid(
                    "Only the person whose circle this is may ask somebody to agree",
                );
            }
            // Path anchor scaffolding. See the note under CircleToAboutMe.
            let Some(target) = as_action_hash(&action.target_address) else {
                return Ok(ValidateCallbackResult::Valid);
            };
            let target_action = must_get_action(target)?;
            if target_action.action().author() != author {
                return invalid("You may only record your own appointment");
            }
            Ok(ValidateCallbackResult::Valid)
        }

        // Only the person asked answers, and only their own answer.
        //
        // The entry rule has already checked that the author is the one the
        // appointment names. This ties the link to its own author too, so
        // nobody can file somebody else's answer where the holder reads it.
        LinkTypes::AppointmentToConsent => {
            let Some(target) = as_action_hash(&action.target_address) else {
                return invalid("An answer link must point at an action");
            };
            let target_action = must_get_action(target)?;
            if target_action.action().author() != author {
                return invalid("You may only link your own answer");
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
/// What is left to check once a knock is sealed.
///
/// "Say what you are called" used to be enforced here, by every peer. It
/// cannot be any more: the name is boxed to the holder, and a peer that
/// cannot read a thing cannot have an opinion about it. That is not a
/// weakness of this design so much as what encryption on a public DHT
/// means, and the HDK says as much — encrypted data cannot be validated.
///
/// The check moved to the app, where it is a courtesy rather than a rule.
/// Nothing was lost by that: an empty name was never dangerous, only
/// useless, and the person it inconveniences is the one who wrote it.
fn validate_knock(_knock: &Knock) -> ExternResult<ValidateCallbackResult> {
    if who_this_room_serves()?.is_none() {
        return invalid("Knocking only means something in a circle's waiting room");
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

/// Is this agent the one a particular appointment asks to agree?
///
/// The appointment has to be named rather than looked up, for the reason that
/// runs through all of this: who is appointed can change, and validation must
/// reach the same answer on every machine forever. A fixed hash is a question
/// with one answer. "Who is appointed now" is not, and never can be.
fn is_appointed_by(agent: &AgentPubKey, appointment: &ActionHash) -> ExternResult<bool> {
    let founder = match membrane()? {
        Membrane::Founder(key, _) => key,
        // A lobby, a waiting room and a broken circle all appoint nobody.
        _ => return Ok(false),
    };

    let action = must_get_action(appointment.clone())?;

    // Only the holder appoints. Without this, anybody could write an
    // appointment naming themselves and then agree to their own arrivals.
    if action.action().author() != &founder {
        return Ok(false);
    }

    let Some(entry_hash) = action.action().entry_hash() else {
        return Ok(false);
    };
    let entry = must_get_entry(entry_hash.clone())?;
    let Ok(appointed) = Appointment::try_from(entry.content.clone()) else {
        return Ok(false);
    };

    Ok(&appointed.agrees == agent)
}

/// Only the person an appointment names may answer it.
///
/// Without this anybody in the circle could write "yes, she is willing" on
/// somebody else's behalf, and the holder's screen would say the safeguard
/// was in place when the person holding it had never heard of it.
fn validate_consent(
    consent: &Consent,
    author: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    if !is_appointed_by(author, &consent.appointment)? {
        return invalid("Only the person who was asked may answer");
    }
    Ok(ValidateCallbackResult::Valid)
}

/// Only the holder may appoint somebody, and never herself.
fn validate_appointment(
    appointment: &Appointment,
    author: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    if !is_the_person(author)? {
        return invalid("Only the person whose circle this is may ask somebody to agree");
    }
    // A safeguard the person under pressure can satisfy alone is not one.
    if &appointment.agrees == author {
        return invalid("The person who agrees to who joins has to be somebody else");
    }
    Ok(ValidateCallbackResult::Valid)
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
    if !is_appointed_by(author, &endorsement.appointment)? {
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
        }) => check_membrane_as_far_as(action.author(), &membrane_proof, true),

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
            EntryTypes::Appointment(a) => validate_appointment(&a, action.author()),
            EntryTypes::Consent(c) => validate_consent(&c, action.author()),
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
            // Appoint somebody else instead. Rewriting who was trusted, and
            // when, would take away the only thing this safeguard now rests
            // on, which is that everybody can see it.
            EntryTypes::Appointment(_) => {
                invalid("An appointment cannot be changed; appoint somebody else")
            }
            // Change your mind by answering again. What you said before stays
            // said: the holder may have acted on it.
            EntryTypes::Consent(_) => {
                invalid("An answer cannot be changed; answer again")
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
