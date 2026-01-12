#!/usr/bin/env python3

import json
import functools
import operator
import argparse

from collections import namedtuple
from itf_py      import value_from_json

action_lookup = {
  'Deposit':        'deposit_act',
  'Transfer':       'transfer_act',
  'Withdraw':       'withdraw_act',
  'BuyInvestment':  'buy_investment_act',
  'SellInvestment': 'sell_investment_act',
}

# Load a log file, parsing every line as an ITF-encoded value
def load_values(src):
    with open(src) as f:
        return [value_from_json(json.loads(line)) for line in f]

def to_quint_action(a):
    if a in action_lookup.keys():
        return action_lookup[a]
    else:
        return a

# Pretty-print an ITF encoded action into the corresponding Quint action call
def to_quint_action_call(action):
    params = map(quintify, action._asdict().values())
    return '{act}({params})'.format(
        act    = action_lookup[type(action).__name__],
        params = ", ".join(params)
            )

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
        # TODO: possibly handle _itf_variant
        fields = value._asdict()
        items = ['{}: {}'.format(k, quintify(v)) for k, v in fields.items()]
        return '{{{}}}'.format(', '.join(items))
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
        name= 'sequence',
        trace= trace
    )
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

    args = parser.parse_args()
    args.func(args)

if __name__ == '__main__':
    main()
