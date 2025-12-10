use num_bigint::BigInt;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct Investment {
    pub owner: String,
    pub amount: BigInt,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BankState {
    pub balances: HashMap<String, BigInt>,
    pub investments: HashMap<BigInt, Investment>,
    pub next_id: BigInt,
}

impl BankState {
    pub fn deposit(&mut self, depositor: String, amount: BigInt) -> Option<String> {
        if amount <= BigInt::from(0) {
            return Some("Amount should be greater than zero".to_string());
        }

        self
            .balances
            .entry(depositor)
            .and_modify(|curr| *curr += amount);

        None
    }

    pub fn withdraw(&mut self, withdrawer: String, amount: BigInt) -> Option<String> {
        if amount <= BigInt::from(0) {
            return Some("Amount should be greater than zero".to_string());
        }

        if self.balances.get(&withdrawer).unwrap() < &amount {
            return Some("Balance is too low".to_string());
        }

        self
            .balances
            .entry(withdrawer)
            .and_modify(|curr| *curr -= amount);

        None
    }

    pub fn transfer(&mut self, sender: String, receiver: String, amount: BigInt) -> Option<String> {
        if amount <= BigInt::from(0) {
            return Some("Amount should be greater than zero".to_string());
        }

        if self.balances.get(&sender).unwrap() < &amount {
            return Some("Balance is too low".to_string());
        }

        self
            .balances
            .entry(sender)
            .and_modify(|curr| *curr -= amount.clone());

        self
            .balances
            .entry(receiver)
            .and_modify(|curr| *curr += amount);

        None
    }

    pub fn buy_investment(&mut self, buyer: String, amount: BigInt) -> Option<String> {
        if amount <= BigInt::from(0) {
            return Some("Amount should be greater than zero".to_string());
        }

        if self.balances.get(&buyer).unwrap() < &amount {
            return Some("Balance is too low".to_string());
        }

        self
            .balances
            .entry(buyer.clone())
            .and_modify(|curr| *curr -= amount.clone());

        self.investments.insert(
            self.next_id.clone(),
            Investment {
                owner: buyer,
                amount,
            },
        );

        self.next_id += 1;

        None
    }

    pub fn sell_investment(&mut self, seller: String, investment_id: BigInt) -> Option<String> {
        if let Some(investment) = self.investments.get(&investment_id) {
            if investment.owner != seller {
                return Some("Seller can't sell an investment they don't own".to_string());
            }
            self
                .balances
                .entry(seller)
                .and_modify(|curr| *curr += investment.amount.clone());

            self.investments.remove(&investment_id);

            None
        }
        else {
            Some("No investment with this id".to_string())
        }
    }
}





