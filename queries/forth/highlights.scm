(core) @function

(operator) @operator

(word) @variable

((word) @constant
  (#match? @constant "^[A-Z_]+$"))

(number) @number

(string) @string

[
  (start_definition)
  (end_definition)
] @punctuation.delimiter

(comment) @comment @spell
