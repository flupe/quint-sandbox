#!/usr/bin/env python3

import json
import functools
import operator
from jinja2 import Environment, FileSystemLoader

with open('log') as f:
    data = [json.loads(line) for line in f]

# bigint cleanup
# ==============
# This step is necessary because currently itf-rs doesn't provide nice helpers for generating traces
# Once the itf-rs library provides proper serializers, this could be skipped

for st in data:
    st['balances'] = {'tag': '#map', 'value': [(user, int(balance)) for (user, balance) in st['balances'].items()]}
    st['next_id']   = int(st['next_id'])

    invests = []
    for (k, v) in st['investments'].items():
        v['amount'] = int(v['amount'])
        invests.append((int(k), v))
    st['investments'] = {
        'tag': "#map",
        'value': invests
    }


# Pretty-print a Python value into a Quint expression
def quintify(value):
    if isinstance(value, dict):
        if 'tag' in value and value['tag'] == '#map':
            pairs = ["{} -> {}".format(quintify(k), quintify(v)) for (k, v) in value['value']]
            return "Map({})".format(", ".join(pairs))
        else:
            pairs = ["{}: {}".format(k, quintify(v)) for (k, v) in sorted(value.items())]
            return "{{{}}}".format(", ".join(pairs))
    elif isinstance(value, str):
        res = repr(value)
        if res.startswith("'"): # ensure double-quotes are used
            inner = res[1:-1].replace('"', '\\"').replace("\\'", "'")
            return '"{}"'.format(inner)
        return res
    elif isinstance(value, bool):
        return str(value).lower()
    else:
        return repr(value)

# Jinja2 environment, that:
# - looks up templates in the templates/ folder
# - adds a filter to pretty-print into Quint values
jenv = Environment(
    loader=FileSystemLoader("templates"),
)
jenv.filters['quintify'] = quintify

quint_template = jenv.get_template('trace.qnt')

print(quint_template.render(trace=data))
