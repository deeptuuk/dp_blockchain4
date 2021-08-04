use num_bigint::BigUint;
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use nut::{DBBuilder, DB};
use std::collections::HashMap;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

const TARGET_BITS: u64 = 12;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    txid: Vec<u8>,
    vout_index: usize,
    script_sig: String,
}

impl TXInput {
    fn can_unlock_output_with(&self, unlock_data: &str) -> bool {
        return self.script_sig == unlock_data;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    value: i64,
    script_pubkey: String,
}

impl TXOutput {
    fn can_be_unlocked_with(&self, unlock_data: &str) -> bool {
        return self.script_pubkey == unlock_data;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    id: Vec<u8>,
    vin: Vec<TXInput>,
    vout: Vec<TXOutput>,
}

impl Transaction {
    pub fn new_coin_base_tx(to: String) -> Transaction {
        let mut tx = Transaction {
            id: vec![0u8; 32],
            vin: vec![TXInput {
                txid: vec![0u8; 32],
                vout_index: 0,
                script_sig: String::from(format!("Reward to {}", to)),
            }],
            vout: vec![TXOutput {
                value: 50,
                script_pubkey: to,
            }],
        };

        Transaction::set_id(&mut tx);
        tx
    }

    fn is_coinbase(&self) -> bool {
        return self.vin[0].txid == vec![0u8; 32]
    }

    pub fn new_utxo_transaction(from: &str, to: &str, amount: i64, bc: &BlockChainDb) -> Result<Transaction, ()> {
        let mut inputs: Vec<TXInput> = Vec::new();
        let mut outputs: Vec<TXOutput> = Vec::new();

        let (acc, valid_outputs) = bc.find_spendable_outputs(from, amount);
        //println!("This is a test {}", acc);
        //println!("{:?}", valid_outputs);

        if acc < amount {
            return Err(())
        }
        else {
            for (txid, outs) in valid_outputs {
                for (_, out) in outs.iter().enumerate() {
                    inputs.push(TXInput {
                        txid: txid.clone(),
                        vout_index: out.clone(),
                        script_sig: String::from(from),
                    })
                }
            }

            outputs.push(TXOutput {
                value: amount,
                script_pubkey: String::from(to),
            });

            if acc > amount {
                outputs.push(TXOutput {
                    value: acc - amount,
                    script_pubkey: String::from(from),
                });               
            }

            let mut tx = Transaction {
                id: vec![0u8; 32],
                vin: inputs,
                vout: outputs,
            };

            Transaction::set_id(&mut tx);

            return Ok(tx)
        }
    }

    fn serialize(temp: &Transaction) -> Vec<u8> {
        serde_json::to_vec(temp).unwrap()
    }    
    
    fn set_id(temp: &mut Transaction) {
        let mut hasher = Sha256::new();
        hasher.update(Transaction::serialize(temp));
        //hasher.update();
        let result = hasher.finalize();
        temp.id = result.as_slice().iter().cloned().collect();
    }
} 

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    timestamp: Duration,
    transactions: Vec<Transaction>,
    prev_block_hash: BigUint,
    hash: BigUint,
    nonce: u128,
}



impl Block {
    pub fn new_genesis_block(coinbase: Transaction) -> Result<Block, ()> {

        let mut block = Block {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap(),
            transactions: vec![coinbase],
            prev_block_hash: BigUint::new(vec![0u32; 8]),
            hash: BigUint::new(vec![0u32; 8]),
            nonce: 0,
        };
        
        match block.proof_of_work() {
            Ok(_) => return Ok(block),
            Err(_) => return Err(()),
        }
    }

    pub fn new_block(transactions: Vec<Transaction>, prev_block_hash: BigUint) -> Result<Block, ()> {
        let mut block = Block {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap(),
            transactions: transactions,
            prev_block_hash: prev_block_hash,
            hash: BigUint::new(vec![0u32; 8]),
            nonce: 0,
        };
        
        match block.proof_of_work() {
            Ok(_) => return Ok(block),
            Err(_) => return Err(()),
        }        
    }

    fn proof_of_work(&mut self) -> Result<(), ()> {

        let mut hasher = Sha256::new();
        let mut result = hasher.finalize_reset();

        let mut target_big = BigUint::new(Vec::new());
        target_big.set_bit(256 - TARGET_BITS, true);

        while self.nonce < u128::MAX {
            hasher.update(
                [
                    self.timestamp.as_nanos().to_string().as_bytes(),
                    &self.hash_transactions().to_bytes_le()[..],
                    &self.prev_block_hash.to_bytes_le()[..],
                    &TARGET_BITS.to_le_bytes(),
                    &self.nonce.to_le_bytes(),
                ]
                .concat(),
            );

            result = hasher.finalize_reset();
            //self.hash = result.as_slice().try_into().expect("slice with incorrect length");

            //let temp = BigUint::from_bytes_le(&self.hash);
            self.hash = BigUint::from_bytes_le(result.as_slice());

            if self.hash < target_big {
                return Ok(())
            } else {
                self.nonce = self.nonce + 1;
            }
        }
        return Err(())
    }

    fn hash_transactions(&self) -> BigUint {
        let mut hasher = Sha256::new();

        let mut temp: Vec<u8> = Vec::new();

        for i in &self.transactions {
            temp.append(&mut i.id.clone());
        }

        hasher.update(temp);
        let result = hasher.finalize();

        BigUint::from_bytes_le(result.as_slice())
    }

    pub fn serialize(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    pub fn deserialize(block: &Vec<u8>) -> Block {
        let temp: Block = serde_json::from_slice(block).unwrap();
        temp
    }

    pub fn show_block(i: &Block) {
        println!("TimeStamp: {:?}", i.timestamp);
        println!("Prev_hash: {:064x}", i.prev_block_hash);
        //println!("Data     : {}", i.data);
        println!("Hash     : {:064x}", i.hash);
        println!("Nonce    : {}", i.nonce);
        println!("");
    }      
}

pub struct BlockChainDb {
    tip: Vec<u8>,
    db: nut::DB,
}

impl BlockChainDb {
    pub fn new_blockchain(address: String) -> BlockChainDb {
        let mut tip: Vec<u8> = Vec::new();

        let mut db = DBBuilder::new("test.db").build().unwrap();
        let mut tx = db.begin_rw_tx().unwrap();

        let mut flag: u8 = 0;

        {
            match tx.bucket(b"blocksBucket") {
                Ok(blocks) => {
                    tip = blocks.get(b"l").unwrap().to_vec();
                },
                Err(_)   => {
                    flag = 1;          
                },
            }
        }

        if flag == 1 {
            let genesis_block = Block::new_genesis_block(Transaction::new_coin_base_tx(address)).unwrap();
            let mut blocks = tx.create_bucket(b"blocksBucket").unwrap();
            blocks.put(
                &genesis_block.hash.to_bytes_le(),
                genesis_block.serialize()
            ).unwrap();

            blocks.put(
                b"l",
                genesis_block.hash.to_bytes_le()
            ).unwrap();    
            
            tip = genesis_block.hash.to_bytes_le();
        }

        BlockChainDb {
            tip: tip,
            db: db,
        }
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) {
        let last_hash;
        {
            let tx = self.db.begin_tx().unwrap();
            let blocks = tx.bucket(b"blocksBucket").unwrap();
            last_hash = blocks.get(b"l").unwrap().to_vec();
        }

        let new_block = Block::new_block(transactions, BigUint::from_bytes_le(&last_hash[..])).unwrap();

        let mut tx = self.db.begin_rw_tx().unwrap();
        let mut blocks = tx.bucket_mut(b"blocksBucket").unwrap();

        blocks.put(
            &new_block.hash.to_bytes_le(),
            new_block.serialize()
        ).unwrap();

        blocks.put(
            b"l",
            new_block.hash.to_bytes_le()
        ).unwrap();    

        self.tip = new_block.hash.to_bytes_le();
    }

    pub fn show_blockchain(&self) {
        let mut blockchainiterator = BlockchainIterator::new(&self);

        loop {
            if let Some(block) = blockchainiterator.next() {
                //println!("{:?}", block)
                Block::show_block(&block);
            }
            else {
                break;
            }
        }     
    }

    pub fn find_unspent_transactions(&self, address: &str) -> Vec<Transaction> {
        let mut unspent_txs: Vec<Transaction> = Vec::new();
        let mut spent_txos: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();
        let mut bci = BlockchainIterator::new(&self);

        loop {
            if let Some(block) = bci.next() {
                //println!("{:?}", block)

                for tx in &block.transactions {
                    
                    'Outputs:
                    for (out_index, out) in tx.vout.iter().enumerate() {
                        if let Some(temp) = spent_txos.get(&tx.id) {
                            for (_, spent_out) in temp.iter().enumerate() {
                                if *spent_out == out_index {
                                    continue 'Outputs;
                                }
                            }
                        }

                        if out.can_be_unlocked_with(&address) {
                            unspent_txs.push(tx.clone());
                        }
                    }

                    if tx.is_coinbase() == false {
                        for (_, in_value) in tx.vin.iter().enumerate() {
                            if in_value.can_unlock_output_with(&address) {
                                spent_txos.entry(in_value.txid.clone()).or_insert(Vec::new()).push(in_value.vout_index);
                            }
                        }
                    }
                }

                //Block::show_block(&block);
            }
            else {
                break;
            }
        }

        unspent_txs

    }

    pub fn find_spendable_outputs(&self, address: &str, amount: i64) -> (i64, HashMap<Vec<u8>, Vec<usize>>) {
        let mut unspent_outputs: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();
        let unspent_txs = self.find_unspent_transactions(address);

        let mut accumulated: i64 = 0;

        'Work:
            for (_, tx) in unspent_txs.iter().enumerate() {
                for (out_idx, out) in tx.vout.iter().enumerate() {
                    if out.can_be_unlocked_with(address) && accumulated < amount {
                        accumulated += out.value;

                        unspent_outputs.entry(tx.id.clone()).or_insert(Vec::new()).push(out_idx);

                        if accumulated >= amount {
                            break 'Work;
                        }
                    }
                }
            }
        
        (accumulated, unspent_outputs)
    }

    pub fn get_balance(&self, address: &str) -> i64 {
        let unspent_txs = self.find_unspent_transactions(address);
        //println!("{:?}", unspent_txs);
        let mut accumulated: i64 = 0;       
        for (_, tx) in unspent_txs.iter().enumerate() {
            for (_, out) in tx.vout.iter().enumerate() {
                if out.can_be_unlocked_with(address) {
                    accumulated += out.value;
                }
            }
        }
        accumulated 
    }
}

pub struct BlockchainIterator {
    tip: Vec<u8>,
    db: nut::DB,    
}

impl BlockchainIterator {
    pub fn new(temp: &BlockChainDb) -> BlockchainIterator {
        BlockchainIterator {
            tip: temp.tip.clone(),
            db: temp.db.clone(),
        }
    }
}

impl Iterator for BlockchainIterator {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {

        let tx = self.db.begin_tx().unwrap();

        let blocks = tx.bucket(b"blocksBucket").unwrap();
        if let Some(temp) = blocks.get(&self.tip[..]) {
            let block: Block = Block::deserialize(&temp.to_vec());
            self.tip = block.prev_block_hash.to_bytes_le();

            return Some(block)
        }
        else {
            return None;
        }
    }
}