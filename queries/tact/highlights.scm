; variable
; --------
(identifier) @variable

; variable.builtin
; ----------------
(self) @variable.builtin

; variable.parameter
; ------------------
(parameter
  name: (identifier) @variable.parameter)

; punctuation.delimiter
; ---------------------
[
  ";"
  ","
  "."
  ":"
  "?"
] @punctuation.delimiter

; punctuation.bracket
; -------------------
[
  "("
  ")"
  "{"
  "}"
] @punctuation.bracket

; operator
; --------
[
  "-"
  "-="
  "+"
  "+="
  "*"
  "*="
  "/"
  "/="
  "%"
  "%="
  "="
  "=="
  "!"
  "!="
  "!!"
  "<"
  "<="
  "<<"
  ">"
  ">="
  ">>"
  "&"
  "|"
  "^"
  "&&"
  "||"
  "->"
] @operator

; constructor
; -----------
(instance_expression
  name: (identifier) @constructor)

(initOf
  name: (identifier) @constructor)

; type
; ----
(type_identifier) @type

; type.builtin
; ------------
((identifier) @type.builtin
  (#any-of? @type.builtin "Context" "SendParameters" "StateInit" "StdAddress" "VarAddress"))

(bounced_type
  "bounced" @type.builtin
  "<" @punctuation.bracket
  ">" @punctuation.bracket)

(map_type
  "map" @type.builtin
  "<" @punctuation.bracket
  ">" @punctuation.bracket)

((type_identifier) @type.builtin
  (#any-of? @type.builtin "Address" "Bool" "Builder" "Cell" "Int" "Slice" "String" "StringBuilder"))

(tlb_serialization
  "as" @keyword
  type: (identifier) @type.builtin)

; string
; ------
(string) @string

; string.escape
; -------------
(escape_sequence) @string.escape

; string.special.path
; -------------------
(import
  library: (string) @string.special.path)

; boolean
; -------
(boolean) @boolean

; constant
; --------
(global_constant
  name: (identifier) @constant)

(storage_constant
  name: (identifier) @constant)

; constant.builtin
; ----------------
(null) @constant.builtin

((identifier) @constant.builtin
  (#any-of? @constant.builtin
    "SendBounceIfActionFail" "SendPayGasSeparately" "SendIgnoreErrors" "SendDestroyIfZero"
    "SendRemainingValue" "SendRemainingBalance" "SendOnlyEstimateFee" "ReserveExact"
    "ReserveAllExcept" "ReserveAtMost" "ReserveAddOriginalBalance" "ReserveInvertSign"
    "ReserveBounceIfActionFail"))

; property
; --------
(instance_argument
  name: (identifier) @variable.member)

(field_access_expression
  name: (identifier) @variable.member)

(field
  name: (identifier) @variable.member)

(storage_variable
  name: (identifier) @variable.member)

; number
; ------
(integer) @number

; keyword
; -------
[
  "with"
  "const"
  "let"
  ; "public" ; -- not used, but declared in grammar.ohm
  ; "extend" ; -- not used, but declared in grammar.ohm
] @keyword

; keyword.type
; ------------
[
  "contract"
  "trait"
  "struct"
  "message"
] @keyword.type

; keyword.function
; ----------------
[
  "fun"
  "native"
  "asm"
] @keyword.function

; keyword.operator
; ----------------
"initOf" @keyword.operator

; keyword.import
; --------------
"import" @keyword.import

; keyword.modifier
; ---------------
[
  "get"
  "mutates"
  "extends"
  "virtual"
  "override"
  "inline"
  "abstract"
] @keyword.modifier

; keyword.repeat
; --------------
(foreach_statement
  .
  (_)
  .
  (_)
  .
  "in" @keyword.repeat)

[
  "while"
  "repeat"
  "do"
  "until"
  "foreach"
] @keyword.repeat

; keyword.return
; --------------
"return" @keyword.return

; keyword.exception
; -----------------
[
  "try"
  "catch"
] @keyword.exception

; keyword.conditional
; -------------------
[
  "if"
  "else"
] @keyword.conditional

; keyword.directive.define
; ------------------------
"primitive" @keyword.directive.define

; function
; --------
(native_function
  name: (identifier) @function)

(asm_function
  name: (identifier) @function)

(global_function
  name: (identifier) @function)

(func_identifier) @function

; function.method
; ---------------
(init_function
  "init" @function.method)

(receive_function
  "receive" @function.method)

(bounced_function
  "bounced" @function.method)

(external_function
  "external" @function.method)

(storage_function
  name: (identifier) @function.method)

; function.call
; -------------
(static_call_expression
  name: (identifier) @function.call)

; function.method.call
; ---------------
(method_call_expression
  name: (identifier) @function.method.call)

; attribute
; ---------
[
  "@name"
  "@interface"
] @attribute

; comment
; -------
(comment) @comment @spell

((comment) @comment.documentation
  (#match? @comment.documentation "^/[*][*][^*].*[*]/$"))
