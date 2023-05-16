/*
 * Example smart contract written in RUST
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://near-docs.io/develop/Contract
 *
 */

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen};

// 인메모리에 저장
const PUZZLE_NUMBER: u8 = 1;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    // 영구스토리지에 저장됨
    crossword_solution: String
}

#[near_bindgen]
impl Contract {

    pub fn get_puzzle_number(&self) -> u8 {
        PUZZLE_NUMBER
    }

    pub fn set_solution(&mut self, solution: String) {
        self.crossword_solution = solution;
    }

    // 로그에 일정량의 가스가 필요하기 떄문에
    pub fn guess_solution(&mut self, solution: String) -> bool{
        if solution == self.crossword_solution {
            env::log_str("You guessed right");
            return true;
        } else {
            env::log_str(("Try again"));
            return  false;
        }
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

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
    fn check_guess_solution() {
        let alice = AccountId::new_unchecked("alice.testnet".to_string());
        let context = get_context(alice);
        testing_env!(context.build());

        let mut contract = Contract {
            crossword_solution: "69c2feb084439956193f4c21936025f14a5a5a78979d67ae34762e18a7206a0f".to_string()
        };
        let mut guess_result = contract.guess_solution("It is Wrong Answer".to_string());
        assert!(!guess_result, "Expected a failure from the wrong guess");

        guess_result = contract.guess_solution(get_hash(&"near nomicon ref finance".to_string()));
        assert!(guess_result, "Expected the correct answer to return true.");

        assert_eq!(
            get_logs(),
            ["Try again", "You guessed right"],
            "Expected a successful log after the previous failed log."
        );

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
