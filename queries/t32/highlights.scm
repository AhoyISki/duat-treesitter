; Keywords, punctuation and operators
[
  "="
  "^^"
  "||"
  "&&"
  "+"
  "-"
  "*"
  "/"
  "%"
  "|"
  "^"
  "=="
  "!="
  ">"
  ">="
  "<="
  "<"
  "<<"
  ">>"
  ".."
  "--"
  "++"
  "+"
  "-"
  "~"
  "!"
  "&"
  "->"
  "*"
  "-="
  "+="
  "*="
  "/="
  "%="
  "|="
  "&="
  "^="
  ">>="
  "<<="
  "--"
  "++"
] @operator

[
  "("
  ")"
  "{"
  "}"
  "["
  "]"
] @punctuation.bracket

[
  ","
  "."
] @punctuation.delimiter

[
  "enum"
  "struct"
  "union"
] @keyword.type

"sizeof" @keyword.operator

[
  "const"
  "volatile"
] @keyword.modifier

; Operators in comma and conditional HLL expressions
(hll_comma_expression
  "," @operator)

(hll_conditional_expression
  [
    "?"
    ":"
  ] @keyword.conditional.ternary)

; Strings and others literal types
(access_class) @constant.builtin

[
  (address)
  (bitmask)
  (file_handle)
  (integer)
  (hll_number_literal)
] @number

[
  (float)
  (frequency)
  (percentage)
  (time)
] @number.float

[
  (string)
  (hll_string_literal)
] @string

(hll_escape_sequence) @string.escape

(path) @string.special.path

(symbol) @string.special.symbol

[
  (character)
  (hll_char_literal)
] @character

; Types in HLL expressions
[
  (hll_type_identifier)
  (hll_type_descriptor)
] @type

(hll_type_qualifier) @keyword.modifier

(hll_primitive_type) @type.builtin

; HLL expressions
(hll_call_expression
  function: (identifier) @function.call)

(hll_call_expression
  function: (hll_field_expression
    field: (hll_field_identifier) @function.call))

; HLL variables
(identifier) @variable

(hll_field_identifier) @variable.member

; Commands
(command_expression
  command: (identifier) @keyword)

(macro_definition
  command: (identifier) @keyword)

(call_expression
  function: (identifier) @function.builtin)

; Returns
((command_expression
  command: (identifier) @keyword.return)
  (#match? @keyword.return "^[eE][nN][dD]([dD][oO])?$"))

((command_expression
  command: (identifier) @keyword.return)
  (#match? @keyword.return "^[rR][eE][tT][uU][rR][nN]$"))

; Subroutine calls
(subroutine_call_expression
  command: (identifier) @keyword
  subroutine: (identifier) @function.call)

; Variables, constants and labels
(macro) @variable.builtin

(trace32_hll_variable) @variable.builtin

(argument_list
  (identifier) @constant.builtin)

((argument_list
  (identifier) @constant.builtin)
  (#match? @constant.builtin "^[%%/][%l%u][%l%u%d.]*$"))

((command_expression
  command: (identifier) @keyword
  arguments: (argument_list
    .
    (identifier) @label))
  (#match? @keyword "^[gG][oO][tT][oO]$"))

(labeled_expression
  label: (identifier) @label)

(option_expression
  (identifier) @constant.builtin)

(format_expression
  (identifier) @constant.builtin)

; Subroutine blocks
(subroutine_block
  command: (identifier) @keyword.function
  subroutine: (identifier) @function)

(labeled_expression
  label: (identifier) @function
  (block))

; Parameter declarations
(parameter_declaration
  command: (identifier) @keyword
  (identifier)? @constant.builtin
  macro: (macro) @variable.parameter)

; Control flow
(if_block
  command: (identifier) @keyword.conditional)

(elif_block
  command: (identifier) @keyword.conditional)

(else_block
  command: (identifier) @keyword.conditional)

(while_block
  command: (identifier) @keyword.repeat)

(repeat_block
  command: (identifier) @keyword.repeat)

(comment) @comment @spell
