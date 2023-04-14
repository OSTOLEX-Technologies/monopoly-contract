use std::collections::HashMap;
use near_sdk::{AccountId, near_bindgen};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::predecessor_account_id;
use near_sdk::serde::Serialize;
use crate::{GameId};

#[near_bindgen]
#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct GameConfig {
    players: String,
    current_player_id: AccountId,
    players_in_game: Vec<AccountId>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct GameData {
    pub game_id: GameId,
    players: String,
    current_offers: String,
    last_transactions: String,
    last_actions: String,
    current_player_id: AccountId,

    vote_kick: HashMap<AccountId, u8>,
    players_votes: HashMap<AccountId, Vec<AccountId>>,
    players_in_game: Vec<AccountId>,
}

impl GameData {
    pub fn new(game_config: &GameConfig, game_id: GameId) -> Self {
        Self {
            game_id,
            players: game_config.players.clone(),
            current_offers: "".to_string(),
            last_transactions: "".to_string(),
            last_actions: "".to_string(),
            current_player_id: game_config.current_player_id.clone(),
            vote_kick: HashMap::new(),
            players_votes: HashMap::new(),
            players_in_game: game_config.players_in_game.clone(),
        }
    }

    pub fn vote_kick(&mut self, player_to_kick_id: AccountId) {
        let account_id = predecessor_account_id();

        match self.players_votes.get_mut(&account_id) {
            None => { self.players_votes.insert(account_id, vec![player_to_kick_id]); },
            Some(player_votes) =>  {
                for player_id in player_votes.iter() {
                    if player_id.eq(&player_to_kick_id) {
                        panic!("You have already voted for this player")
                    }
                }

                player_votes.push(player_to_kick_id.clone());
                if player_votes.len() >= self.players_in_game.len() - 1 {
                    let mut idx = 0;
                    for player_id in self.players_in_game.iter() {
                        if player_id.eq(&player_to_kick_id) {
                            self.players_in_game.swap_remove(idx);
                            return;
                        }
                        idx += 1;
                    }
                }
            }
        };
    }
}