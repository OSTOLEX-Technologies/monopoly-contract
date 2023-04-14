mod game;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{AccountId, BorshStorageKey, CryptoHash, near_bindgen};
use near_sdk::store::LookupMap;
use crate::game::{GameConfig, GameData};
use crate::StorageKeys::Games;

type GameId = u64;

#[derive(BorshStorageKey, BorshDeserialize, BorshSerialize)]
enum StorageKeys {
    Games,
    VoteKick {hash: CryptoHash},
    PlayersVotes {hash: CryptoHash},
    Votes {hash: CryptoHash},
    PlayersInGame {hash: CryptoHash},
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    games: LookupMap<GameId, GameData>,
    next_game_id: u64,
}

impl Default for Contract{
    fn default() -> Self {
        Self {
            games: LookupMap::new(Games),
            next_game_id: 0,
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn init_game(&mut self, game_config: GameConfig) -> GameData {
        let game_data = GameData::new(&game_config, self.next_game_id);
        self.next_game_id += 1;

        game_data
    }

    pub fn make_move(&mut self, game_data: GameData) {
        if !self.games.contains_key(&game_data.game_id) {
            panic!("Game not found")
        };

        self.games.insert(game_data.game_id, game_data);
    }

    pub fn get_game_data(&self, game_id: GameId) -> GameData {
        self.games.get(&game_id).expect("Game not found").clone()
    }

    pub fn vote_kick(&mut self, player_to_kick_id: AccountId, game_id: GameId) {
        let game_data = self.games.get_mut(&game_id).expect("Game not found");
        game_data.vote_kick(player_to_kick_id);
    }
}