use {
    anchor_spl::associated_token,
    litesvm::LiteSVM,
    litesvm_token::CreateMint,
    solana_keypair::Keypair,
    solana_message::{Instruction, Message, VersionedMessage},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

mod ix_handlers;
use ix_handlers::*;

fn send(
    svm: &mut LiteSVM,
    ixs: &[Instruction],
    payer: &Keypair,
    signers: &[&Keypair],
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

fn setup() -> (
    LiteSVM,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
) {
    let program_id = amm_video::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/amm_video.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let mint_x = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();
    let mint_y = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let config =
        Pubkey::find_program_address(&[b"config", &123u64.to_le_bytes()], &amm_video::id()).0;
    let mint_lp = Pubkey::find_program_address(&[b"lp", config.as_ref()], &amm_video::id()).0;
    let treasury_authority =
        Pubkey::find_program_address(&[b"treasury", config.as_ref()], &amm_video::id()).0;

    let vault_x = associated_token::get_associated_token_address(&config, &mint_x);
    let vault_y = associated_token::get_associated_token_address(&config, &mint_y);
    let treasury_x = associated_token::get_associated_token_address(&treasury_authority, &mint_x);
    let treasury_y = associated_token::get_associated_token_address(&treasury_authority, &mint_y);

    (
        svm,
        payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority,
        treasury_x,
        treasury_y,
    )
}

#[test]
fn test_initialize() {
    let (
        mut svm,
        payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority,
        treasury_x,
        treasury_y,
    ) = setup();
    let instruction = create_initialise_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority,
        treasury_x,
        treasury_y,
    );
    let res = send(&mut svm, &[instruction], &payer, &[&payer]);
    assert!(res.is_ok());
}

#[test]
pub fn test_deposit() {
    let (
        mut svm,
        payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority,
        treasury_x,
        treasury_y,
    ) = setup();
    let init_ix = create_initialise_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority,
        treasury_x,
        treasury_y,
    );
    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let res = send(&mut svm, &[init_ix, deposit_ix], &payer, &[&payer]);
    assert!(res.is_ok());
}

#[test]
pub fn test_withdraw() {
    let (
        mut svm,
        payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority,
        treasury_x,
        treasury_y,
    ) = setup();
    let init_ix = create_initialise_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority,
        treasury_x,
        treasury_y,
    );
    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let withdraw_ix = create_withdraw_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let res = send(
        &mut svm,
        &[init_ix, deposit_ix, withdraw_ix],
        &payer,
        &[&payer],
    );
    assert!(res.is_ok());
}

#[test]
pub fn test_swap_x_for_y() {
    let (
        mut svm,
        payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority,
        treasury_x,
        treasury_y,
    ) = setup();
    let init_ix = create_initialise_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority,
        treasury_x,
        treasury_y,
    );
    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let swap_ix = create_swap_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        mint_lp,
        config,
        vault_x,
        vault_y,
        treasury_authority,
        treasury_x,
        treasury_y,
        true,          // is_x = true (swap X -> Y)
        10_000_000,
        5_000_000,
    );
    let res = send(&mut svm, &[init_ix, deposit_ix, swap_ix], &payer, &[&payer]);
    assert!(res.is_ok());
}

#[test]
pub fn test_swap_y_for_x() {
    let (
        mut svm,
        payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority,
        treasury_x,
        treasury_y,
    ) = setup();
    let init_ix = create_initialise_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority,
        treasury_x,
        treasury_y,
    );
    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let swap_ix = create_swap_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        mint_lp,
        config,
        vault_x,
        vault_y,
        treasury_authority,
        treasury_x,
        treasury_y,
        false,         // is_x = false (swap Y -> X)
        10_000_000,
        5_000_000,
    );
    let res = send(&mut svm, &[init_ix, deposit_ix, swap_ix], &payer, &[&payer]);
    assert!(res.is_ok());
}

#[test]
fn test_swap_rejects_zero_amount() {
    let (
        mut svm,
        payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority,
        treasury_x,
        treasury_y,
    ) = setup();
    let init_ix = create_initialise_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority,
        treasury_x,
        treasury_y,
    );
    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let swap_ix = create_swap_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        mint_lp,
        config,
        vault_x,
        vault_y,
        treasury_authority,
        treasury_x,
        treasury_y,
        true,
        0,             // zero amount should fail
        0,
    );
    let res = send(&mut svm, &[init_ix, deposit_ix, swap_ix], &payer, &[&payer]);
    assert!(res.is_err());
}
