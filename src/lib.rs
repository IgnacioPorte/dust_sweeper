use bitcoin::{
    Transaction, TxIn, TxOut, OutPoint, ScriptBuf, Witness, Sequence, transaction::Version, Address
};
use bitcoin::psbt::Psbt;
use bitcoin::Amount;
use bitcoincore_rpc::{Client, RpcApi};
use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;
use std::str::FromStr;

pub struct DustSweeper {
    rpc_client: Client,
    threshold: u64,
}

impl DustSweeper {
    pub fn new(rpc_client: Client, threshold: u64) -> Self {
        Self { rpc_client, threshold }
    }

    pub fn get_dust_utxos(&self) -> Result<Vec<bitcoincore_rpc::json::ListUnspentResultEntry>, bitcoincore_rpc::Error> {
        let utxos = self.rpc_client.list_unspent(None, None, None, None, None)?;
        Ok(utxos.into_iter()
            .filter(|u| u.amount.to_sat() < self.threshold)
            .collect())
    }

    fn group_utxos_by_address(&self, utxos: Vec<bitcoincore_rpc::json::ListUnspentResultEntry>) -> HashMap<String, Vec<bitcoincore_rpc::json::ListUnspentResultEntry>> {
        let mut grouped: HashMap<String, Vec<bitcoincore_rpc::json::ListUnspentResultEntry>> = HashMap::new();

        for utxo in utxos {
            if let Some(address) = &utxo.address {
                grouped.entry(address.clone().assume_checked().to_string()).or_default().push(utxo);
            }
        }

        grouped
    }

    pub fn build_psbts_burn(&self, dust_utxos: Vec<bitcoincore_rpc::json::ListUnspentResultEntry>, fee: u64, burn_addr: &str) -> Result<Vec<Psbt>, bitcoin::psbt::Error> {
        let utxos_by_address = self.group_utxos_by_address(dust_utxos);
        let mut psbts = Vec::new();

        for (_address, utxos) in utxos_by_address {
            let psbt = self.build_psbt(utxos, fee, burn_addr)?;
            psbts.push(psbt);
        }

        Ok(psbts)
    }

    fn build_psbt(&self, dust_utxos: Vec<bitcoincore_rpc::json::ListUnspentResultEntry>, fee: u64, burn_addr: &str) -> Result<Psbt, bitcoin::psbt::Error> {
        let inputs: Vec<TxIn> = dust_utxos.iter().map(|u| {
            TxIn {
                previous_output: OutPoint { txid: u.txid, vout: u.vout },
                script_sig: ScriptBuf::new(),
                sequence: Sequence(0xFFFFFFFD),
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

    pub fn psbt_to_base64(psbt: &Psbt) -> String {
        general_purpose::STANDARD.encode(psbt.serialize())
    }
}
