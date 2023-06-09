/*
 * Example smart contract written in RUST
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://near-docs.io/develop/Contract
 *
 */

use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::{ AccountId, env, log, near_bindgen, PanicOnDefault, Promise };
use near_sdk::collections::{ LookupMap, UnorderedSet };
use near_sdk::json_types::U128;
use near_sdk::serde::{ Deserialize, Serialize };

// 인메모리에 저장
const PRIZE_AMOUNT: u128 = 5_000_000_000_000_000_000_000_000;

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonPuzzle {
    solution_hash: String,
    status: PuzzleStatus,
    answer: Vec<Answer>,
}
#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UnsolvedPuzzles {
    puzzles: Vec<JsonPuzzle>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Puzzle {
    status: PuzzleStatus,
    answer: Vec<Answer>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum PuzzleStatus {
    Unsolved,
    Solved {
        memo: String,
    },
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Answer {
    num: u8,
    start: CoordinatePair, // ⟵ Another struct we've defined
    direction: AnswerDirection, // ⟵ An enum we'll get to soon
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

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Crossword {
    // 영구스토리지에 저장됨
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
            "Only the owner may call this method"
        );

        let existing = self.puzzles.insert(
            &solution_hash,
            &(Puzzle {
                status: PuzzleStatus::Unsolved,
                answer: answers,
            })
        );
        assert!(existing.is_none(), "Puzzle with that key already exists");
        self.unsolved_puzzles.insert(&solution_hash);
    }

    pub fn submit_solution(&mut self, solution: String, memo: String) {
        let hashed_input = env::sha256(solution.as_bytes());
        let hashed_input_hex = hex::encode(&hashed_input);
        let mut puzzle = self.puzzles.get(&hashed_input_hex).expect("ERR_NOT_CORRECT_ANSWER");

        puzzle.status = match puzzle.status {
            PuzzleStatus::Unsolved =>
                PuzzleStatus::Solved {
                    memo: memo.clone(),
                },
            _ => {
                env::panic_str("ERR_PUZZLE_SOLVED");
            }
        };

        self.puzzles.insert(&hashed_input_hex, &puzzle);
        self.unsolved_puzzles.remove(&hashed_input_hex);

        log!("Puzzle with solution hash {} solved, with memo {}", hashed_input_hex, memo);

        // 맞춘 사람에게 Prize Money 전송

        Promise::new(env::predecessor_account_id()).transfer(PRIZE_AMOUNT);
    }

    pub fn get_puzzle_status(&self, solution_hash: String) -> Option<PuzzleStatus> {
        let puzzle = self.puzzles.get(&solution_hash);
        if puzzle.is_none() {
            return None;
        }
        Some(puzzle.unwrap().status)
    }
    pub fn get_unsolved_puzzles(&self) -> UnsolvedPuzzles {
        let solution_hashes = self.unsolved_puzzles.to_vec();
        let mut all_unsolved_puzzles = vec![];
        for hash in solution_hashes {
            let puzzle = self.puzzles
                .get(&hash)
                .unwrap_or_else(|| env::panic_str("ERR_LOADING_PUZZLE"));
            let json_puzzle = JsonPuzzle {
                solution_hash: hash,
                status: puzzle.status,
                answer: puzzle.answer,
            };
            all_unsolved_puzzles.push(json_puzzle);
        }
        UnsolvedPuzzles {
            puzzles: all_unsolved_puzzles,
        }
    }

    /* Version 1 */

    // pub fn get_puzzle_number(&self) -> u8 {
    //     PUZZLE_NUMBER
    // }
    //
    // pub fn get_solution(&self) -> String {
    //     self.crossword_solution.clone()
    // }

    // pub fn set_solution(&mut self, solution: String) {
    //     self.crossword_solution = solution;
    // }

    // 로그에 일정량의 가스가 필요하기 떄문에

    // pub fn guess_solution(&mut self, solution: String) -> bool {
    //     let hashed_input = env::sha256(solution.as_bytes());
    //     let hashed_input_hex = hex::encode(&hashed_input);
    //     if solution == self.crossword_solution {
    //         env::log_str("You guessed right");
    //         return true;
    //     } else {
    //         env::log_str(("Try again"));
    //         return  false;
    //     }
    // }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{ get_logs, VMContextBuilder };
    use near_sdk::{ testing_env, AccountId };

    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    fn get_hash(string: &String) -> String {
        let hash_bytes = env::sha256(string.as_bytes());
        let hash_string = hex::encode(hash_bytes);
        hash_string
    }

    #[test]
    fn debug_get_hash() {
        // Basic set up for a unit test
        // contract 단위 테스트를 위한 set up!
        testing_env!(VMContextBuilder::new().build());

        let debug_solution = "near nomicon ref finance";
        let debug_hash_bytes = env::sha256(debug_solution.as_bytes());
        let debug_hash_string = hex::encode(debug_hash_bytes);
        println!("Let's debug: {:?}", debug_hash_string);
    }
}