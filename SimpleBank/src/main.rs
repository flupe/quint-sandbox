#![allow(dead_code)]
#![allow(unused_variables)]

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor};
use std::str::FromStr;
// use serde::{Serialize};

mod bank;
mod action;

use bank::{BankState, ErrorMsg};
use action::Action;

struct BankApp {
    pub state: BankState,
}

impl BankApp {
    pub fn new() -> Self {
        BankApp { state: BankState::new() }
    }

    pub fn apply_action(&mut self, action: Action) -> Result<(), ErrorMsg> {
        match action {
            Action::Deposit { depositor, amount }            => self.state.deposit(depositor, amount),
            Action::Withdraw { withdrawer, amount }          => self.state.withdraw(withdrawer, amount),
            Action::Transfer { sender, receiver, amount }    => self.state.transfer(sender, receiver, amount),
            Action::BuyInvestment { buyer, amount }          => self.state.buy_investment(buyer, amount),
            Action::SellInvestment { seller, investment_id } => self.state.sell_investment(seller, investment_id),
        }
    }
}

fn main() -> rustyline::Result<()> {
    let mut app = BankApp::new();
    let mut rl = DefaultEditor::new()?;

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                match Action::from_str(&line) {
                    Ok(action) => {
                        match app.apply_action(action) {
                            Ok(()) => {
                                println!("{:?}", app.state);
                            }
                            Err(err) => {
                                println!("Could not apply action: {}", err);
                            }
                        };
                    },
                    Err(err) => {
                        println!("Error: {}", err);
                    },
                }
            },
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("Exiting...");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }

    Ok(())
}
