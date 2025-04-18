; inherits: soql

; Apex + SOQL
[
  "["
  "]"
  "{"
  "}"
  "("
  ")"
] @punctuation.bracket

[
  ","
  "."
  ":"
  "?"
  ";"
] @punctuation.delimiter

; Default general color definition
(identifier) @variable

(type_identifier) @type

; Methods
(method_declaration
  name: (identifier) @function.method)

(method_invocation
  name: (identifier) @function.method.call)

(super) @function.builtin

; Annotations
(annotation
  name: (identifier) @attribute)

; Types
(interface_declaration
  name: (identifier) @type)

(class_declaration
  name: (identifier) @type)

(class_declaration
  (superclass) @type)

(enum_declaration
  name: (identifier) @type)

(enum_constant
  name: (identifier) @constant)

(type_arguments
  "<" @punctuation.delimiter)

(type_arguments
  ">" @punctuation.delimiter)

(field_access
  object: (identifier) @type)

(field_access
  field: (identifier) @property)

((scoped_identifier
  scope: (identifier) @type)
  (#match? @type "^[A-Z]"))

((method_invocation
  object: (identifier) @type)
  (#match? @type "^[A-Z]"))

(method_declaration
  (formal_parameters
    (formal_parameter
      name: (identifier) @variable.parameter)))

(constructor_declaration
  name: (identifier) @constructor)

(dml_type) @function.builtin

(assignment_operator) @operator

(update_operator) @operator

(trigger_declaration
  name: (identifier) @type
  object: (identifier) @type
  (trigger_event) @keyword
  (","
    (trigger_event) @keyword)*)

[
  "@"
  "="
  "!="
  "<="
  ">="
] @operator

(binary_expression
  operator: [
    ">"
    "<"
    "=="
    "==="
    "!=="
    "&&"
    "||"
    "+"
    "-"
    "*"
    "/"
    "&"
    "|"
    "^"
    "%"
    "<<"
    ">>"
    ">>>"
  ] @operator)

(unary_expression
  operator: [
    "+"
    "-"
    "!"
    "~"
  ]) @operator

"=>" @operator

[
  (boolean_type)
  (void_type)
] @type.builtin

; Fields
(field_declaration
  declarator: (variable_declarator
    name: (identifier) @variable.member))

(field_access
  field: (identifier) @variable.member)

; Variables
(variable_declarator
  (identifier) @property)

(field_declaration
  (modifiers
    (modifier
      [
        (final)
        (static)
      ])
    (modifier
      [
        (final)
        (static)
      ]))
  (variable_declarator
    name: (identifier) @constant))

((identifier) @constant
  (#match? @constant "^[A-Z][A-Z0-9_]+$")) ; SCREAM SNAKE CASE

(this) @variable.builtin

; Literals
[
  (int)
  (decimal)
  (currency_literal)
] @number

(string_literal) @string

[
  (line_comment)
  (block_comment)
] @comment

(null_literal) @constant.builtin

; ;; Keywords
[
  "abstract"
  "final"
  "private"
  "protected"
  "public"
  "static"
] @keyword.modifier

[
  "if"
  "else"
  "switch"
] @keyword.conditional

[
  "for"
  "while"
  "do"
  "break"
] @keyword.repeat

"return" @keyword.return

[
  "throw"
  "finally"
  "try"
  "catch"
] @keyword.exception

"new" @keyword.operator

[
  (abstract)
  (all_rows_clause)
  "continue"
  "extends"
  (final)
  "get"
  (global)
  "implements"
  "instanceof"
  "on"
  (override)
  (private)
  (protected)
  (public)
  "set"
  (static)
  (testMethod)
  (webservice)
  (transient)
  "trigger"
  (virtual)
  "when"
  (with_sharing)
  (without_sharing)
  (inherited_sharing)
] @keyword

[
  "interface"
  "class"
  "enum"
] @keyword.type

"System.runAs" @function.builtin
