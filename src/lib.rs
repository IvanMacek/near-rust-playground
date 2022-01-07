use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{log, env, near_bindgen, PanicOnDefault, AccountId, Promise};
use near_sdk::collections::{LookupMap, UnorderedSet};

const PRIZE_AMOUNT: u128 = 5_000_000_000_000_000_000_000_000;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Puzzle {
    status: PuzzleStatus,
    answer: Vec<Answer>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Answer {
    num: u8,
    start: CoordinatePair,
    direction: AnswerDirection,
    length: u8,
    clue: String,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CoordinatePair {
    x: u8,
    y: u8,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum AnswerDirection {
    Across,
    Down,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum PuzzleStatus {
    Unsolved,
    Solved { memo: String },
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonPuzzle {
    solution_hash: String,
    status: PuzzleStatus,
    answer: Vec<Answer>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Crossword {
    owner_id: AccountId,
    puzzles: LookupMap<String, Puzzle>,
    unsolved_puzzles: UnorderedSet<String>,
}

#[near_bindgen]
impl Crossword {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            puzzles: LookupMap::new(b"c"),
            unsolved_puzzles: UnorderedSet::new(b"u"),
        }
    }

    pub fn new_puzzle(&mut self, solution_hash: String, answers: Vec<Answer>) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only the owner can call this method"
        );
        let existing = self.puzzles.insert(
            &solution_hash, 
            &Puzzle {
                status: PuzzleStatus::Unsolved, 
                answer: answers,
            }
        );

        assert!(existing.is_none(), "Puzzle with that key already exists");
        self.unsolved_puzzles.insert(&solution_hash);
    }

    pub fn submit_solution(&mut self, solution: String, memo: String) {
        let hashed_input = env::sha256(solution.as_bytes());
        let hashed_input_hex = hex::encode(&hashed_input);
    
        // Check to see if the hashed answer is among the puzzles
        let mut puzzle = self
            .puzzles
            .get(&hashed_input_hex)
            .expect("ERR_NOT_CORRECT_ANSWER");
    
        // Check if the puzzle is already solved. If it's unsolved, set the status to solved,
        //   then proceed to update the puzzle and pay the winner.
        puzzle.status = match puzzle.status {
            PuzzleStatus::Unsolved => PuzzleStatus::Solved { memo: memo.clone() },
            _ => {
                env::panic_str("ERR_PUZZLE_SOLVED");
            }
        };
    
        // Reinsert the puzzle back in after we modified the status:
        self.puzzles.insert(&hashed_input_hex, &puzzle);
        // Remove from the list of unsolved ones
        self.unsolved_puzzles.remove(&hashed_input_hex);
    
        log!(
            "Puzzle with solution hash {} solved, with memo: {}",
            hashed_input_hex,
            memo
        );
    
        // Transfer the prize money to the winner
        Promise::new(env::predecessor_account_id()).transfer(PRIZE_AMOUNT);
    }
}

/*
 * the rest of this file sets up unit tests
 * to run these, the command will be:
 * cargo test --package rust-template -- --nocapture
 * Note: 'rust-template' comes from Cargo.toml's 'name' key
 */

