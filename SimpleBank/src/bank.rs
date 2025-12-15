#![allow(dead_code)]
#![allow(unused_variables)]

use num_bigint::BigInt;
use serde::{Deserialize};
use std::collections::HashMap;

pub type ErrorMsg = String;

#[derive(Clone, Debug, Deserialize)]
pub struct Investment {
    pub owner: String,
    pub amount: BigInt,
}

// NOTE(flupe):
//  currently, the state is an exact copy of the Quint state (or the converse)
//  we could imagine having a different representation for the Rust state
//  and an equivalence relation instead.
#[derive(Clone, Debug, Deserialize)]
pub struct BankState {
    pub balances: HashMap<String, BigInt>,
    pub investments: HashMap<BigInt, Investment>,
    pub next_id: BigInt,
}


impl BankState {
    pub fn new() -> Self {
        BankState {
            balances:    HashMap::new(),
            investments: HashMap::new(),
            next_id:     BigInt::from(0),
        }
    }

    pub fn deposit(&mut self, depositor: String, amount: BigInt) -> Result<(), ErrorMsg> {
        if amount <= BigInt::from(0) {
            return Err("Amount should be greater than zero".to_string());
        }

        self.balances.entry(depositor)
            .and_modify(|curr| *curr += amount.clone())
            .or_insert(amount);

        Ok(())
    }

    pub fn withdraw(&mut self, withdrawer: String, amount: BigInt) -> Result<(), ErrorMsg> {
        if amount <= BigInt::from(0) {
            return Err("Amount should be greater than zero".to_string());
        }

        let balance = self.balances.get(&withdrawer)
                          .ok_or(format!("Could not find withdrawer {}", withdrawer))?;

        if balance < &amount {
            return Err("Balance is too low".to_string());
        }

        self.balances
            .entry(withdrawer)
            .and_modify(|curr| *curr -= amount);

        Ok(())
    }

    pub fn transfer(&mut self, sender: String, receiver: String, amount: BigInt) -> Result<(), ErrorMsg> {
        if amount <= BigInt::from(0) {
            return Err("Amount should be greater than zero".to_string());
        }

        let balance = self.balances.get(&sender)
                          .ok_or(format!("Could not find sender {}", sender))?;

        if balance < &amount {
            return Err("Balance is too low".to_string());
        }

        self
            .balances
            .entry(sender)
            .and_modify(|curr| *curr -= amount.clone());

        self
            .balances
            .entry(receiver)
            .and_modify(|curr| *curr += amount.clone())
            .or_insert(amount);

        Ok(())
    }

    pub fn buy_investment(&mut self, buyer: String, amount: BigInt) -> Result<(), ErrorMsg> {
        if amount <= BigInt::from(0) {
            return Err("Amount should be greater than zero".to_string());
        }

        let balance = self.balances.get(&buyer)
                          .ok_or(format!("Could not find buyer {}", buyer))?;

        if balance < &amount {
            return Err("Balance is too low".to_string());
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

        Ok(())
    }

    pub fn sell_investment(&mut self, seller: String, investment_id: BigInt) -> Result<(), ErrorMsg> {
        if let Some(investment) = self.investments.get(&investment_id) {
            if investment.owner != seller {
                return Err("Seller can't sell an investment they don't own".to_string());
            }
            self
                .balances
                .entry(seller)
                .and_modify(|curr| *curr += investment.amount.clone());

            self.investments.remove(&investment_id);

            Ok(())
        }
        else {
            Err("No investment with this id".to_string())
        }
    }

}
