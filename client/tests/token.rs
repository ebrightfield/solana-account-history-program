use std::thread::sleep;
use std::time::Duration;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use clap::{IntoApp, Parser};
use solana_client::rpc_client::RpcClient;
use solana_sdk::clock::Slot;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::instruction::{initialize_mint, mint_to};
use account_history_client::config::KeypairArg;
use account_history_client::{initialize_account_history, update};
use account_history_program::state::account_history_address;
use account_history_program::state::interpreted::{AccountHistory, AccountHistoryIterator, AccountHistoryIteratorRev};
use bytemuck::{Zeroable, Pod};
use solana_sdk::program_pack::Pack;
use solana_sdk::system_instruction::create_account;
use spl_token::state::Mint;

#[derive(Default, Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct HistoricalBalance {
    slot: Slot,
    balance: u64,
}

/// Record token data
#[test]
fn token() {
    let matches = KeypairArg::into_app().get_matches();
    let keypair = KeypairArg::parse().resolve(&matches, None).unwrap();
    println!("Signer: {}", keypair.pubkey());
    let client = RpcClient::new_with_commitment("http://localhost:8899", CommitmentConfig::processed());
    let mint = Keypair::new();
    let mint_pubkey = mint.pubkey();
    let ix0 = create_account(
        &keypair.pubkey(),
        &mint_pubkey,
        1000000000,
        Mint::LEN as u64,
        &Token::id(),
    );
    let ix1 = initialize_mint(
        &Token::id(),
        &mint_pubkey,
        &keypair.pubkey(),
        None,
        6
    ).unwrap();
    let ix2 = create_associated_token_account(
        &keypair.pubkey(),
        &keypair.pubkey(),
        &mint_pubkey,
        &Token::id(),
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix0.clone(), ix1.clone(), ix2.clone()],
        Some(&keypair.pubkey()),
        &vec![keypair, Box::new(mint)],
        client.get_latest_blockhash().unwrap()
    );
    let signature = client.send_transaction(&tx)
        .map_err(|e| {
            println!("{:#?}", &e);
            e
        }).unwrap();
    println!("{}", signature);

    sleep(Duration::from_secs(1));

    println!("Initializing account history");
    // Initialize a new history account to watch the token balance of the ATA
    let keypair = KeypairArg::parse().resolve(&matches, None).unwrap();
    let watched_account = get_associated_token_address(&keypair.pubkey(), &mint_pubkey);
    let seed = Keypair::new();
    let history_address = account_history_address(seed.pubkey().to_bytes()).0;
    // Balance is stored at region 64:8
    let ix = initialize_account_history(
        3,
        vec![(64, 8)],
        None,
        None,
        keypair.pubkey(),
        seed.pubkey(),
        watched_account,
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&keypair.pubkey()),
        &vec![keypair, Box::new(seed)],
        client.get_latest_blockhash().unwrap()
    );
    let signature = client.send_transaction(&tx)
        .map_err(|e| {
            println!("{:#?}", &e);
            e
        }).unwrap();
    println!("{}", signature);

    sleep(Duration::from_secs(1));

    let mut data = client.get_account_data(
        &history_address,
    ).unwrap();
    let balance_history = AccountHistory::<HistoricalBalance>::from_buffer(&mut data).unwrap();
    assert_eq!(balance_history.most_recent_index(), 1);
    for i in 1..8 {
        let keypair = KeypairArg::parse().resolve(&matches, None).unwrap();
        let ix1 = mint_to(
            &Token::id(),
            &mint_pubkey,
            &watched_account,
            &keypair.pubkey(),
            &[],
            1000,
        ).unwrap();
        let ix2 = update(
            keypair.pubkey(),
            history_address,
            watched_account,
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix1, ix2],
            Some(&keypair.pubkey()),
            &vec![keypair],
            client.get_latest_blockhash().unwrap()
        );
        let signature = client.send_transaction(&tx)
            .map_err(|e| {
                println!("{:#?}", &e);
                e
            }).unwrap();
        println!("{}", signature);

        sleep(Duration::from_secs(1));
        let mut data = client.get_account_data(
            &history_address,
        ).unwrap();
        let balance_history = AccountHistory::<HistoricalBalance>::from_buffer(&mut data).unwrap();
        assert_eq!(balance_history.most_recent_index(), (1 + i) % balance_history.capacity());
    }

    let mut data = client.get_account_data(
        &history_address,
    ).unwrap();
    let balance_history = AccountHistory::<HistoricalBalance>::from_buffer(&mut data).unwrap();
    println!("{:#?}", balance_history);
    assert_eq!(balance_history.most_recent_entry().balance, 7000);
    let iterator = AccountHistoryIterator::from(&balance_history);
    println!("{:?}", &iterator);
    for (i, b) in iterator.enumerate() {
        assert_eq!(b.balance as usize, 7000-(i*1000));
    }
    let iterator = AccountHistoryIteratorRev::from(&balance_history);
    println!("{:?}", &iterator);
    for (i, b) in iterator.enumerate() {
        assert_eq!(b.balance as usize, 5000+(i*1000));
    }
}