This folder contains an example of conformance testing using Quint.

We have the following two components:

1. A Quint model that describes a *simplistic banking system*.
  Users can deposit, transfer and withdraw money.
  They can also buy and sell investments (they own).

2. A simple Rust implementation of the same system. The banking operations are
   implemented as a simple transition system (See `src/bank.rs`). There is a
   `BankState`, on which all the operations are implemented, modifying the state
   in place and returning success information. On a separate layer, available
   actions are reified into an enum, and we implemented a REPL that parses
   commands and applies them to the underlying banking state machine (See
   `src/action.rs` and `src/main.rs`).

Conformance testing is carried out in two ways:

- The Quint model is used to generate random (valid) execution traces.
  Those traces are imported on the Rust side and executed on the real
  implementation, checking that the states we obtain after every steps
  corresponds to the one extracted from Quint.
  This is often referred to as Model-Based Testing (MBT).

  See `tests/bank_mbt.rs` to see how this is setup, along with `test.sh` for the
  generation of traces.


> [!IMPORTANT]  
> A new official methodology has since been released by the Quint team:
> [Quint Connect](https://github.com/informalsystems/quint-connect).

- Going the other direction, we showcase a simple methodology.
  The Rust app is setup so that it can export real execution traces, 
  that can then be converted into Quint code containing the observed execution trace.
  Quint can be used to ensure that the Quint model can exhibit such an
  execution trace.

  Checking that the real execution can be observed from the Quint model can be
  carried out using different strategies:

  - The simplest: we extend the Quint model state with a log variable that
    stores the trace of all intermediate states. After importing the state trace 
    from a real Rust execution, we run the model checker with invariant "the
    state trace log isn't equal to the exported trace". If the model checker
    finds a counter-example: all good! This means there exists a model execution
    that exhibits exactly what was observed in the real system.

  - Using [Quint *runs*][runs].

  - Using transition replay. We adapt the Quint model so that the system state holds
    a list of forced transitions that should happen before random exploration.

[runs]: https://quint-lang.org/docs/lang#runs

