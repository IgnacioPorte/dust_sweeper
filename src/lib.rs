use bitcoin::{
    Transaction, TxIn, TxOut, OutPoint, ScriptBuf, Witness, Sequence, transaction::Version, Address
};
use bitcoin::psbt::Psbt;
use bitcoin::Amount;
use bitcoincore_rpc::{Client, RpcApi};
use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;
use std::str::FromStr;

/// DustSweeper provides functionality to identify and clean up dust UTXOs from a Bitcoin wallet
/// in a privacy-preserving way.
///
/// # Privacy Features
/// - Groups UTXOs by address to avoid linking different addresses
/// - Creates separate transactions for each address's dust
///
/// # Example
/// ```no_run
/// use bitcoincore_rpc::{Auth, Client};
/// use dust_cleaner::DustSweeper;
///
/// let rpc_client = Client::new(
///     "http://localhost:18443",
///     Auth::UserPass("user".to_string(), "pass".to_string())
/// ).unwrap();
///
/// let sweeper = DustSweeper::new(rpc_client, 1000);
/// let dust_utxos = sweeper.get_dust_utxos().unwrap();
/// ```
pub struct DustSweeper {
    rpc_client: Client,
    threshold: u64,
}

impl DustSweeper {
    /// Creates a new DustSweeper instance
    ///
    /// # Arguments
    /// * `rpc_client` - Bitcoin Core RPC client
    /// * `threshold` - Amount in satoshis below which UTXOs are considered dust
    pub fn new(rpc_client: Client, threshold: u64) -> Self {
        Self { rpc_client, threshold }
    }

    /// Retrieves all UTXOs below the dust threshold from the wallet
    ///
    /// # Returns
    /// * `Result<Vec<ListUnspentResultEntry>>` - List of dust UTXOs or RPC error
    pub fn get_dust_utxos(&self) -> Result<Vec<bitcoincore_rpc::json::ListUnspentResultEntry>, bitcoincore_rpc::Error> {
        let utxos = self.rpc_client.list_unspent(None, None, None, None, None)?;
        Ok(utxos.into_iter()
            .filter(|u| u.amount.to_sat() < self.threshold)
            .collect())
    }

    /// Groups UTXOs by their source address to maintain privacy
    ///
    /// # Arguments
    /// * `utxos` - Vector of UTXOs to group
    ///
    /// # Returns
    /// * `HashMap<String, Vec<ListUnspentResultEntry>>` - UTXOs grouped by address
    fn group_utxos_by_address(&self, utxos: Vec<bitcoincore_rpc::json::ListUnspentResultEntry>) -> HashMap<String, Vec<bitcoincore_rpc::json::ListUnspentResultEntry>> {
        let mut grouped: HashMap<String, Vec<bitcoincore_rpc::json::ListUnspentResultEntry>> = HashMap::new();

        for utxo in utxos {
            if let Some(address) = &utxo.address {
                grouped.entry(address.clone().assume_checked().to_string()).or_default().push(utxo);
            }
        }

        grouped
    }

    /// Creates PSBTs to burn dust UTXOs to a specified address
    ///
    /// # Arguments
    /// * `dust_utxos` - Vector of dust UTXOs to burn
    /// * `fee` - Transaction fee in satoshis
    /// * `burn_addr` - Address to send dust to
    ///
    /// # Returns
    /// * `Result<Vec<Psbt>>` - Vector of unsigned PSBTs or error
    pub fn build_psbts_burn(&self, dust_utxos: Vec<bitcoincore_rpc::json::ListUnspentResultEntry>, fee: u64, burn_addr: &str) -> Result<Vec<Psbt>, bitcoin::psbt::Error> {
        let utxos_by_address = self.group_utxos_by_address(dust_utxos);
        let mut psbts = Vec::new();

        for (_address, utxos) in utxos_by_address {
            let psbt = self.build_psbt(utxos, fee, burn_addr)?;
            psbts.push(psbt);
        }

        Ok(psbts)
    }

    /// Builds a single PSBT for a group of UTXOs
    ///
    /// # Arguments
    /// * `dust_utxos` - Vector of dust UTXOs from the same address
    /// * `fee` - Transaction fee in satoshis
    /// * `burn_addr` - Address to send dust to
    ///
    /// # Returns
    /// * `Result<Psbt>` - Unsigned PSBT or error
    fn build_psbt(&self, dust_utxos: Vec<bitcoincore_rpc::json::ListUnspentResultEntry>, fee: u64, burn_addr: &str) -> Result<Psbt, bitcoin::psbt::Error> {
        let inputs: Vec<TxIn> = dust_utxos.iter().map(|u| {
            TxIn {
                previous_output: OutPoint { txid: u.txid, vout: u.vout },
                script_sig: ScriptBuf::new(),
                sequence: Sequence(0xFFFFFFFD), // Enable RBF
                witness: Witness::new(),
            }
        }).collect();

        let total_input_amount: u64 = dust_utxos.iter().map(|u| u.amount.to_sat()).sum();
        let output_value = total_input_amount.saturating_sub(fee);

        let burn_address = Address::from_str(burn_addr)
            .expect("❌ Invalid burn address")
            .assume_checked();

        let txout = TxOut {
            value: Amount::from_sat(output_value),
            script_pubkey: burn_address.script_pubkey(),
        };

        let outputs: Vec<TxOut> = vec![txout];

        let unsigned_tx = Transaction {
            version: Version(2),
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: inputs,
            output: outputs,
        };

        Psbt::from_unsigned_tx(unsigned_tx)
    }

    /// Converts a PSBT to base64 encoding for transport
    ///
    /// # Arguments
    /// * `psbt` - The PSBT to encode
    ///
    /// # Returns
    /// * `String` - Base64 encoded PSBT
    pub fn psbt_to_base64(psbt: &Psbt) -> String {
        general_purpose::STANDARD.encode(psbt.serialize())
    }
}
