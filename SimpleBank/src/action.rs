#![allow(dead_code)]
#![allow(unused_variables)]

use num_bigint::BigInt;
use std::str::FromStr;
use std::str::{SplitWhitespace};
use serde::{Serialize};
use serde_with_macros::serde_as;

use crate::helpers::ItfBigInt;

#[serde_as]
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "tag", content = "value")]
pub enum Action {
    Deposit {
        depositor: String,
        #[serde_as(serialize_as = "ItfBigInt")]
        amount: BigInt
    },
    Withdraw {
        withdrawer: String,
        #[serde_as(serialize_as = "ItfBigInt")]
        amount: BigInt
    },
    Transfer {
        sender: String,
        receiver: String,
        #[serde_as(serialize_as = "ItfBigInt")]
        amount: BigInt
    },
    BuyInvestment {
        buyer: String,
        #[serde_as(serialize_as = "ItfBigInt")]
        amount: BigInt
    },
    SellInvestment {
        seller: String,
        #[serde_as(serialize_as = "ItfBigInt")]
        investment_id: BigInt
    },
}

// parsing helpers
fn next_arg<'a>(s: &mut SplitWhitespace<'a>) -> Result<&'a str, String> {
    s.next().ok_or("Too few arguments".to_string())
}

fn no_more_args(s: &mut SplitWhitespace) -> Result<(), String> {
    if s.next().is_some() {
        Err("Too many arguments".to_string())
    }
    else {
        Ok(())
    }
}

impl FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();

        let cmd = parts.next().ok_or("Missing input".to_string())?;

        match cmd {
            "deposit" => {
                let depositor: String  = next_arg(&mut parts)?.to_string();
                let amount: BigInt     = next_arg(&mut parts)?
                    .parse().map_err(|_| "Invalid amount")?;

                no_more_args(&mut parts)?;

                Ok(Action::Deposit { depositor, amount })
            },
            "withdraw" => {
                let withdrawer: String  = next_arg(&mut parts)?.to_string();
                let amount: BigInt      = next_arg(&mut parts)?
                    .parse().map_err(|_| "Invalid amount")?;

                no_more_args(&mut parts)?;

                Ok(Action::Withdraw { withdrawer, amount })
            },
            "transfer" => {
                let sender: String   = next_arg(&mut parts)?.to_string();
                let receiver: String = next_arg(&mut parts)?.to_string();
                let amount: BigInt   = next_arg(&mut parts)?
                    .parse().map_err(|_| "Invalid amount")?;

                no_more_args(&mut parts)?;

                Ok(Action::Transfer { sender, receiver, amount })
            },
            "buy_investment" => {
                let buyer: String   = next_arg(&mut parts)?.to_string();
                let amount: BigInt  = next_arg(&mut parts)?
                    .parse().map_err(|_| "Invalid amount")?;

                no_more_args(&mut parts)?;

                Ok(Action::BuyInvestment { buyer, amount })
            },
            "sell_investment" => {
                let seller: String  = next_arg(&mut parts)?.to_string();
                let investment_id: BigInt  = next_arg(&mut parts)?
                    .parse().map_err(|_| "Invalid id")?;

                no_more_args(&mut parts)?;

                Ok(Action::SellInvestment { seller, investment_id })
            },
            _ => {
                Err(format!("Unknown command: {}", cmd))
            },
        }
    }
}
