# stream-monitor
A monitor for type checking the output of commands in shell scripts. With a streamed input and access to a specially serialized DFA, efficiently runs a line by line check of the input over the DFA, panicking upon a failed match and streaming each line along upon a successful one.
