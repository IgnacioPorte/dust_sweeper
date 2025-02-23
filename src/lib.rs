use bitcoin::{
    Transaction, TxIn, TxOut, OutPoint, ScriptBuf, Witness, Sequence, transaction::Version, Address
};
use bitcoin::psbt::Psbt;
use bitcoin::Amount;

use bitcoincore_rpc::{Client, RpcApi};
use base64::{engine::general_purpose, Engine as _};
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
    
    pub fn build_psbt(&self, dust_utxos: Vec<bitcoincore_rpc::json::ListUnspentResultEntry>) -> Result<Psbt, bitcoin::psbt::Error> {
        let inputs: Vec<TxIn> = dust_utxos.iter().map(|u| {
            TxIn {
                previous_output: OutPoint { txid: u.txid, vout: u.vout },
                script_sig: ScriptBuf::new(),
                sequence: Sequence(0xFFFFFFFD),
                witness: Witness::new(),
            }
        }).collect();
    
        let total_input_amount: u64 = dust_utxos.iter().map(|u| u.amount.to_sat()).sum();
    
        let burn_address = Address::from_str("1BitcoinEaterAddressDontSendf59kuE")
            .expect("Invalid burn address")
            .assume_checked();
    
        let fee = 500;

        if total_input_amount <= fee {
            panic!("Total dust amount too small to cover fee!");
        }
    
        let txout = TxOut {
            value: Amount::from_sat(total_input_amount - fee),
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
