use dp_blockchain4::{Block, BlockChainDb, BlockchainIterator, Transaction};

fn main() {
    let mut blockchain = BlockChainDb::new_blockchain(String::from("Ivan"));
    //blockchain.add_block(vec![Transaction::new_coin_base_tx(String::from("Ivan"))]);
    let tx = Transaction::new_utxo_transaction("Ivan", "Pedro", 5, &blockchain).unwrap();
    blockchain.add_block(vec![tx]);

    let tx1 = Transaction::new_utxo_transaction("Ivan", "Test", 5, &blockchain).unwrap();
    blockchain.add_block(vec![tx1]);   

    blockchain.show_blockchain();

    println!("Balance of 'Ivan': {}",blockchain.get_balance("Ivan"));
    println!("Balance of 'Ivan': {}",blockchain.get_balance("Pedro"));
    println!("Balance of 'Pedro': {}",blockchain.get_balance("Test"));
}