use dust_cleaner::DustSweeper;
use clap::Parser;
use bitcoincore_rpc::{Auth, Client};


#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    rpc: String,

    #[clap(short, long)]
    user: String,

    #[clap(short, long)]
    pass: String,

    #[clap(short, long, default_value = "1000")]
    threshold: u64,
}

fn main() {
    let args = Args::parse();

    let rpc_client = Client::new(
        &args.rpc,
        Auth::UserPass(args.user, args.pass)
    ).expect("RPC connection failed");

    let sweeper = DustSweeper::new(rpc_client, args.threshold);

    let dust_utxos = sweeper.get_dust_utxos().expect("Failed fetching dust UTXOs");

    if dust_utxos.is_empty() {
        println!("🎉 No dust UTXOs below {} sats found!", args.threshold);
        return;
    }

    println!("🔍 Found {} dust UTXOs:", dust_utxos.len());
    for utxo in &dust_utxos {
        println!("• {}:{} - {} sats", utxo.txid, utxo.vout, utxo.amount.to_sat());
    }

    let psbt = sweeper.build_psbt(dust_utxos).expect("Failed building PSBT");
    let psbt_base64 = DustSweeper::psbt_to_base64(&psbt);
    println!("\n📝 PSBT (base64):\n{}", psbt_base64);
}
