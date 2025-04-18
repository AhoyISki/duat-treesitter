(string) @string

(escape_sequence) @string.escape

(capture
  (identifier) @type)

(predicate
  name: (identifier) @function.call)

(named_node
  name: (identifier) @variable)

(field_definition
  name: (identifier) @variable.member)

(negated_field
  "!" @operator
  (identifier) @property)

(comment) @comment @spell

(quantifier) @operator

(predicate_type) @punctuation.special

"." @operator

[
  "["
  "]"
  "("
  ")"
] @punctuation.bracket

[
  ":"
  "/"
] @punctuation.delimiter

[
  "@"
  "#"
] @punctuation.special

"_" @character.special

((parameters
  (identifier) @number)
  (#match? @number "^[-+]?[0-9]+(.[0-9]+)?$"))

((program
  .
  (comment)*
  .
  (comment) @keyword.import @nospell)
  (#match? @keyword.import "^;+ *inherits *:"))

((program
  .
  (comment)*
  .
  (comment) @keyword.directive @nospell)
  (#match? @keyword.directive "^;+ *extends *$"))

((comment) @keyword.directive @nospell
  (#match? @keyword.directive "^;+%s*format%-ignore%s*$"))

((predicate
  name: (identifier) @_name
  parameters: (parameters
    .
    (capture)?
    .
    (identifier) @property))
  (#eq? @_name "set"))

((predicate
  name: (identifier) @_name
  parameters: (parameters
    (string
      "\"" @string
      "\"" @string) @string.regexp))
  (#any-of? @_name "match" "not-match" "vim-match" "not-vim-match" "match" "not-match"))

((predicate
  name: (identifier) @_name
  parameters: (parameters
    (string
      "\"" @string
      "\"" @string) @string.regexp
    .
    (string) .))
  (#any-of? @_name "gsub" "not-gsub"))
