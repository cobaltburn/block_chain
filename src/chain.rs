use std::{fs, io};
use std::fs::{File, OpenOptions};
use std::io::{Write, BufRead};
use std::process::id;
use std::ptr::hash;
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;
use std::thread::spawn;
use crypto_hash::{Algorithm,hex_digest};

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
        for _ in 0..14 {
            let tx = tx.clone();
            let previous_hash = previous_block.hash.clone();
            spawn(move || {
                let mut result = None;
                let proof = "000000";
                let mut rng = rand::thread_rng();
                loop {
                    let proof_of_work: i64 = rng.gen();
                    let hash = hex_digest(
                        Algorithm::SHA256,
                        format!("{}{}{}{}", id, time, previous_hash, proof_of_work)
                            .as_bytes()
                    );
                    if proof == &hash[..proof.len()] {
                        result = Some((proof_of_work, hash));
                    }
                    match tx.send(result.clone()) {
                        Ok(_) => (),
                        _ => break,
                    }
                }
            });
        }
        loop {
            if let Some((proof_of_work, hash)) = rx.recv().unwrap() {
                return Block {
                    id, time, proof_of_work, hash,
                    previous_hash: previous_block.hash.clone(),
                }
            }
        };

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
        let id = 0;
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs();
        let previous_hash = String::from("0000000000000000000000000000000000000000000000000000000000000000");
        let proof_of_work: i64 = 0;
        let hash = hex_digest(
            Algorithm::SHA256,
            format!("{}{}{}{}", id, time, previous_hash, proof_of_work)
                .as_bytes()
        );
        let new_block = Block::new(&Block{
            id,
            time,
            previous_hash,
            proof_of_work,
            hash
        });
        let mut file = File::create("/home/michael/Documents/test/block_chain1.txt")
            .expect("file creation failed");
        let block_string = new_block.string();
        println!("{}", block_string);
        file.write_all(block_string.as_bytes())
            .expect("write failed");
        Chain { chain: vec![new_block]}
    }

    pub fn from(file: File) -> Chain {
        let file_iter = io::BufReader::new(file).lines();
        let mut chain = Vec::new();
        for  line in file_iter {
            if let Ok(block) = line {
                let mut block_vals = block.split(", ");
                chain.push(Block {
                    id: block_vals.next().unwrap().parse::<i32>().unwrap(),
                    time: block_vals.next().unwrap().parse::<u64>().unwrap(),
                    previous_hash: String::from(block_vals.next().unwrap()),
                    proof_of_work: block_vals.next().unwrap().parse::<i64>().unwrap(),
                    hash: String::from(block_vals.next().unwrap())
                });
            }
        }
        if !Chain::validate(&chain) {
            panic!("Invalid chain");
        }
        Chain { chain }
    }

    fn validate (chain: &Vec<Block>) -> bool {
        chain
            .iter()
            .zip(chain.iter().skip(1))
            .all(|(current, next)|
                     hex_digest(Algorithm::SHA256,
                                format!("{}{}{}{}",
                                        current.id,
                                        current.time,
                                        current.previous_hash,
                                        current.proof_of_work)
                                    .as_bytes()
                     ) == next.previous_hash )
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
    #[warn(unused_imports)]
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
            hash: String::from("0000000000000000000000000000000000000000000000000000000000000000"),
        };
        let new = Block::new(&blank);
        println!("{}, {}, {}, {}, {}", new.id, new.time, new.previous_hash, new.proof_of_work, new.hash);
    }
    #[test]
    fn check_hash() {
        let id = 1;
        let time = 1669066161;
        let previous_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        let proof_of_work: i64 = 8805102349990813383;
        let x = hex_digest(Algorithm::SHA256,
                   format!("{}{}{}{}", id, time, previous_hash, proof_of_work).as_bytes());

        println!("{}", x == "00009292bc04348d9bf2aa34024b0d8325af69a984af41af967b3651780613d0")
    }

}

