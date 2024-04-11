#![no_main]

use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

use ibc::core::host::types::identifiers::ChainId;
use ibc_testkit::hosts::block::HostType;
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

use ibc::core::client::context::client_state::ClientStateCommon;
use ibc::core::client::context::ClientValidationContext;
use ibc::core::client::types::msgs::{ClientMsg, MsgCreateClient};
use ibc::core::client::types::Height;
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::{ClientStateRef, ValidationContext};
use ibc_testkit::fixtures::core::signer::dummy_account_id;
use ibc_testkit::testapp::ibc::clients::mock::client_state::{
    client_type as mock_client_type, MockClientState,
};
use ibc_testkit::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use ibc_testkit::testapp::ibc::clients::mock::header::MockHeader;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::{MockContext, MockIbcStore};

fn main() {
    let height: Height = env::read();

    let mut ctx = MockContext {
        host_chain_type: HostType::Mock,
        host_chain_id: ChainId::new("mockZ-1").unwrap(),
        max_history_size: 5,
        history: vec![],
        block_time: Duration::from_secs(15),
        ibc_store: Arc::new(Mutex::new(MockIbcStore::default())),
    };

    let mut router = MockRouter::new_with_transfer();
    let signer = dummy_account_id();

    let msg = MsgCreateClient::new(
        MockClientState::new(MockHeader::new(height)).into(),
        MockConsensusState::new(MockHeader::new(height)).into(),
        signer,
    );

    let msg_envelope = MsgEnvelope::from(ClientMsg::from(msg.clone()));

    let client_type = mock_client_type();
    let client_id = client_type.build_client_id(ctx.client_counter().unwrap());

    let res = validate(&ctx, &router, msg_envelope.clone());

    assert!(res.is_ok(), "validation happy path");

    let res = execute(&mut ctx, &mut router, msg_envelope);

    assert!(res.is_ok(), "execution happy path");

    let expected_client_state = ClientStateRef::<MockContext>::try_from(msg.client_state).unwrap();
    assert_eq!(expected_client_state.client_type(), client_type);
    assert_eq!(ctx.client_state(&client_id).unwrap(), expected_client_state);

    // write public output to the journal
    env::commit(&client_id);
}
