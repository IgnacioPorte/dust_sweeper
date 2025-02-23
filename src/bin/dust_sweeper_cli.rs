use dust_cleaner::DustSweeper;
use clap::Parser;
use bitcoincore_rpc::{Auth, Client};

#[derive(Parser)]
struct Args {
    #[clap(short, long, help = "Bitcoin Core RPC URL (e.g., http://localhost:18443)")]
    rpc: String,

    #[clap(short, long, help = "RPC username")]
    user: String,

    #[clap(short, long, help = "RPC password")]
    pass: String,

    #[clap(short, long, default_value = "1000", help = "Dust threshold in sats")]
    threshold: u64,

    #[clap(long, help = "Dry-run mode (only list dust UTXOs, no PSBT created)")]
    dry_run: bool,

    #[clap(long, default_value = "1BitcoinEaterAddressDontSendf59kuE", help = "Burn address to send dust to")]
    burn_address: String,

    #[clap(long, default_value = "500", help = "Fixed fee in sats")]
    fee: u64,
}

fn main() {
    let args = Args::parse();

    let rpc_client = Client::new(
        &args.rpc,
        Auth::UserPass(args.user, args.pass)
    ).expect("❌ RPC connection failed");

    let sweeper = DustSweeper::new(rpc_client, args.threshold);

    let dust_utxos = sweeper.get_dust_utxos().expect("❌ Failed fetching dust UTXOs");

    if dust_utxos.is_empty() {
        println!("✅ No dust UTXOs below {} sats found!", args.threshold);
        return;
    }

    println!("🔍 Found {} dust UTXOs:", dust_utxos.len());
    for utxo in &dust_utxos {
        println!("💰 UTXO: {} sats", utxo.amount);
    }

    println!("Using fixed fee: {} sats", args.fee);

    if args.dry_run {
        println!("\n🚨 Dry-run enabled. No PSBT created.");
        return;
    }

    let psbts = sweeper.build_psbts_burn(dust_utxos, args.fee, &args.burn_address)
        .expect("❌ Failed building PSBT");

    for (i, psbt) in psbts.iter().enumerate() {
        let psbt_base64 = DustSweeper::psbt_to_base64(psbt);
        println!("\n🔥 PSBT #{} (Burning dust to {}):\n{}", i + 1, args.burn_address, psbt_base64);
    }
}
