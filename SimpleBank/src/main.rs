#![allow(unused_variables)]

use std::str::FromStr;
use std::path::{PathBuf};
use std::fs::{File, OpenOptions};
use std::io::Write;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor};
use serde_json::{to_string};
use clap::Parser;

mod bank;
mod action;

use bank::{BankState, ErrorMsg};
use action::Action;

// CLI arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Log file to dump the state trace
    #[clap(short, long)]
    state_log_file: Option<PathBuf>,

    /// Log file to dump the state AND action trace
    #[clap(short, long)]
    action_log_file: Option<PathBuf>,
}

struct BankApp {
    state:      BankState,
    state_log:  Option<File>,
    action_log: Option<File>,
}

fn setup_logfile(buf: Option<PathBuf>) -> Option<File> {
    buf.map(|src| {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(src.into_os_string()).unwrap()
    })
}

impl BankApp {
    pub fn new(args: Args) -> Self {
        let state_log  = setup_logfile(args.state_log_file);
        let action_log = setup_logfile(args.action_log_file);

        BankApp {
            state: BankState::new(),
            state_log,
            action_log,
        }
    }

    pub fn log_state(&mut self) {
        if let Some(ref mut log) = self.state_log {
            writeln!(log, "{}", to_string(&self.state).unwrap()).unwrap();
        };
    }

    pub fn log_action(&mut self, action: Action) {
        if let Some(ref mut log) = self.action_log {
            writeln!(log, "{}", to_string(&action).unwrap()).unwrap();
        };
    }

    pub fn apply_action(&mut self, action: Action) -> Result<(), ErrorMsg> {
        let result =
            match action.clone() {
                Action::Deposit { depositor, amount }            => self.state.deposit(depositor, amount),
                Action::Withdraw { withdrawer, amount }          => self.state.withdraw(withdrawer, amount),
                Action::Transfer { sender, receiver, amount }    => self.state.transfer(sender, receiver, amount),
                Action::BuyInvestment { buyer, amount }          => self.state.buy_investment(buyer, amount),
                Action::SellInvestment { seller, investment_id } => self.state.sell_investment(seller, investment_id),
            }?;

        self.log_state();
        self.log_action(action);

        Ok(result)
    }

    pub fn repl(&mut self) -> rustyline::Result<()> {
        let mut rl = DefaultEditor::new()?;

        self.log_state();

        loop {
            let readline = rl.readline("> ");
            match readline {
                Ok(line) => {
                    match Action::from_str(&line) {
                        Ok(action) => {
                            rl.add_history_entry(&line)?;
                            match self.apply_action(action) {
                                Ok(()) => {
                                    println!("{:?}", self.state);
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
                    return Err(err);
                }
            }
        }

        Ok(())
    }
}

fn main() -> rustyline::Result<()> {
    let args = Args::parse();
    let mut app = BankApp::new(args);
    app.repl()
}
