#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]

// #[cfg(test)]
pub mod tests {
    use itf::de::{self, As};
    use itf::trace_from_str;
    use num_bigint::BigInt;
    use serde::Deserialize;
    use std::fs;
    use simple_bank::bank::*;

    #[derive(Clone, Debug, Deserialize)]
    pub struct NondetPicks {
        #[serde(with = "As::<de::Option::<_>>")]
        pub depositor: Option<String>,

        #[serde(with = "As::<de::Option::<_>>")]
        pub withdrawer: Option<String>,

        #[serde(with = "As::<de::Option::<_>>")]
        pub sender: Option<String>,

        #[serde(with = "As::<de::Option::<_>>")]
        pub receiver: Option<String>,

        #[serde(with = "As::<de::Option::<_>>")]
        pub amount: Option<BigInt>,

        #[serde(with = "As::<de::Option::<_>>")]
        pub buyer: Option<String>,

        #[serde(with = "As::<de::Option::<_>>")]
        pub seller: Option<String>,

        #[serde(with = "As::<de::Option::<_>>")]
        pub id: Option<BigInt>,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct State {
        pub bank_state: BankState,
        #[serde(with = "As::<de::Option::<_>>")]
        pub error: Option<String>,
        #[serde(rename = "mbt::actionTaken")]
        pub action_taken: String,
        #[serde(rename = "mbt::nondetPicks")]
        pub nondet_picks: NondetPicks,
    }

    fn compare_error(trace_error: Option<String>, app_error: Result<(), String>) {
        if trace_error.is_some() {
            assert!(
                app_error.is_err(),
                "Expected action to fail with error: {:?}, but it succeeded",
                trace_error
            );
            println!("Action failed as expected");
        } else {
            assert!(
                app_error.is_ok(),
                "Expected action to succeed, but it failed with error: {:?}",
                app_error
            );
            println!("Action successful as expected");
        }
    }

    // Data-driven test cases using Quint-exported traces
    #[datatest::files("traces", { input in r"out(.*)\.itf\.json" })]
    #[test]
    fn check_trace(input: &str) {
        let trace: itf::Trace<State> = trace_from_str(input).unwrap();

        let mut bank_state = trace.states[0].value.bank_state.clone();

        for state in trace.states {
            let action_taken = state.value.action_taken;
            let picks        = state.value.nondet_picks;

            match action_taken.as_str() {
                "init" => { println!("initializing"); }

                "deposit_action" => {
                    let depositor = picks.depositor.clone().unwrap();
                    let amount    = picks.amount.clone().unwrap();
                    println!("deposit({}, {})", depositor, amount);

                    let res = bank_state.deposit(depositor, amount);
                    compare_error(state.value.error.clone(), res)
                }
                "withdraw_action" => {
                    let withdrawer = picks.withdrawer.clone().unwrap();
                    let amount     = picks.amount.clone().unwrap();
                    println!("withdraw({}, {})", withdrawer, amount);

                    let res = bank_state.withdraw(withdrawer, amount);
                    compare_error(state.value.error.clone(), res)
                }
                "transfer_action" => {
                    let sender   = picks.sender.clone().unwrap();
                    let receiver = picks.receiver.clone().unwrap();
                    let amount   = picks.amount.clone().unwrap();
                    println!("transfer({}, {}, {})", sender, receiver, amount);

                    let res = bank_state.transfer(sender, receiver, amount);
                    compare_error(state.value.error.clone(), res)
                }
                "buy_investment_action" => {
                    let buyer  = picks.buyer.clone().unwrap();
                    let amount = picks.amount.clone().unwrap();
                    println!("buy_investment({}, {})", buyer, amount);

                    let res = bank_state.buy_investment(buyer, amount);
                    compare_error(state.value.error.clone(), res)
                }
                "sell_investment_action" => {
                    let seller = picks.seller.clone().unwrap();
                    let id     = picks.id.clone().unwrap();
                    println!("sell_investment({}, {})", seller, id);

                    let res = bank_state.sell_investment(seller, id);
                    compare_error(state.value.error.clone(), res)
                }

                action => panic!("Invalid action taken {}", action),
            }
        }
    }
}
