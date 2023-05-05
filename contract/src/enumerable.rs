use crate::*;

#[near_bindgen]
impl Contract {
    pub fn get_game_data(&self, game_id: GameId) -> GameData {
        self.games.get(&game_id).expect("Game not found").clone()
    }

    pub fn get_game_id(&self, account_id: AccountId) -> Option<GameId> {
        self.game_per_account_id.get(&account_id).expect("Account not found").clone()
    }
}