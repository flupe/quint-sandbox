#!/usr/bin/env python3

import json
import functools
import operator
import argparse

from collections import namedtuple
from itf_py      import value_from_json

# Lookup table for matching action constructors with their respective Quint action names.
action_lookup = {
  'Deposit':        'deposit_act',
  'Transfer':       'transfer_act',
  'Withdraw':       'withdraw_act',
  'BuyInvestment':  'buy_investment_act',
  'SellInvestment': 'sell_investment_act',
}

# Pretty-print an ITF encoded action into the corresponding Quint action call
# e.g Deposit(depositor = "bob", amount = 15) will become 'deposit_act("bob", 15)'
def to_quint_action_call(action):
    params = map(quintify, action._asdict().values())
    return '{act}({params})'.format(
        act    = action_lookup[type(action).__name__],
        params = ", ".join(params))

# Load a log file, parsing every line as an ITF-encoded value
def load_values(src):
    with open(src) as f:
        return [value_from_json(json.loads(line)) for line in f]

# Pretty-print a Python ITF value into a Quint expression
def quintify(value):
    # simple dicts become Quint Maps
    if isinstance(value, dict):
        items = ['{} -> {}'.format(quintify(k), quintify(v)) for k, v in value.items()]
        return 'Map({})'.format(', '.join(items))

    # strings have to use double quotes
    elif isinstance(value, str):
        res = repr(value)
        if res.startswith("'"): # ensure double-quotes are used
            inner = res[1:-1].replace('"', '\\"').replace("\\'", "'")
            return '"{}"'.format(inner)
        return res

    # booleans are lowercase
    elif isinstance(value, bool):
        return str(value).lower()

    # lists syntax is identical
    elif isinstance(value, list):
        items = map(quintify, value)
        return "[{}]".format(", ".join(items))

    # named tuples become Quint records
    elif isinstance(value, tuple) and hasattr(value, "_fields"):
        fields = value._asdict()
        items = ['{}: {}'.format(k, quintify(v)) for k, v in fields.items()]
        args = '{{{}}}'.format(', '.join(items))

        # if the value is a variant, put the variant name
        if hasattr(value, '_itf_variant') and value._itf_variant:
            return '{}({})'.format(type(value).__name__, args)

        # otherwise, it's just a record
        else:
            return args

    # for everything else, just use Python repr pretty-printing (e.g int)
    else:
        return repr(value)


# Generate a Quint file that contains the log of states,
# along with an invariant that checks that the log is unreachable.
def gen_trace(args):
    states = load_values(args.state_file)
    code = '''// This Quint module was generated. Do NOT edit manually.
module {name} {{
  import bank.* from "./bank"

  // Generated trace from an actual execution in the Rust implementation
  val observed_trace: List[BankState] =
    {trace}

  // Invariant stating that such a trace cannot be observed.
  // This invariant should NOT hold: we expect the model-checker to exhibit
  // a sequence of states matching the original trace.
  val unreachableTrace = log != observed_trace
}}'''.format(
        name= 'trace',
        trace= quintify(states)
    )
    print(code)

# Generate a Quint file that defines a run with all the expected transitions.
# We expect all intermediate states to match.
def gen_run(args):
    states  = load_values(args.state_file)
    actions = load_values(args.action_file)

    # TODO: sanity checks that there are enough actions?

    trace = ""

    for action, state in zip(actions, states[1:]):
        trace += "\n      .then({action})\n      .expect(bank_state == {st})".format(
                   action = to_quint_action_call(action),
                   st     = quintify(state))

    code = '''// This Quint module was generated. Do NOT edit manually.
module {name} {{
  import bank.* from "./bank"

  // Generated run from an actual execution in the Rust implementation
  run observed_trace: bool =
    init{trace}

}}'''.format(
        name  = 'sequence',
        trace = trace)

    print(code)

# Generate a Quint file that produces the list of reified transitions.
# Also produce a run that calls `next` repeatedly, and checks for final state compliance.
def gen_replay(args):
    states  = load_values(args.state_file)
    actions = load_values(args.action_file)

    code = '''// This Quint module was generated. Do NOT edit manually.
module {name} {{
  import bank.* from "./bank"

  // Sequence of actions extracted from a real execution
  val actions: List[Action] =
    {actions}

  run checkTrace: bool =
    initWithForced(actions)
      .then({steps}.reps(_ => step))
      .expect(bank_state == {final_state})

}}'''.format(
        name        = 'replay',
        actions     = quintify(actions),
        steps       = len(actions),
        final_state = quintify(states[-1]))

    print(code)


# CLI parsing
def main():
    parser = argparse.ArgumentParser(
      description='''Quint generator for converting Rust traces
                     into Quint executions''')

    subparsers = parser.add_subparsers(
      title='subcommands',
      description='generation targets',
      required=True)

    trace_parser = subparsers.add_parser('trace')
    trace_parser.add_argument("state_file",  help="log file for state trace")
    trace_parser.set_defaults(func=gen_trace)

    run_parser = subparsers.add_parser('run')
    run_parser.add_argument("state_file",  help="log file for state trace")
    run_parser.add_argument("action_file", help="log file for action trace")
    run_parser.set_defaults(func=gen_run)

    replay_parser = subparsers.add_parser('replay')
    replay_parser.add_argument("state_file",  help="log file for state trace")
    replay_parser.add_argument("action_file", help="log file for action trace")
    replay_parser.set_defaults(func=gen_replay)

    args = parser.parse_args()
    args.func(args)

if __name__ == '__main__':
    main()
