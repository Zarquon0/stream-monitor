import json
from automata.fa.dfa import DFA
from automata.fa.gnfa import GNFA


def byte_to_char(b):
    b = int(b)
    # printable ASCII range
    if 32 <= b <= 126:
        return chr(b)
    return f'\\x{b:02x}'

def load_dfa_from_json(path_to_json):
    with open(path_to_json, 'r') as f:
        data = json.load(f)

    start_state = data["start_state"]
    final_states = set(data["match_states"])
    transition_table = data["transition_table"]

    all_bytes = {str(b) for b in range(256)}
    all_chars = {byte_to_char(b) for b in all_bytes}
    transitions = {}
    states = set()
    states.add('0')  # Dead state

    # Build all valid transitions
    for entry in transition_table:
        curr = entry["curr_state"]
        r_start = int(entry["range_start"])
        r_end = int(entry["range_end"])
        next_state = entry["next_state"]

        states.update([curr, next_state])
        if curr not in transitions:
            transitions[curr] = {}

        for byte in range(r_start, r_end + 1):
            char_symbol = byte_to_char(str(byte))
            transitions[curr][char_symbol] = next_state

    # Fill in missing transitions to dead state
    for state in states:
        if state not in transitions:
            transitions[state] = {}
        defined_symbols = set(transitions[state].keys())
        undefined_symbols = all_chars - defined_symbols
        for symbol in undefined_symbols:
            transitions[state][symbol] = '0'

    # Dead state loops to itself
    transitions['0'] = {s: '0' for s in all_chars}

    dfa = DFA(
        states=states,
        input_symbols=all_chars,
        transitions=transitions,
        initial_state=start_state,
        final_states=final_states
    )

    return dfa

def convert_dfa_to_regex(dfa):
    gnfa = GNFA.from_dfa(dfa)
    regex = gnfa.to_regex()
    return regex

if __name__=="__main__":
    print()