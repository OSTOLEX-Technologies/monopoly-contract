use crate::storage_tracker::StorageTracker;
use crate::*;
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::json_types::U128;
use near_sdk::serde::Serialize;
use near_sdk::{env, require, Balance, Promise, StorageUsage};

pub const MIN_STORAGE_BYTES: StorageUsage = 2000;
const MIN_STORAGE_BALANCE: Balance = MIN_STORAGE_BYTES as Balance * env::STORAGE_PRICE_PER_BYTE;

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Account {
    pub storage_balance: Balance,
    pub used_bytes: StorageUsage,
    #[serde(skip)]
    #[borsh_skip]
    pub storage_tracker: StorageTracker,
}

impl Account {
    pub fn new() -> Self {
        Self {
            storage_balance: 0,
            used_bytes: 0,
            storage_tracker: Default::default(),
        }
    }

    pub fn start_storage_tracker(&mut self) {
        self.storage_tracker.start();
    }

    pub fn stop_storage_tracker(&mut self) {
        self.storage_tracker.stop();
    }

    fn assert_storage_covered(&self) {
        let storage_balance_needed = Balance::from(self.used_bytes) * env::storage_byte_cost();
        assert!(
            storage_balance_needed <= self.storage_balance,
            "Not enough storage balance"
        );
    }
}

impl Contract {
    pub fn internal_get_account(&self, account_id: &AccountId) -> Account {
        self.accounts
            .get(account_id)
            .expect("Account not found")
            .clone()
    }

    pub fn internal_unwrap_account_or_create(
        &mut self,
        account_id: &AccountId,
        storage_deposit: Balance,
    ) -> Account {
        require!(
            env::is_valid_account_id(account_id.as_bytes()),
            "Invalid account id"
        );

        return if !self.accounts.contains_key(account_id) {
            self.internal_create_account(account_id, storage_deposit, false);
            self.internal_get_account(account_id)
        } else {
            let mut account: Account = self.internal_get_account(account_id);
            account.storage_balance += storage_deposit;
            account
        };
    }

    pub fn internal_create_account(
        &mut self,
        account_id: &AccountId,
        storage_deposit: Balance,
        registration_only: bool,
    ) {
        let min_balance = self.storage_balance_bounds().min.0;
        if storage_deposit < min_balance {
            env::panic_str("The attached deposit is less than the minimum storage balance");
        }

        let mut account = Account::new();
        if registration_only {
            let refund = storage_deposit - min_balance;
            if refund > 0 {
                Promise::new(predecessor_account_id()).transfer(refund);
            }
            account.storage_balance = min_balance;
        } else {
            account.storage_balance = storage_deposit;
        }

        self.internal_set_account(account_id, account);
    }

    pub fn internal_set_account(&mut self, account_id: &AccountId, mut account: Account) -> bool {
        if account.storage_tracker.bytes_added > account.storage_tracker.bytes_released {
            let extra_bytes_used =
                account.storage_tracker.bytes_added - account.storage_tracker.bytes_released;
            account.used_bytes += extra_bytes_used;
            account.assert_storage_covered();
        } else if account.storage_tracker.bytes_added < account.storage_tracker.bytes_released {
            let bytes_released =
                account.storage_tracker.bytes_released - account.storage_tracker.bytes_added;
            assert!(
                account.used_bytes >= bytes_released,
                "Internal storage accounting bug"
            );
            account.used_bytes -= bytes_released;
        }
        account.storage_tracker.bytes_released = 0;
        account.storage_tracker.bytes_added = 0;
        self.accounts
            .insert(account_id.clone(), account.into())
            .is_some()
    }
}

#[near_bindgen]
impl StorageManagement for Contract {
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let user = account_id.unwrap_or(env::predecessor_account_id());
        let registration_only = registration_only.unwrap_or(false);
        let attached_deposit = env::attached_deposit();
        let min_balance = self.storage_balance_bounds().min;

        if attached_deposit < min_balance.0 {
            env::panic_str("Deposit is less than the minimum storage balance");
        }

        let mut account = self.internal_unwrap_account_or_create(&user, attached_deposit);

        if registration_only {
            let refund = attached_deposit - min_balance.0;
            if refund > 0 {
                Promise::new(env::predecessor_account_id()).transfer(refund);
            }
            account.storage_balance = min_balance.0;
        } else {
            account.storage_balance = attached_deposit;
        }

        self.internal_set_account(&user, account);

        self.storage_balance_of(user).unwrap()
    }

    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        let user = env::predecessor_account_id();
        let amount_to_withdraw = amount.unwrap().0;

        let mut account = self.internal_get_account(&user);
        let storage_balance = account.storage_balance;
        
        // Assume that we need at least storage_balance_bounds at storage_balance after operation
        // TODO: check if remaining_balance < self.storage_balance_bounds().min.0 or < 0
        if storage_balance < amount_to_withdraw {
            env::panic_str("Insufficient storage balance");
        }

        let remaining_balance = storage_balance - amount_to_withdraw;

        account.storage_balance = remaining_balance;
        self.internal_set_account(&user, account);

        Promise::new(env::predecessor_account_id()).transfer(amount_to_withdraw);
        
        self.storage_balance_of(user).unwrap()
    }

    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        let user = env::predecessor_account_id();
        let force_unregister = force.unwrap_or(false);
        let account = self.internal_get_account(&user);

        if !force_unregister && account.used_bytes > 0 {
            env::panic_str("Cannot unregister storage. Storage must be empty or use force");
        }
        
        self.accounts.remove(&user);

        return true;
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        StorageBalanceBounds {
            min: U128(MIN_STORAGE_BALANCE),
            max: None,
        }
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        let account = self.internal_get_account(&account_id);
        
        Some(StorageBalance {
                total: account.storage_balance.into(),
                available: U128((account.storage_balance - 
                    Balance::from(account.used_bytes)) * env::storage_byte_cost(),
                ),
            })
    }

}
