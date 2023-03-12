
use anyhow::anyhow;
use clap::{IntoApp, Parser};
use solana_clap_v3_utils::keypair::pubkey_from_path;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use account_history_client::{DataRegion, initialize_account_close, initialize_account_history, resolve_account_close, update};
use account_history_client::config::{KeypairArg, UrlArg};
use account_history_program::state::{account_history_address, AccountHistoryRaw};
/// Solana Account History CLI
///
/// Interact with the Account History Program,
/// and take historical snapshots of account data.
#[derive(Parser, Debug)]
pub struct Opt {
    #[clap(flatten)]
    rpc_url: UrlArg,
    #[clap(flatten)]
    keypair: KeypairArg,
    #[clap(subcommand)]
    subcommand: Subcommand,
}
impl Opt {
    pub fn process(self) -> anyhow::Result<()> {
        let app = Opt::into_app();
        let matches = app.get_matches();
        let rpc_url = self.rpc_url.resolve(None)?;
        let client = RpcClient::new(rpc_url);
        let signer = self.keypair.resolve(&matches, None)?;
        let signer_pubkey = signer.pubkey();
        match self.subcommand {
            Subcommand::Initialize {
                capacity,
                min_slot_delay,
                min_close_delay,
                watched_account,
                data_regions,
            } => {
                let watched_account = pubkey_from_path(
                    &matches,
                    &watched_account,
                    "keypair",
                    &mut None,
                ).map_err(|_| anyhow!("Invalid pubkey or path: {}", watched_account))?;
                let seed = Keypair::new();
                let ix = initialize_account_history(
                    capacity,
                    data_regions.into_iter().map(|d| d.into()).collect(),
                    min_slot_delay,
                    min_close_delay,
                    signer_pubkey,
                    seed.pubkey(),
                    watched_account,
                );
                let addr = account_history_address(seed.pubkey().to_bytes()).0;
                println!("Creating history account: {} watching data at {}", addr, watched_account);
                let tx = Transaction::new_signed_with_payer(
                    &[ix],
                    Some(&signer_pubkey),
                    &vec![signer, Box::new(seed)],
                    client.get_latest_blockhash()?
                );
                let signature = client.send_transaction(&tx)
                    .map_err(|e| {
                        println!("{:#?}", &e);
                        e
                    })?;
                println!("{}", signature);
            },
            Subcommand::Update { history_account } => {
                let mut account_data = client.get_account_data(&history_account)?;
                let history = AccountHistoryRaw::from_buffer(&mut account_data)?;
                let ix = update(
                    signer_pubkey,
                    history_account,
                    history.associated_account(),
                );
                let tx = Transaction::new_signed_with_payer(
                    &[ix],
                    Some(&signer_pubkey),
                    &vec![signer],
                    client.get_latest_blockhash()?
                );
                let signature = client.send_transaction(&tx)
                    .map_err(|e| {
                        println!("{:#?}", &e);
                        e
                    })?;
                println!("{}", signature);
            },
            Subcommand::InitClose { history_account } => {
                println!("Initializing account close procedure on {}", history_account);
                let ix = initialize_account_close(
                    signer_pubkey,
                    history_account,
                );
                let tx = Transaction::new_signed_with_payer(
                    &[ix],
                    Some(&signer_pubkey),
                    &vec![signer],
                    client.get_latest_blockhash()?
                );
                let signature = client.send_transaction(&tx)
                    .map_err(|e| {
                        println!("{:#?}", &e);
                        e
                    })?;
                println!("{}", signature);
            },
            Subcommand::ResolveClose { history_account, rent_recipient } => {
                println!("Resolving account close procedure on {}", history_account);
                let ix = resolve_account_close(
                    signer_pubkey,
                    history_account,
                    rent_recipient,
                );
                let tx = Transaction::new_signed_with_payer(
                    &[ix],
                    Some(&signer_pubkey),
                    &vec![signer],
                    client.get_latest_blockhash()?
                );
                let signature = client.send_transaction(&tx)
                    .map_err(|e| {
                        println!("{:#?}", &e);
                        e
                    })?;
                println!("{}", signature);
            },
        }
        Ok(())
    }
}

#[derive(Parser, Debug)]
pub enum Subcommand {
    /// Create a new history account.
    Initialize {
        /// How many data elements to store
        #[clap(long)]
        capacity: u32,
        /// Configures the minimum elapsed slots since last update.
        /// Defaults to 1.
        #[clap(long)]
        min_slot_delay: Option<u32>,
        /// Configures the amount of required wait time before the account can be closed.
        /// Defaults to 0.
        #[clap(long)]
        min_close_delay: Option<u32>,
        /// The target account whose data to watch
        watched_account: String,
        /// Account data regions to capture and store on each element of the history account.
        #[clap(min_values=1, parse(try_from_str = DataRegion::try_from))]
        data_regions: Vec<DataRegion>,
    },
    /// Store a current snapshot of account data.
    Update {
        #[clap(parse(try_from_str=Pubkey::try_from))]
        history_account: Pubkey,
    },
    /// Start the process to close a history account.
    InitClose {
        #[clap(parse(try_from_str=Pubkey::try_from))]
        history_account: Pubkey,
    },
    /// Finish the process to close a history account.
    ResolveClose {
        #[clap(long, parse(try_from_str=Pubkey::try_from))]
        rent_recipient: Option<Pubkey>,
        #[clap(parse(try_from_str=Pubkey::try_from))]
        history_account: Pubkey,
    },
}


fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    opt.process()?;
    Ok(())
}