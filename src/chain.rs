use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::mpsc;
use crypto_hash::{Algorithm, hex_digest};
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;
use std::thread::spawn;

#[derive(Debug)]
struct Block {
    id: i32,
    time: u64,
    previous_hash: String,
    proof_of_work: i64,
    hash: String,
}

impl Block {
    fn new(previous_block: &Block) -> Block {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs();
        let id = previous_block.id + 1;

        let (tx, rx) = mpsc::channel();

        for _ in 0..4 {
            let tx = tx.clone();
            let previous_hash = previous_block.previous_hash.clone();
            spawn(move || {
                let mut result = None;
                let proof = "00000";
                let mut rng = rand::thread_rng();
                loop {
                    let proof_of_work: i64 = rng.gen();
                    let format_block = format!("{}{}{}{}", id, time, previous_hash, proof_of_work);
                    let hash = hex_digest(Algorithm::SHA256, format_block.as_bytes());
                    if proof == &hash[..proof.len()] {
                        result = Some((proof_of_work, hash))
                    }
                    match tx.send(result.clone()) {
                        Ok(_) => (),
                        _ => break,
                    }
                }
            });
        }
        let (proof_of_work, hash) = loop {
            match rx.recv().unwrap() {
                Some(x) => break x,
                None => (),
            }
        };

        Block {
            id, time, proof_of_work, hash,
            previous_hash: previous_block.hash.clone(),
        }
    }
}

impl Block {
    fn string(&self) -> String {
        format!("{}, {}, {}, {}, {}", self.id, self.time, self.previous_hash, self.proof_of_work, self.hash)
    }
}

pub struct Chain {
    chain: Vec<Block>
}

impl Chain {
    pub fn new() -> Chain {
        let blank = Block {
            id: 0,
            time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards")
                .as_secs(),
            previous_hash: String::from(""),
            proof_of_work: 0,
            hash: String::from("0000000000000000000000000000000000000000000000000000000000000000"),
        };
        let new_block = Block::new(&blank);
        let mut file = File::create("/home/michael/Documents/test/block_chain1.txt").expect("file creation failed");
        let block_string = new_block.string();
        println!("{}", block_string);
        file.write_all(block_string.as_bytes()).expect("write failed");
        Chain { chain: vec![new_block]}
    }

}

impl Chain {
    pub fn add_block (&mut self) {
        let new_block = Block::new(&self.chain[self.chain.len() - 1]);
        self.chain.push(new_block);
        self.save();
    }

    fn save(&self) {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(format!("/home/michael/Documents/test/block_chain{id}.txt", id = self.chain.len()))
            .expect("File failed to create");
        fs::copy(
            format!("/home/michael/Documents/test/block_chain{id}.txt", id = self.chain.len() - 1),
            format!("/home/michael/Documents/test/block_chain{id}.txt", id = self.chain.len())
        ).expect("copy failed");
        let block_string = self.chain[self.chain.len() - 1].string();
        println!("{}", block_string);
        file.write_all(format!("\n{}", block_string).as_bytes()).expect("write failed");
    }
}


mod tests {
    use super::*;

    #[test]
    fn build_block() {
        let blank = Block {
            id: 0,
            time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards")
                .as_secs(),
            previous_hash: String::from(""),
            proof_of_work: 0,
            hash: String::from(""),
        };
        let new = Block::new(&blank);
        println!("{}, {}, {}, {}, {}", new.id, new.time, new.previous_hash, new.proof_of_work, new.hash)
    }
    #[test]
    fn start_chain () {
        let mut x = Chain::new();
        for _ in 0..9 {
            x.add_block()
        }
    }

}

