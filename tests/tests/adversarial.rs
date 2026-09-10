//! Adversarial tests.
//!
//! Every rule this project claims is a rule only if something rejects when it
//! is broken. These tests exist to break them.
//!
//! Prompted by an external red-team review which found that membership had
//! been implemented far more strongly than authorship, plus two holes it
//! missed: link creation and deletes were entirely unvalidated.

use aboutme_integrity::{AboutMe, CircleProperties, Invitation};
use holochain::prelude::*;
use holochain::sweettest::*;
use std::collections::HashMap;
use std::path::PathBuf;

const ZOME: &str = "aboutme";

fn dna_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("dnas/aboutme/workdir/aboutme.dna")
}

/// All seven sections of About Me, so a test that round-trips a record
/// round-trips the whole of one.
fn an_about_me(name: &str) -> AboutMe {
    AboutMe {
        display_name: name.to_string(),
        what_matters_to_me: "Seeing my grandchildren".into(),
        people_who_matter: "My daughter Ruth".into(),
        how_to_communicate_with_me: "Speak to my left side, I'm deaf on the right".into(),
        my_wellness: "I am not myself when I stop reading".into(),
        please_do_and_please_do_not: "Please do not move my chair".into(),
        how_to_support_me: "Give me time to answer".into(),
        also_worth_knowing: "I was a district nurse for thirty years".into(),
        supported_to_write_this_by: "My daughter Ruth".into(),
    }
}

/// A circle whose founder is `founder`, built from the real packed DNA.
async fn circle_dna(founder: &AgentPubKey) -> DnaFile {
    circle_dna_with_seconder(founder, None).await
}

/// The same, for a circle that asks two people to agree before anybody joins.
async fn circle_dna_with_seconder(
    founder: &AgentPubKey,
    seconder: Option<&AgentPubKey>,
) -> DnaFile {
    let properties = CircleProperties {
        founder: Some(founder.to_string()),
        lobby: false,
        seconder: seconder.map(|k| k.to_string()),
    };
    SweetDnaFile::from_bundle_with_overrides(
        &dna_path(),
        DnaModifiersOpt::none().with_properties(properties),
    )
    .await
    .expect("the packed DNA should load; run `hc dna pack` first")
}

/// Install the circle for one agent, optionally presenting an invitation.
///
/// Returns `Err` when the membrane rejects them, which is the point of several
/// of these tests.
async fn join(
    conductor: &SweetConductor,
    app_id: &str,
    agent: &AgentPubKey,
    dna: &DnaFile,
    invitation: Option<&Invitation>,
) -> anyhow::Result<CellId> {
    let bundle = app_bundle_from_dnas(&[("circle".to_string(), dna.clone())], false, None).await;

    let membrane_proof = invitation
        .map(|i| SerializedBytes::try_from(i.clone()).map(MembraneProof::new))
        .transpose()?;

    let roles = HashMap::from([(
        "circle".to_string(),
        RoleSettings::Provisioned {
            membrane_proof,
            modifiers: None,
            init_properties: None,
        },
    )]);

    let app = conductor
        .raw_handle()
        .install_app_bundle(InstallAppPayload {
            source: AppBundleSource::Bytes(bundle.pack()?.into()),
            agent_key: Some(agent.clone()),
            installed_app_id: Some(app_id.to_string()),
            roles_settings: Some(roles),
            network_seed: None,
            ignore_genesis_failure: false,
            restore_from_dht: false,
        })
        .await?;

    conductor
        .raw_handle()
        .enable_app(app_id.to_string())
        .await?;

    let cell_id = app
        .provisioned_cells()
        .next()
        .map(|(_, cell_id)| cell_id)
        .ok_or_else(|| anyhow::anyhow!("no provisioned cell"))?;

    Ok(cell_id)
}

fn zome(cell_id: &CellId) -> SweetZome {
    SweetZome::new(cell_id.clone(), ZOME.into())
}

/// A hash that refers to nothing, for tests about a message rather than about
/// what the message points at. A signal is never looked up — it is a nudge, and
/// the tests below are about who it claims to be from.
fn fake_hash() -> ActionHash {
    ActionHash::from_raw_36(vec![0; 36])
}

/// Alice founds a circle and Bob joins it with a genuine invitation.
async fn a_circle_with_a_member() -> (SweetConductor, CellId, CellId) {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let bob = SweetAgents::one(conductor.keystore()).await;
    let dna = circle_dna(&alice).await;

    let alice_cell = join(&conductor, "alice", &alice, &dna, None)
        .await
        .expect("the founder needs no invitation to her own circle");

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    let bob_cell = join(&conductor, "bob", &bob, &dna, Some(&bundle.invitation))
        .await
        .expect("an invited agent should be admitted");

    (conductor, alice_cell, bob_cell)
}

// ---------------------------------------------------------------------------
// The membrane: who may enter
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn uninvited_agent_cannot_join() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let mallory = SweetAgents::one(conductor.keystore()).await;
    let dna = circle_dna(&alice).await;

    join(&conductor, "alice", &alice, &dna, None).await.unwrap();

    let result = join(&conductor, "mallory", &mallory, &dna, None).await;
    assert!(
        result.is_err(),
        "an agent with no invitation must not be able to join"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn an_invitation_cannot_be_passed_on() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let bob = SweetAgents::one(conductor.keystore()).await;
    let dave = SweetAgents::one(conductor.keystore()).await;
    let dna = circle_dna(&alice).await;

    let alice_cell = join(&conductor, "alice", &alice, &dna, None).await.unwrap();

    // Alice invites Bob. Bob hands his invitation to Dave.
    let for_bob: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    let result = join(&conductor, "dave", &dave, &dna, Some(&for_bob.invitation)).await;
    assert!(
        result.is_err(),
        "an invitation is signed over the invitee's own key and must not transfer"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_forge_an_invitation() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let bob = SweetAgents::one(conductor.keystore()).await;
    let mallory = SweetAgents::one(conductor.keystore()).await;
    let dna = circle_dna(&alice).await;

    let alice_cell = join(&conductor, "alice", &alice, &dna, None).await.unwrap();
    let for_bob: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;
    let bob_cell = join(&conductor, "bob", &bob, &dna, Some(&for_bob.invitation))
        .await
        .unwrap();

    // Bob can call invite() — there is no permission check on it — but his
    // signature is not the founder's.
    let forged: aboutme::InvitationBundle = conductor
        .call(
            &zome(&bob_cell),
            "invite",
            aboutme::InviteInput {
                invitee: mallory.to_string(),
                name: String::new(),
            },
        )
        .await;

    let result = join(
        &conductor,
        "mallory",
        &mallory,
        &dna,
        Some(&forged.invitation),
    )
    .await;
    assert!(
        result.is_err(),
        "only the founder's signature admits anyone"
    );
}

// ---------------------------------------------------------------------------
// Authorship: who may speak as the person
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn the_person_can_write_their_own_about_me() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;

    let record: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;

    assert!(record.action().entry_hash().is_some());
}

/// The fault that produced four separate bugs on four screens.
///
/// Every list is fetched from the network, which is right for other people
/// and wrong for me: for the first seconds after I write something the
/// network has not heard of it. The screens said so — a "Who are you?" form
/// on the page just filled in, "Nothing has been written yet" above "Read
/// this over".
///
/// These read immediately after writing, with no wait and no polling,
/// because that is the moment the person is looking at the screen.
#[tokio::test(flavor = "multi_thread")]
async fn what_i_just_wrote_is_there_the_instant_i_look() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    // Bob introduces himself and must be in the list at once.
    let _: Record = conductor
        .call(
            &zome(&bob_cell),
            "introduce_myself",
            aboutme_integrity::Member {
                name: "Gareth".to_string(),
                relationship: "her son".to_string(),
            },
        )
        .await;

    let members: Vec<Record> = conductor.call(&zome(&bob_cell), "get_members", ()).await;
    assert!(
        !members.is_empty(),
        "a person who has just said who they are must not be asked again"
    );

    // Alice writes her record and must find it at once.
    let created: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;
    let original = created.action_address().clone();

    let originals: Vec<ActionHash> = conductor
        .call(&zome(&alice_cell), "get_circle_about_me", ())
        .await;
    assert!(
        originals.contains(&original),
        "\"Nothing has been written yet\" about words just typed in"
    );

    // And a correction must be what she sees, not the version before it.
    let mut corrected = an_about_me("Alice Bell");
    corrected.what_matters_to_me = "Seeing my grandchildren on Sundays".into();
    let updated: Record = conductor
        .call(
            &zome(&alice_cell),
            "update_about_me",
            aboutme::UpdateAboutMeInput {
                original_action_hash: original.clone(),
                previous_action_hash: original.clone(),
                about_me: corrected,
            },
        )
        .await;

    let current: aboutme::CurrentAboutMe = conductor
        .call(&zome(&alice_cell), "get_current_about_me", original)
        .await;
    assert_eq!(
        current.record.as_ref().map(|r| r.action_address().clone()),
        Some(updated.action_address().clone()),
        "a correction must be the version she is looking at"
    );

    // Bob offers something and must see that it was written down.
    let suggestion: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    let offered: Vec<aboutme::SuggestionWithOutcome> = conductor
        .call(&zome(&bob_cell), "get_suggestions", ())
        .await;
    assert!(
        offered
            .iter()
            .any(|s| s.suggestion.action_address() == suggestion.action_address()),
        "somebody who offers something must be able to see that it was taken"
    );

    // And Alice's decision must stick, or she will decide it twice.
    let _: Record = conductor
        .call(
            &zome(&alice_cell),
            "decide_on_suggestion",
            aboutme::DecideInput {
                suggestion: suggestion.action_address().clone(),
                accepted: true,
            },
        )
        .await;

    let decided: Vec<aboutme::SuggestionWithOutcome> = conductor
        .call(&zome(&alice_cell), "get_suggestions", ())
        .await;
    assert!(
        decided
            .iter()
            .find(|s| s.suggestion.action_address() == suggestion.action_address())
            .is_some_and(|s| s.outcome.is_some()),
        "a suggestion just decided must not still look undecided"
    );
}

/// Correcting your own record is not a disagreement with yourself.
///
/// This shipped, and it was found by looking at the screen rather than by any
/// test: writing a record and then editing it once announced "This was changed
/// in 2 places while devices were apart." The count was of every version in
/// the chain, which is two after any ordinary edit. What matters is how many
/// loose ends the chain has, and a chain edited in order has exactly one.
#[tokio::test(flavor = "multi_thread")]
async fn editing_your_own_record_in_order_is_not_a_disagreement() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;

    let created: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;
    let original = created.action_address().clone();

    // Two corrections, one after the other, each on top of the last — which is
    // what a person sitting at one machine actually does.
    let mut head = original.clone();
    for matters in [
        "Seeing my grandchildren",
        "Seeing my grandchildren on Sundays",
    ] {
        let mut edited = an_about_me("Alice Bell");
        edited.what_matters_to_me = matters.into();

        let updated: Record = conductor
            .call(
                &zome(&alice_cell),
                "update_about_me",
                aboutme::UpdateAboutMeInput {
                    original_action_hash: original.clone(),
                    previous_action_hash: head.clone(),
                    about_me: edited,
                },
            )
            .await;
        head = updated.action_address().clone();
    }

    let current: aboutme::CurrentAboutMe = conductor
        .call(&zome(&alice_cell), "get_current_about_me", original)
        .await;

    assert_eq!(
        current.divergent_versions, 1,
        "three versions in a row are one account of a person, not three competing ones"
    );
}

/// And the case the warning exists for, which must still be caught.
#[tokio::test(flavor = "multi_thread")]
async fn two_edits_from_the_same_starting_point_do_disagree() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;

    let created: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;
    let original = created.action_address().clone();

    // Both updates name the original as what they replace: neither knew about
    // the other. This is what two devices apart produce.
    for matters in ["At home if possible", "With her sister"] {
        let mut edited = an_about_me("Alice Bell");
        edited.what_matters_to_me = matters.into();

        let _: Record = conductor
            .call(
                &zome(&alice_cell),
                "update_about_me",
                aboutme::UpdateAboutMeInput {
                    original_action_hash: original.clone(),
                    previous_action_hash: original.clone(),
                    about_me: edited,
                },
            )
            .await;
    }

    let current: aboutme::CurrentAboutMe = conductor
        .call(&zome(&alice_cell), "get_current_about_me", original)
        .await;

    assert_eq!(
        current.divergent_versions, 2,
        "two versions written from the same starting point are a real fork, and          somebody should be told rather than shown one of them silently"
    );
}

// ---------------------------------------------------------------------------
// A second yes: circles that ask two people to agree
// ---------------------------------------------------------------------------
//
// The safeguarding case is not somebody breaking in. It is the holder being
// talked into letting a person in — a plausible caller, a new "friend", a
// relative nobody trusts. The holder is the one under that pressure, so a rule
// the holder can waive alone is not a safeguard at all.
//
// Compare RIX Multi Me's "Buddy", who can veto a share. This is the other way
// round and stronger: nothing happens unless the second person actively
// agrees. A veto has to arrive in time to stop something already moving; a
// signature that was never given stops nothing, because nothing started.

/// Everything a circle with a second yes needs: the founder, the person who
/// must also agree, somebody to invite, and a lobby for the seconder to sign
/// in — which is how the real app does it, and why the seconder never has to
/// be a member of the circle at all.
async fn a_circle_that_asks_two_people() -> (
    SweetConductor,
    CellId, // the founder's cell
    CellId, // the seconder's lobby cell
    DnaFile,
    AgentPubKey, // somebody waiting to be invited
) {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let ruth = SweetAgents::one(conductor.keystore()).await;
    let bob = SweetAgents::one(conductor.keystore()).await;

    let dna = circle_dna_with_seconder(&alice, Some(&ruth)).await;
    let alice_cell = join(&conductor, "alice", &alice, &dna, None)
        .await
        .expect("the founder needs no invitation to her own circle");

    let lobby = lobby_dna().await;
    let ruth_lobby = join(&conductor, "ruth-lobby", &ruth, &lobby, None)
        .await
        .expect("anyone may enter the lobby");

    (conductor, alice_cell, ruth_lobby, dna, bob)
}

/// One signature is not enough where the circle asks for two.
#[tokio::test(flavor = "multi_thread")]
async fn half_an_invitation_opens_nothing() {
    let (conductor, alice_cell, _ruth_lobby, dna, bob) = a_circle_that_asks_two_people().await;

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    assert!(
        bundle.invitation.seconded.is_none(),
        "inviting somebody leaves the holder with only her own signature on it"
    );
    assert!(
        bundle.seconder.is_some(),
        "and the invitation says who else must agree, because the joiner needs \
         that to compute the same circle at all"
    );

    let refused = join(&conductor, "bob", &bob, &dna, Some(&bundle.invitation)).await;
    assert!(
        refused.is_err(),
        "the holder's signature alone must not open a circle that asks for two"
    );
}

/// Two signatures do.
#[tokio::test(flavor = "multi_thread")]
async fn two_people_agreeing_lets_somebody_in() {
    let (conductor, alice_cell, ruth_lobby, dna, bob) = a_circle_that_asks_two_people().await;

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    // Ruth signs from the lobby, never having joined the circle: she is
    // agreeing to who gets in without being able to read a word of it.
    let seconded: Signature = conductor
        .call(&zome(&ruth_lobby), "second_an_invitation", bob.to_string())
        .await;

    let invitation = Invitation {
        signature: bundle.invitation.signature.clone(),
        seconded: Some(seconded),
    };

    assert!(
        join(&conductor, "bob", &bob, &dna, Some(&invitation))
            .await
            .is_ok(),
        "with both agreements the door opens"
    );
}

/// The person who must agree does not have to agree to themselves.
///
/// Without this the feature eats its own tail: the one person who has to
/// approve every arrival cannot arrive, because approving their own way in
/// would mean doing it from outside a circle they are not in yet. They come in
/// on the holder's invitation alone, and from then on nobody else does.
#[tokio::test(flavor = "multi_thread")]
async fn the_second_yes_needs_no_second_yes_of_their_own() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let ruth = SweetAgents::one(conductor.keystore()).await;

    let dna = circle_dna_with_seconder(&alice, Some(&ruth)).await;
    let alice_cell = join(&conductor, "alice", &alice, &dna, None)
        .await
        .expect("the founder needs no invitation to her own circle");

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: ruth.to_string(),
                name: String::new(),
            },
        )
        .await;

    assert!(
        bundle.invitation.seconded.is_none(),
        "an ordinary invitation, with only the holder's signature on it"
    );

    assert!(
        join(&conductor, "ruth", &ruth, &dna, Some(&bundle.invitation))
            .await
            .is_ok(),
        "the person the circle asks must be able to get into it"
    );
}

/// The holder cannot be both people. That is the entire point.
#[tokio::test(flavor = "multi_thread")]
async fn the_holder_cannot_give_the_second_yes_herself() {
    let (conductor, alice_cell, _ruth_lobby, dna, bob) = a_circle_that_asks_two_people().await;

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    // Alice signs a second time, from her own cell, trying to be both
    // signatures. This is the attack the feature exists for: the holder under
    // pressure, waiving her own safeguard.
    let forged: Signature = conductor
        .call(&zome(&alice_cell), "second_an_invitation", bob.to_string())
        .await;

    let invitation = Invitation {
        signature: bundle.invitation.signature.clone(),
        seconded: Some(forged),
    };

    assert!(
        join(&conductor, "bob", &bob, &dna, Some(&invitation))
            .await
            .is_err(),
        "a safeguard the holder can waive alone is not a safeguard"
    );
}

/// A circle that asks nobody carries on working exactly as it did.
#[tokio::test(flavor = "multi_thread")]
async fn a_circle_with_no_second_yes_is_unchanged() {
    let (conductor, _alice_cell, bob_cell) = a_circle_with_a_member().await;

    let seconder: Option<AgentPubKey> = conductor
        .call(&zome(&bob_cell), "who_seconds_here", ())
        .await;

    assert!(
        seconder.is_none(),
        "most circles ask one person, and must not start asking two by accident"
    );
}

/// The central red-team finding.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_write_the_persons_about_me() {
    let (conductor, _, bob_cell) = a_circle_with_a_member().await;

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&bob_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;

    assert!(
        result.is_err(),
        "membership must not confer the right to author somebody else's account of themselves"
    );
}

/// The hole the review missed: link creation was unvalidated, so a member could
/// point the person's own update chain at an entry of their own. Readers follow
/// that chain, so the person's record would show the impostor's content without
/// the person's entry ever being touched.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_hijack_the_update_chain() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let alice_record: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;
    let alice_original = alice_record.action_address().clone();

    // Bob cannot even create an About Me to point at any more, so the hijack
    // fails at the first step. Belt and braces: try to update Alice's record
    // directly as well.
    let forged: Result<Record, _> = conductor
        .call_fallible(
            &zome(&bob_cell),
            "update_about_me",
            aboutme::UpdateAboutMeInput {
                original_action_hash: alice_original.clone(),
                previous_action_hash: alice_original,
                about_me: an_about_me("Alice Bell"),
            },
        )
        .await;

    assert!(
        forged.is_err(),
        "a member must not be able to extend the person's update chain"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_delete_the_persons_record() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let alice_record: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;

    let result: Result<ActionHash, _> = conductor
        .call_fallible(
            &zome(&bob_cell),
            "delete_about_me",
            alice_record.action_address().clone(),
        )
        .await;

    assert!(result.is_err(), "only the author of a record may delete it");
}

// ---------------------------------------------------------------------------
// Acknowledgements
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn a_member_can_acknowledge_the_record() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let alice_record: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;

    let _ack: Record = conductor
        .call(
            &zome(&bob_cell),
            "acknowledge",
            aboutme::AcknowledgeInput {
                about_me: alice_record.action_address().clone(),
                role: "district nurse".to_string(),
            },
        )
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn nobody_can_acknowledge_their_own_record() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;

    let alice_record: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&alice_cell),
            "acknowledge",
            aboutme::AcknowledgeInput {
                about_me: alice_record.action_address().clone(),
                role: "myself".to_string(),
            },
        )
        .await;

    assert!(
        result.is_err(),
        "a self-acknowledgement would make the evidence worthless"
    );
}

// ---------------------------------------------------------------------------
// Configuration must fail closed
// ---------------------------------------------------------------------------

/// The most dangerous thing that could go wrong quietly: a circle shipped with
/// no founder configured, standing wide open.
///
/// Absence of configuration must never mean absence of a membrane.
#[tokio::test(flavor = "multi_thread")]
async fn a_circle_with_no_founder_admits_nobody() {
    let conductor = SweetConductor::standard().await;
    let mallory = SweetAgents::one(conductor.keystore()).await;

    let dna = SweetDnaFile::from_bundle_with_overrides(
        &dna_path(),
        DnaModifiersOpt::none().with_properties(CircleProperties {
            founder: None,
            seconder: None,
            lobby: false,
        }),
    )
    .await
    .unwrap();

    let result = join(&conductor, "mallory", &mallory, &dna, None).await;
    assert!(
        result.is_err(),
        "a circle with no founder must be closed to everybody, not open to everybody"
    );
}

/// A founder that is present but not a valid key is a misconfiguration, and
/// must be treated as one rather than falling through to an open circle.
#[tokio::test(flavor = "multi_thread")]
async fn a_circle_with_a_malformed_founder_admits_nobody() {
    let conductor = SweetConductor::standard().await;
    let mallory = SweetAgents::one(conductor.keystore()).await;

    let dna = SweetDnaFile::from_bundle_with_overrides(
        &dna_path(),
        DnaModifiersOpt::none().with_properties(CircleProperties {
            founder: Some("not-an-agent-key".to_string()),
            seconder: None,
            lobby: false,
        }),
    )
    .await
    .unwrap();

    let result = join(&conductor, "mallory", &mallory, &dna, None).await;
    assert!(
        result.is_err(),
        "a malformed founder must close the circle, not open it"
    );
}

// ---------------------------------------------------------------------------
// Entry content
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn an_about_me_needs_a_display_name() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;

    let mut blank = an_about_me("");
    blank.display_name = "   ".to_string();

    let result: Result<Record, _> = conductor
        .call_fallible(&zome(&alice_cell), "create_about_me", blank)
        .await;

    assert!(
        result.is_err(),
        "a record nobody is named in is not a record of anybody"
    );
}

// ---------------------------------------------------------------------------
// Circles are separate networks, not separate labels
// ---------------------------------------------------------------------------

/// A circle is a cloned cell whose properties name its holder. Because
/// properties form part of the DNA hash, two holders' circles are different
/// networks — which is what makes "one circle per person" a fact about the
/// maths rather than an access rule someone could get wrong.
#[tokio::test(flavor = "multi_thread")]
async fn two_holders_circles_are_different_networks() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let hers: ClonedCell = conductor
        .call(
            &zome(&alice_cell),
            "create_circle",
            aboutme::CreateCircleInput {
                founder: alice_cell.agent_pubkey().to_string(),
                name: "Alice".to_string(),
                network_seed: "shared-seed".to_string(),
                seconder: None,
            },
        )
        .await;

    // Same seed, same everything except who holds it.
    let his: ClonedCell = conductor
        .call(
            &zome(&bob_cell),
            "create_circle",
            aboutme::CreateCircleInput {
                founder: bob_cell.agent_pubkey().to_string(),
                name: "Bob".to_string(),
                network_seed: "shared-seed".to_string(),
                seconder: None,
            },
        )
        .await;

    assert_ne!(
        hers.cell_id.dna_hash(),
        his.cell_id.dna_hash(),
        "circles with different holders must be different networks, even with the same seed"
    );
    assert_ne!(
        hers.cell_id.dna_hash(),
        alice_cell.dna_hash(),
        "a circle must be its own network, not the cell it was cloned from"
    );
}

/// You cannot conjure a circle in somebody else's name.
///
/// Creating a circle means joining it, and joining runs the membrane check. An
/// agent who names someone else as holder, and has no invitation from them, is
/// refused at genesis.
///
/// **This is what settles the proxy question.** The holder is whoever will
/// administer the circle — a daughter, a case manager — not necessarily the
/// person the record describes. Who the record is *about* is content;
/// who holds it is the membrane.
#[tokio::test(flavor = "multi_thread")]
async fn nobody_can_create_a_circle_in_another_persons_name() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;
    let someone_else = SweetAgents::one(conductor.keystore()).await;

    let result: Result<ClonedCell, _> = conductor
        .call_fallible(
            &zome(&alice_cell),
            "create_circle",
            aboutme::CreateCircleInput {
                founder: someone_else.to_string(),
                name: "Not mine to make".to_string(),
                network_seed: "seed".to_string(),
                seconder: None,
            },
        )
        .await;

    assert!(
        result.is_err(),
        "naming someone else as holder must not let you into the circle you just made"
    );
}

/// The same person can hold more than one circle, kept apart by the seed.
#[tokio::test(flavor = "multi_thread")]
async fn one_person_can_have_separate_circles() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;
    let alice = alice_cell.agent_pubkey().clone();

    let mut hashes = Vec::new();
    for seed in ["care", "respite"] {
        let cell: ClonedCell = conductor
            .call(
                &zome(&alice_cell),
                "create_circle",
                aboutme::CreateCircleInput {
                    founder: alice.to_string(),
                    name: seed.to_string(),
                    network_seed: seed.to_string(),
                    seconder: None,
                },
            )
            .await;
        hashes.push(cell.cell_id.dna_hash().clone());
    }

    assert_ne!(
        hashes[0], hashes[1],
        "a different seed must give a different network"
    );
}

// ---------------------------------------------------------------------------
// The lobby
// ---------------------------------------------------------------------------

/// A lobby whose only job is to exist so the app can be installed and clone
/// real circles from it.
async fn lobby_dna() -> DnaFile {
    SweetDnaFile::from_bundle_with_overrides(
        &dna_path(),
        DnaModifiersOpt::none().with_properties(CircleProperties {
            founder: None,
            lobby: true,
            seconder: None,
        }),
    )
    .await
    .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn anyone_may_enter_the_lobby() {
    let conductor = SweetConductor::standard().await;
    let anyone = SweetAgents::one(conductor.keystore()).await;

    join(&conductor, "anyone", &anyone, &lobby_dna().await, None)
        .await
        .expect("the lobby exists to be installable by anyone");
}

/// Everybody who installs the app shares the lobby, so it must hold nothing.
/// Joining it is unrestricted precisely because there is nothing to reach.
#[tokio::test(flavor = "multi_thread")]
async fn nobody_may_write_in_the_lobby() {
    let conductor = SweetConductor::standard().await;
    let anyone = SweetAgents::one(conductor.keystore()).await;
    let cell = join(&conductor, "anyone", &anyone, &lobby_dna().await, None)
        .await
        .unwrap();

    let result: Result<Record, _> = conductor
        .call_fallible(&zome(&cell), "create_about_me", an_about_me("Anybody"))
        .await;

    assert!(
        result.is_err(),
        "a shared lobby that accepts writes is a shared database of strangers"
    );
}

/// The lobby's actual purpose: cloning a real circle out of it.
#[tokio::test(flavor = "multi_thread")]
async fn a_circle_can_be_cloned_from_the_lobby() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let lobby = join(&conductor, "alice", &alice, &lobby_dna().await, None)
        .await
        .unwrap();

    let circle: ClonedCell = conductor
        .call(
            &zome(&lobby),
            "create_circle",
            aboutme::CreateCircleInput {
                founder: alice.to_string(),
                name: "Alice".to_string(),
                network_seed: "seed".to_string(),
                seconder: None,
            },
        )
        .await;

    assert_ne!(
        circle.cell_id.dna_hash(),
        lobby.dna_hash(),
        "a circle must be its own network, not the lobby it came from"
    );

    // And unlike the lobby, the circle accepts the person's own record.
    let _: Record = conductor
        .call(
            &zome(&circle.cell_id),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;
}

// ---------------------------------------------------------------------------
// Signals: telling the holder without polling
// ---------------------------------------------------------------------------

/// The thing families currently have no way of knowing: whether anybody read
/// it. No polling, no server, no notification service — the professional's
/// She sends an invitation and then has no way of knowing whether it worked.
///
/// Everything arrived correctly and the screen said nothing, so the only way
/// to find out whether a nephew had joined was to ring him up. The list of
/// people is the record of who is here; this signal only saves her going to
/// look at it.
#[tokio::test(flavor = "multi_thread")]
async fn the_holder_is_told_when_her_invitation_is_taken_up() {
    let (conductor, _, bob_cell) = a_circle_with_a_member().await;

    let mut alice_hears = conductor.subscribe_to_app_signals("alice".to_string());

    let _: Record = conductor
        .call(
            &zome(&bob_cell),
            "introduce_myself",
            aboutme_integrity::Member {
                name: "Dave Smythe".to_string(),
                relationship: "her nephew".to_string(),
            },
        )
        .await;

    let signal = tokio::time::timeout(std::time::Duration::from_secs(60), alice_hears.recv())
        .await
        .expect("the person who sent the invitation should be told")
        .expect("the signal channel should stay open");

    match signal {
        Signal::App { signal, .. } => {
            let decoded: aboutme::Signal = signal
                .into_inner()
                .decode()
                .expect("the signal should be one of ours");
            let aboutme::Signal::Introduced {
                by,
                name,
                relationship,
                joined,
            } = decoded
            else {
                panic!("expected an Introduced signal, got {decoded:?}");
            };
            assert_eq!(by, *bob_cell.agent_pubkey());
            assert_eq!(name, "Dave Smythe");
            assert_eq!(relationship, "her nephew", "in his own words, unchecked");
            assert!(
                joined,
                "the first time somebody speaks up, they have joined"
            );
        }
        other => panic!("expected an app signal, got {other:?}"),
    }
}

/// And correcting yourself is not joining twice.
#[tokio::test(flavor = "multi_thread")]
async fn correcting_your_own_introduction_is_not_another_arrival() {
    let (conductor, _, bob_cell) = a_circle_with_a_member().await;

    let _: Record = conductor
        .call(
            &zome(&bob_cell),
            "introduce_myself",
            aboutme_integrity::Member {
                name: "Dave Smyth".to_string(),
                relationship: "her nephew".to_string(),
            },
        )
        .await;

    // Subscribed only now, so the first introduction cannot be mistaken for
    // the one under test.
    let mut alice_hears = conductor.subscribe_to_app_signals("alice".to_string());

    let _: Record = conductor
        .call(
            &zome(&bob_cell),
            "introduce_myself",
            aboutme_integrity::Member {
                name: "Dave Smythe".to_string(),
                relationship: "her nephew".to_string(),
            },
        )
        .await;

    let signal = tokio::time::timeout(std::time::Duration::from_secs(60), alice_hears.recv())
        .await
        .expect("she should still be told")
        .expect("the signal channel should stay open");

    match signal {
        Signal::App { signal, .. } => {
            let decoded: aboutme::Signal = signal
                .into_inner()
                .decode()
                .expect("the signal should be one of ours");
            let aboutme::Signal::Introduced { joined, name, .. } = decoded else {
                panic!("expected an Introduced signal, got {decoded:?}");
            };
            assert_eq!(name, "Dave Smythe");
            assert!(
                !joined,
                "somebody fixing the spelling of their own name has not arrived again"
            );
        }
        other => panic!("expected an app signal, got {other:?}"),
    }
}

/// device tells the holder's device directly.
#[tokio::test(flavor = "multi_thread")]
async fn the_holder_is_told_when_someone_reads_the_record() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let mut alice_hears = conductor.subscribe_to_app_signals("alice".to_string());

    let record: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;

    let _: Record = conductor
        .call(
            &zome(&bob_cell),
            "acknowledge",
            aboutme::AcknowledgeInput {
                about_me: record.action_address().clone(),
                role: "district nurse".to_string(),
            },
        )
        .await;

    let signal = tokio::time::timeout(std::time::Duration::from_secs(60), alice_hears.recv())
        .await
        .expect("the holder should be told within a reasonable time")
        .expect("the signal channel should stay open");

    match signal {
        Signal::App { signal, .. } => {
            let decoded: aboutme::Signal = signal
                .into_inner()
                .decode()
                .expect("the signal should be one of ours");
            let aboutme::Signal::Acknowledged { by, role, .. } = decoded else {
                panic!("expected an Acknowledged signal, got {decoded:?}");
            };
            assert_eq!(by, *bob_cell.agent_pubkey(), "the reader should be named");
            assert_eq!(role, "district nurse", "the claimed role travels with it");
        }
        other => panic!("expected an app signal, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Suggestions: everyone contributes, one voice remains
// ---------------------------------------------------------------------------

fn a_suggestion() -> aboutme_integrity::Suggestion {
    aboutme_integrity::Suggestion {
        field: aboutme_integrity::AboutMeField::WhatMattersToMe,
        text: "Her allotment. She talked about it all summer.".to_string(),
        because: "I am her son.".to_string(),
    }
}

/// The whole point. A member who may not write the record may still offer
/// something for it.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_can_offer_a_suggestion() {
    let (conductor, _, bob_cell) = a_circle_with_a_member().await;

    let record: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    assert!(record.action().entry_hash().is_some());
}

/// And the holder sees it.
#[tokio::test(flavor = "multi_thread")]
async fn the_holder_sees_what_was_offered() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let _: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    let offered: Vec<aboutme::SuggestionWithOutcome> = conductor
        .call(&zome(&alice_cell), "get_suggestions", ())
        .await;

    assert_eq!(offered.len(), 1, "the holder should see the suggestion");
    assert!(
        offered[0].outcome.is_none(),
        "an undecided suggestion has no outcome yet"
    );
}

/// A suggestion is an offer, not an edit. Only the holder decides.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_decide_on_a_suggestion() {
    let (conductor, _, bob_cell) = a_circle_with_a_member().await;

    let offered: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&bob_cell),
            "decide_on_suggestion",
            aboutme::DecideInput {
                suggestion: offered.action_address().clone(),
                accepted: true,
            },
        )
        .await;

    assert!(
        result.is_err(),
        "offering something must not be the same as putting it in the record"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_holder_can_decide_and_the_decision_is_kept() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let offered: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    let _: Record = conductor
        .call(
            &zome(&alice_cell),
            "decide_on_suggestion",
            aboutme::DecideInput {
                suggestion: offered.action_address().clone(),
                // Set aside rather than accepted, because that is the case
                // that must not disappear.
                accepted: false,
            },
        )
        .await;

    let after: Vec<aboutme::SuggestionWithOutcome> = conductor
        .call(&zome(&alice_cell), "get_suggestions", ())
        .await;

    assert_eq!(after.len(), 1, "a suggestion set aside is still there");
    assert!(
        after[0].outcome.is_some(),
        "and what became of it is recorded, not silently dropped"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_change_somebody_elses_suggestion() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let offered: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    let mut altered = a_suggestion();
    altered.text = "Something Bob never said.".to_string();

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&alice_cell),
            "update_suggestion",
            (offered.action_address().clone(), altered),
        )
        .await;

    assert!(
        result.is_err(),
        "nobody may put words in another member's mouth"
    );
}

// ---------------------------------------------------------------------------
// The whole journey
// ---------------------------------------------------------------------------

/// Everything a real circle does, in order.
///
/// The individual rules are tested above. This exists because the parts can
/// all be right while the journey is broken — which is exactly what happened
/// by hand: the record was written, its reference reached the joiner, and the
/// content did not follow. Nothing above would have caught that.
#[tokio::test(flavor = "multi_thread")]
async fn the_whole_journey() {
    let (conductor, alice, bob) = a_circle_with_a_member().await;

    // 1. The person's record.
    let written: Record = conductor
        .call(
            &zome(&alice),
            "create_about_me",
            an_about_me("Margaret Smythe"),
        )
        .await;
    let original = written.action_address().clone();

    // 2. It has to actually reach the other member. Gossip carries links and
    //    entries separately, so the reference can arrive without the content —
    //    poll for both, the way a person pressing "Check again" would.
    let mut arrived = None;
    for _ in 0..60 {
        let originals: Vec<ActionHash> =
            conductor.call(&zome(&bob), "get_circle_about_me", ()).await;

        if let Some(found) = originals.first() {
            let current: aboutme::CurrentAboutMe = conductor
                .call(&zome(&bob), "get_current_about_me", found.clone())
                .await;
            if current.record.is_some() {
                arrived = current.record;
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    assert!(
        arrived.is_some(),
        "the record must reach the other member — the reference and the content"
    );

    // 3. Bob says who he is. Nobody verifies it.
    let _: Record = conductor
        .call(
            &zome(&bob),
            "introduce_myself",
            aboutme_integrity::Member {
                name: "Gareth".to_string(),
                relationship: "her son".to_string(),
            },
        )
        .await;

    // 4. He remembers something she enjoyed. He cannot write it himself.
    let offered: Record = conductor.call(&zome(&bob), "suggest", a_suggestion()).await;

    // 5. The holder sees it and decides.
    let waiting: Vec<aboutme::SuggestionWithOutcome> =
        conductor.call(&zome(&alice), "get_suggestions", ()).await;
    assert_eq!(waiting.len(), 1, "the holder should see what was offered");

    let _: Record = conductor
        .call(
            &zome(&alice),
            "decide_on_suggestion",
            aboutme::DecideInput {
                suggestion: offered.action_address().clone(),
                accepted: true,
            },
        )
        .await;

    // 6. And Bob marks that he has read it: the whole professional workflow.
    let _: Record = conductor
        .call(
            &zome(&bob),
            "acknowledge",
            aboutme::AcknowledgeInput {
                about_me: original.clone(),
                role: "her son".to_string(),
            },
        )
        .await;

    let readers: Vec<Record> = conductor
        .call(&zome(&alice), "get_acknowledgements", original)
        .await;
    assert_eq!(readers.len(), 1, "the holder should know it was read");
}

// ---------------------------------------------------------------------------
// Signals: a nudge that cannot lie about who it is from
// ---------------------------------------------------------------------------
//
// A signal is not evidence. The acknowledgement written to the chain is the
// evidence, and a signal only saves somebody going to look.
//
// But the interface acts on one directly — it puts a sentence on the screen
// naming a person — so it has to be true. Every signal says who it is from, and
// that field is filled in by whoever sent it. Until it was checked against who
// actually called, any member of a circle could make the holder's screen say
// that a district nurse had read the record when nobody had.

/// Claiming to be somebody else gets you nothing.
#[tokio::test(flavor = "multi_thread")]
async fn a_signal_that_names_the_wrong_sender_is_dropped() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let mut bob_hears = conductor.subscribe_to_app_signals("bob".to_string());

    // Bob announces that Alice has offered something. She has not.
    let _: () = conductor
        .call(
            &zome(&bob_cell),
            "recv_remote_signal",
            aboutme::Signal::Suggested {
                suggestion: fake_hash(),
                by: alice_cell.agent_pubkey().clone(),
                text: "Words Alice never wrote.".to_string(),
            },
        )
        .await;

    let heard = tokio::time::timeout(std::time::Duration::from_secs(5), bob_hears.recv()).await;

    assert!(
        heard.is_err(),
        "a signal claiming to be from somebody else must reach no screen"
    );
}

/// And the honest case still works, so the check above is not simply a wall.
#[tokio::test(flavor = "multi_thread")]
async fn a_signal_from_the_person_it_names_is_delivered() {
    let (conductor, _alice_cell, bob_cell) = a_circle_with_a_member().await;

    let mut bob_hears = conductor.subscribe_to_app_signals("bob".to_string());

    let _: () = conductor
        .call(
            &zome(&bob_cell),
            "recv_remote_signal",
            aboutme::Signal::Suggested {
                suggestion: fake_hash(),
                by: bob_cell.agent_pubkey().clone(),
                text: "Her allotment.".to_string(),
            },
        )
        .await;

    let signal = tokio::time::timeout(std::time::Duration::from_secs(60), bob_hears.recv())
        .await
        .expect("a signal that names its real sender should arrive")
        .expect("the signal channel should stay open");

    match signal {
        Signal::App { signal, .. } => {
            let decoded: aboutme::Signal = signal
                .into_inner()
                .decode()
                .expect("the signal should be one of ours");
            let aboutme::Signal::Suggested { by, .. } = decoded else {
                panic!("expected a Suggested signal, got {decoded:?}");
            };
            assert_eq!(by, *bob_cell.agent_pubkey());
        }
        other => panic!("expected an app signal, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Everybody must agree which one is the current one
// ---------------------------------------------------------------------------
//
// Lists here are assembled from links, and the order links arrive in is not
// promised to be the same on any two machines. Anywhere a reader takes the last
// of something as the one that counts, that order has to be imposed rather than
// inherited — see `order_versions` in the integrity crate, which says the same
// thing about versions of a record.

/// Correcting how you describe yourself, and having the correction be the one
/// that shows.
#[tokio::test(flavor = "multi_thread")]
async fn the_latest_introduction_is_the_one_that_counts() {
    let (conductor, _alice_cell, bob_cell) = a_circle_with_a_member().await;

    for name in ["Dave Smyth", "Dave Smythe"] {
        let _: Record = conductor
            .call(
                &zome(&bob_cell),
                "introduce_myself",
                aboutme_integrity::Member {
                    name: name.to_string(),
                    relationship: "her nephew".to_string(),
                },
            )
            .await;
    }

    let members: Vec<Record> = conductor.call(&zome(&bob_cell), "get_members", ()).await;

    // A reader folds these into a map keyed by author, so the last one wins.
    // Both of Bob's are here; the corrected spelling has to be the later.
    let mine: Vec<String> = members
        .iter()
        .filter(|r| r.action().author() == bob_cell.agent_pubkey())
        .filter_map(|r| {
            r.entry()
                .to_app_option::<aboutme_integrity::Member>()
                .ok()
                .flatten()
        })
        .map(|m| m.name)
        .collect();

    assert_eq!(
        mine,
        vec!["Dave Smyth".to_string(), "Dave Smythe".to_string()],
        "introductions must come back oldest first, so a correction is last"
    );
}

/// Changing your mind about a suggestion, and having the change be the one that
/// shows.
#[tokio::test(flavor = "multi_thread")]
async fn the_latest_decision_is_the_one_that_counts() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let offered: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    // Set aside, and then thought better of.
    for accepted in [false, true] {
        let _: Record = conductor
            .call(
                &zome(&alice_cell),
                "decide_on_suggestion",
                aboutme::DecideInput {
                    suggestion: offered.action_address().clone(),
                    accepted,
                },
            )
            .await;
    }

    let after: Vec<aboutme::SuggestionWithOutcome> = conductor
        .call(&zome(&alice_cell), "get_suggestions", ())
        .await;

    assert_eq!(after.len(), 1);
    let outcome = after[0]
        .outcome
        .as_ref()
        .expect("a decided suggestion has an outcome")
        .entry()
        .to_app_option::<aboutme_integrity::SuggestionOutcome>()
        .ok()
        .flatten()
        .expect("and it is an outcome");

    assert!(
        outcome.accepted,
        "the decision shown must be the last one made, not the first"
    );
}

// ---------------------------------------------------------------------------
// Configuration that fails closed, in the case that was added last
// ---------------------------------------------------------------------------

/// A circle naming a second person it cannot read admits nobody at all —
/// including the holder.
///
/// The equivalent for a malformed founder has a test above. This one was added
/// afterwards and did not get one, which is exactly how the two would drift
/// apart. Getting it wrong must close the door rather than open it: absence of
/// configuration must never mean absence of a membrane, and neither must a typo
/// in it.
#[tokio::test(flavor = "multi_thread")]
async fn a_circle_with_an_unreadable_second_person_admits_nobody() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;

    let dna = SweetDnaFile::from_bundle_with_overrides(
        &dna_path(),
        DnaModifiersOpt::none().with_properties(CircleProperties {
            founder: Some(alice.to_string()),
            lobby: false,
            seconder: Some("not an identifier at all".to_string()),
        }),
    )
    .await
    .expect("the packed DNA should load");

    let result = join(&conductor, "alice", &alice, &dna, None).await;

    assert!(
        result.is_err(),
        "a circle that names a second person it cannot read must close, \
         not quietly forget the second person"
    );
}

// ---------------------------------------------------------------------------
// An acknowledgement must be about a record
// ---------------------------------------------------------------------------

/// Reading something is only meaningful about an About Me.
///
/// The hash an acknowledgement points at comes from whoever is acknowledging,
/// so it is the one part of this they choose. Without the check, an
/// acknowledgement could be attached to anything at all and counted as evidence
/// that somebody had read the person's record.
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_cannot_point_at_something_that_is_not_a_record() {
    let (conductor, _alice_cell, bob_cell) = a_circle_with_a_member().await;

    // A real entry Bob wrote, which is simply not an About Me.
    let offered: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&bob_cell),
            "acknowledge",
            aboutme::AcknowledgeInput {
                about_me: offered.action_address().clone(),
                role: "district nurse".to_string(),
            },
        )
        .await;

    assert!(
        result.is_err(),
        "an acknowledgement must reference an About Me and nothing else"
    );
}

// ---------------------------------------------------------------------------
// The second person is told who they are agreeing to
// ---------------------------------------------------------------------------

/// A key identifies nobody, so the holder says who she thinks it is.
///
/// Without this the person being asked to agree sees a line of base64 and
/// nothing else, and the only thing they can actually agree to is that they
/// were asked — which safeguards nobody. It is her claim and nothing checks
/// it; what it gives the second person is something they can answer.
#[tokio::test(flavor = "multi_thread")]
async fn an_invitation_carries_who_the_holder_says_it_is_for() {
    let (conductor, alice_cell, _bob_cell) = a_circle_with_a_member().await;

    let ronnie = SweetAgents::one(conductor.keystore()).await;

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: ronnie.to_string(),
                name: "  Ronnie Smythe  ".to_string(),
            },
        )
        .await;

    assert_eq!(
        bundle.invitee_name, "Ronnie Smythe",
        "the name travels with the invitation, trimmed"
    );
    assert_eq!(
        bundle.invitee,
        ronnie.to_string(),
        "and it is attached to the key it was given for"
    );
}

/// An invitation with no name still works.
///
/// The name is a help, not a requirement. Refusing to make an invitation
/// without one would turn a courtesy into a gate, and there is nothing to
/// enforce because nothing is checked anyway.
#[tokio::test(flavor = "multi_thread")]
async fn an_invitation_without_a_name_is_still_an_invitation() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let bob = SweetAgents::one(conductor.keystore()).await;
    let dna = circle_dna(&alice).await;

    let alice_cell = join(&conductor, "alice", &alice, &dna, None).await.unwrap();

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    assert_eq!(bundle.invitee_name, "");

    join(&conductor, "bob", &bob, &dna, Some(&bundle.invitation))
        .await
        .expect("a nameless invitation still admits the person it names");
}
