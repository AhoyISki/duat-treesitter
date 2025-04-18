; Primitives
(boolean) @boolean

(comment) @comment @spell

((comment) @comment.documentation
  (#match? @comment.documentation "^[-][-][-]"))

((comment) @comment.documentation
  (#match? @comment.documentation "^[-][-](%s?)@"))

(shebang_comment) @keyword.directive

(identifier) @variable

((identifier) @variable.builtin
  (#eq? @variable.builtin "self"))

(nil) @constant.builtin

(number) @number

(string) @string

(table_constructor
  [
    "{"
    "}"
  ] @constructor)

(varargs
  "..." @constant.builtin)

[
  ","
  "."
  ":"
  ";"
] @punctuation.delimiter

(escape_sequence) @string.escape

(format_specifier) @string.escape

; Basic statements/Keywords
[
  "if"
  "then"
  "elseif"
  "else"
] @keyword.conditional

[
  "for"
  "while"
  "repeat"
  "until"
] @keyword.repeat

"return" @keyword.return

[
  "in"
  "local"
  (break)
  (goto)
  "do"
  "end"
] @keyword

(label) @label

; Global isn't a real keyword, but it gets special treatment in these places
(var_declaration
  "global" @keyword)

(type_declaration
  "global" @keyword)

(function_statement
  "global" @keyword)

(record_declaration
  "global" @keyword)

(enum_declaration
  "global" @keyword)

; Ops
(bin_op
  (op) @operator)

(unary_op
  (op) @operator)

[
  "="
  "as"
] @operator

; Functions
(function_statement
  "function" @keyword.function
  .
  name: (_) @function)

(anon_function
  "function" @keyword.function)

(function_body
  "end" @keyword.function)

(arg
  name: (identifier) @variable.parameter)

(function_signature
  (arguments
    .
    (arg
      name: (identifier) @variable.builtin))
  (#eq? @variable.builtin "self"))

(typeargs
  "<" @punctuation.bracket
  .
  (_) @variable.parameter
  .
  (","
    .
    (_) @variable.parameter)*
  .
  ">" @punctuation.bracket)

(function_call
  (identifier) @function
  .
  (arguments))

(function_call
  (index
    (_)
    key: (identifier) @function)
  .
  (arguments))

(function_call
  (method_index
    (_)
    key: (identifier) @function)
  .
  (arguments))

; Types
(record_declaration
  .
  "record" @keyword.type
  name: (identifier) @type)

(anon_record
  .
  "record" @keyword.type)

(record_body
  (record_declaration
    .
    "record" @keyword.type
    .
    name: (identifier) @type))

(record_body
  (enum_declaration
    .
    "enum" @keyword.type
    .
    name: (identifier) @type))

(record_body
  (typedef
    .
    "type" @keyword
    .
    name: (identifier) @type
    .
    "="))

(record_body
  (metamethod
    "metamethod" @keyword))

(record_body
  (userdata) @keyword)

(enum_declaration
  "enum" @keyword.type
  name: (identifier) @type)

(type_declaration
  "type" @keyword)

(type_declaration
  (identifier) @type)

(simple_type
  name: (identifier) @type)

(type_index
  (identifier) @type)

(type_union
  "|" @operator)

(function_type
  "function" @type)

; The rest of it
(var_declaration
  declarators: (var_declarators
    (var
      name: (identifier) @variable)))

(var_declaration
  declarators: (var_declarators
    (var
      "<" @punctuation.bracket
      .
      attribute: (attribute) @attribute
      .
      ">" @punctuation.bracket)))

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket
