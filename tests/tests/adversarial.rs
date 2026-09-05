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

fn an_about_me(name: &str) -> AboutMe {
    AboutMe {
        display_name: name.to_string(),
        what_matters_to_me: "Seeing my grandchildren".into(),
        how_to_communicate_with_me: "Speak to my left side, I'm deaf on the right".into(),
        how_to_support_me: "Give me time to answer".into(),
        people_who_matter: "My daughter Ruth".into(),
    }
}

/// A circle whose founder is `founder`, built from the real packed DNA.
async fn circle_dna(founder: &AgentPubKey) -> DnaFile {
    let properties = CircleProperties {
        founder: founder.clone(),
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
        .map(|i| {
            SerializedBytes::try_from(i.clone()).map(MembraneProof::new)
        })
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

/// Alice founds a circle and Bob joins it with a genuine invitation.
async fn a_circle_with_a_member() -> (SweetConductor, CellId, CellId) {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let bob = SweetAgents::one(conductor.keystore()).await;
    let dna = circle_dna(&alice).await;

    let alice_cell = join(&conductor, "alice", &alice, &dna, None)
        .await
        .expect("the founder needs no invitation to her own circle");

    let invitation: Invitation = conductor.call(&zome(&alice_cell), "invite", bob.clone()).await;

    let bob_cell = join(&conductor, "bob", &bob, &dna, Some(&invitation))
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
    let for_bob: Invitation = conductor.call(&zome(&alice_cell), "invite", bob.clone()).await;

    let result = join(&conductor, "dave", &dave, &dna, Some(&for_bob)).await;
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
    let for_bob: Invitation = conductor.call(&zome(&alice_cell), "invite", bob.clone()).await;
    let bob_cell = join(&conductor, "bob", &bob, &dna, Some(&for_bob))
        .await
        .unwrap();

    // Bob can call invite() — there is no permission check on it — but his
    // signature is not the founder's.
    let forged: Invitation = conductor
        .call(&zome(&bob_cell), "invite", mallory.clone())
        .await;

    let result = join(&conductor, "mallory", &mallory, &dna, Some(&forged)).await;
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

    assert!(
        result.is_err(),
        "only the author of a record may delete it"
    );
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
