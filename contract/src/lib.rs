mod account;
mod game;
mod storage_tracker;
mod enumerable;

use crate::account::Account;
use crate::game::{GameConfig, GameData};
use crate::StorageKeys::{Accounts, GamePerAccountId, Games, StorageDeposit};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::{attached_deposit, predecessor_account_id, storage_byte_cost, storage_usage};
use near_sdk::store::LookupMap;
use near_sdk::{near_bindgen, AccountId, Balance, BorshStorageKey, CryptoHash};

type GameId = u64;

#[derive(BorshStorageKey, BorshDeserialize, BorshSerialize)]
enum StorageKeys {
    Games,
    GamePerAccountId,
    VoteKick { hash: CryptoHash },
    PlayersVotes { hash: CryptoHash },
    Votes { hash: CryptoHash },
    PlayersInGame { hash: CryptoHash },
    StorageDeposit,
    Accounts,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    games: LookupMap<GameId, GameData>,
    accounts: LookupMap<AccountId, Account>,
    game_per_account_id: LookupMap<AccountId, Option<GameId>>,
    storage_deposit: LookupMap<AccountId, Balance>,
    next_game_id: u64,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            games: LookupMap::new(Games),
            accounts: LookupMap::new(Accounts),
            game_per_account_id: LookupMap::new(GamePerAccountId),
            storage_deposit: LookupMap::new(StorageDeposit),
            next_game_id: 0,
        }
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn init_game(&mut self, game_config: GameConfig) -> GameData {
        let game_data = GameData::new(&game_config, self.next_game_id);
        let account_id = predecessor_account_id();
        let deposit = attached_deposit();
        let mut account = self.internal_unwrap_account_or_create(&account_id, deposit);
        account.start_storage_tracker();
        self.games.insert(self.next_game_id, game_data.clone());
        account.stop_storage_tracker();
        self.internal_set_account(&account_id, account);
        self.next_game_id += 1;
        game_data
    }

    pub fn make_move(&mut self, game_data: GameData) {
        let account_id = predecessor_account_id();
        if account_id.ne(&game_data.current_player_id) {
            panic!("Now is not you turn")
        }

        if game_data.is_game_over {
            return;
        }
        self.games
            .insert(game_data.game_id, game_data)
            .expect("Game not found");
    }


    pub fn vote_kick(&mut self, player_to_kick_id: AccountId, game_id: GameId) {
        let game_data = self.games.get_mut(&game_id).expect("Game not found");
        let is_player_kicked = game_data.vote_kick(player_to_kick_id.clone());

        if is_player_kicked {
            self.game_per_account_id.insert(player_to_kick_id, None);
        }
    }
}
