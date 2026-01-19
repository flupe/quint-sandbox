# Quint Conformance Testing Example

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

## Building the project

### Rust setup

Ensure you're using Nightly Rust.

```bash
rustup toolchain install nightly
rustup default nightly
```

The project should compile with `cargo build`.

### Python setup

1. Ensure you have [Python][python] and [pip][pip] up and running
2. Setup a virtual environment
3. Install the dependencies locally ([`itf-py`][itf-py])

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

[python]: https://www.python.org/
[pip]: https://pip.pypa.io/en/stable/
[itf-py]: https://github.com/konnov/itf-py

Then the Quint code generator is ready to be used! Simply type `./genquint.py` 
with the appropriate arguments.

## Running conformance tests

### Model-Based Testing (Quint -> Rust)

Running `./test.sh` will run simulations using the Quint model, and export the
generated traces in the `traces/` folder. This same script will immediately call
`cargo test` to check that all these traces execute fine on the Rust
implementation and states match.

### Conformance completeness (Rust -> Quint)

#### Extracting traces from the real system

When going the other direction, the purpose is to extract traces of interest observed in the real system
and import them. There are many valid ways to generate traces: from a pool of existing recorded scenarii, from randomly generated traces (e.g. from a property-based testing infrastructure), or from live monitoring.

In this demo, we decided to produce logs from an interactive session with the Rust aplication.

```bash
cargo run -- --help
```

```
Usage: simple_bank [OPTIONS]

Options:
  -s, --state-log-file <STATE_LOG_FILE>    Log file to dump the state trace
  -a, --action-log-file <ACTION_LOG_FILE>  Log file to dump the state AND action trace
  -h, --help                               Print help
  -V, --version                            Print version
```

Two kinds of log files can be produced:
- `-s`: one containing a trace of all the intermediate states of the application.

   ```
   {"balances":{"#map":[]},"investments":{"#map":[]},"next_id":{"#bigint":"0"}}
   {"balances":{"#map":[["bob",{"#bigint":"20"}]]},"investments":{"#map":[]},"next_id":{"#bigint":"0"}}
   {"balances":{"#map":[["alice",{"#bigint":"10"}],["bob",{"#bigint":"10"}]]},"investments":{"#map":[]},"next_id":{"#bigint":"0"}}
   {"balances":{"#map":[["alice",{"#bigint":"10"}],["bob",{"#bigint":"0"}]]},"investments":{"#map":[[{"#bigint":"0"},{"owner":"bob","amount":{"#bigint":"10"}}]]},"next_id":{"#bigint":"1"}}
   {"balances":{"#map":[["alice",{"#bigint":"10"}],["bob",{"#bigint":"10"}]]},"investments":{"#map":[]},"next_id":{"#bigint":"1"}}
   ```
- `-a`: one containing the trace of all actions that have been observed.

   ```
   {"tag":"Deposit","value":{"depositor":"bob","amount":{"#bigint":"20"}}}
   {"tag":"Transfer","value":{"sender":"bob","receiver":"alice","amount":{"#bigint":"10"}}}
   {"tag":"BuyInvestment","value":{"buyer":"bob","amount":{"#bigint":"10"}}}
   {"tag":"SellInvestment","value":{"seller":"bob","investment_id":{"#bigint":"0"}}}
   ```

In the following, we expect logs to have been generated under the following names:

```bash
cargo run -- -s log -a act
```

#### Importing them in Quint

The `genquint.py` script generates Quint modules on demand, from extracted traces.
We showcase 3 flavors of generated modules.

```bash
./genquint.py --help
```

```
usage: genquint.py [-h] {trace,run,replay} ...
genquint.py: error: the following arguments are required: {trace,run,replay}
```

1. Trace generation.

   The model is modified to store a log of all its intermediate states (See `bank.qnt:log`).
   We generate a module that contains the Quint equivalent of the recorded trace.
   This module also contains an invariant stating that the trace should not be reachable.
   We expect the model-checker to find a counterexample for this invariant, proving that this trace 
   is possible in our model.

   ```bash
   ./genquint.py trace log > trace.qnt
   ```

   ```quint
   // This Quint module was generated. Do NOT edit manually.
   module trace {
     import bank.* from "./bank"
   
     // Generated trace from an actual execution in the Rust implementation
     val observed_trace: List[BankState] =
       [{balances: Map(), investments: Map(), next_id: 0}, {balances: Map("bob" -> 20), investments: Map(), next_id: 0}, {balances: Map("alice" -> 10, "bob" -> 10), investments: Map(), next_id: 0}, {balances: Map("alice" -> 10, "bob" -> 0), investments: Map(0 -> {owner: "bob", amount: 10}), next_id: 1}, {balances: Map("alice" -> 10, "bob" -> 10), investments: Map(), next_id: 1}]
   
     // Invariant stating that such a trace cannot be observed.
     // This invariant should NOT hold: we expect the model-checker to exhibit
     // a sequence of states matching the original trace.
     val unreachableTrace = log != observed_trace
   }
   ```

   ```bash
   quint verify trace.qnt --invariant unreachableTrace
   ```

   ```
   [violation] Found an issue (6092ms).
   error: found a counterexample
   ```

   Note that this kind of generated module is only suitable for short traces.
   The state space grows exponentially as the trace length increases.

2. Run generation.

   We alternatively generate proper Quint *runs*.

   ```bash
   ./genquint.py run log act > sequence.qnt
   ```

   ```quint
  // This Quint module was generated. Do NOT edit manually.
  module sequence {
    import bank.* from "./bank"
  
    // Generated run from an actual execution in the Rust implementation
    run observed_trace: bool =
      init
        .then(enforced(deposit_act("bob", 20)))
        .expect(bank_state == {balances: Map("bob" -> 20), investments: Map(), next_id: 0})
        .then(enforced(transfer_act("bob", "alice", 10)))
        .expect(bank_state == {balances: Map("alice" -> 10, "bob" -> 10), investments: Map(), next_id: 0})
        .then(enforced(buy_investment_act("bob", 10)))
        .expect(bank_state == {balances: Map("alice" -> 10, "bob" -> 0), investments: Map(0 -> {owner: "bob", amount: 10}), next_id: 1})
        .then(enforced(sell_investment_act("bob", 0)))
        .expect(bank_state == {balances: Map("alice" -> 10, "bob" -> 10), investments: Map(), next_id: 1})
  }
   ```

   ```bash
   quint test sequence.qnt --match=observed_trace
   ```

   ```
   sequence
       ok observed_trace passed 1 test(s)

     1 passing (11ms)
   ```

   The generation script converts actions parsed in the log into their Quint equivalent defined
   in the model. See the top of the Python file to find/edit the lookup table.

3. Replay.

   It is not uncommon for Quint models to have in the state a list of (reified) actions
   that have to be taken first before random steps take over.

   In the model, this is the role of the `replay` variable.
   In the `step` function, unless this list is empty, we take the next forced action and apply it.
   Otherwise, we take a random action.


   ```bash
   ./genquint.py replay log act > replay.qnt
   ```

   ```quint
   // This Quint module was generated. Do NOT edit manually.
   module replay {
     import bank.* from "./bank"
   
     // Sequence of actions extracted from a real execution
     val actions: List[Action] =
       [Deposit({depositor: "bob", amount: 20}), Transfer({sender: "bob", receiver: "alice", amount: 10}), BuyInvestment({buyer: "bob", amount: 10}), SellInvestment({seller: "bob", investment_id: 0})]
   
     run checkTrace: bool =
       initWithForced(actions)
         .then(4.reps(_ => step))
         .expect(bank_state == {balances: Map("alice" -> 10, "bob" -> 10), investments: Map(), next_id: 1})
   
   }
   ```

   ```bash
   quint test replay.qnt --match=checkTrace
   ```

   ```
   replay
       ok checkTrace passed 1 test(s)

     1 passing (12ms)
   ```
