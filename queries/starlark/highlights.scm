; From tree-sitter-python licensed under MIT License
; Copyright (c) 2016 Max Brunsfeld
; Variables
(identifier) @variable

; Reset highlighting in f-string interpolations
(interpolation) @none

; Identifier naming conventions
((identifier) @type
  (#match? @type "^[A-Z].*[a-z]"))

((identifier) @constant
  (#match? @constant "^[A-Z][A-Z_0-9]*$"))

((identifier) @constant.builtin
  (#match? @constant.builtin "^__[a-zA-Z0-9_]*__$"))

((identifier) @constant.builtin
  (#any-of? @constant.builtin
    ; https://docs.python.org/3/library/constants.html
    "NotImplemented" "Ellipsis" "quit" "exit" "copyright" "credits" "license"))

((attribute
  attribute: (identifier) @variable.member)
  (#match? @variable.member "^[%l_].*$"))

((assignment
  left: (identifier) @type.definition
  (type
    (identifier) @_annotation))
  (#eq? @_annotation "TypeAlias"))

((assignment
  left: (identifier) @type.definition
  right: (call
    function: (identifier) @_func))
  (#any-of? @_func "TypeVar" "NewType"))

; Decorators
((decorator
  "@" @attribute)
  (#set! priority 101))

(decorator
  (identifier) @attribute)

(decorator
  (attribute
    attribute: (identifier) @attribute))

(decorator
  (call
    (identifier) @attribute))

(decorator
  (call
    (attribute
      attribute: (identifier) @attribute)))

((decorator
  (identifier) @attribute.builtin)
  (#any-of? @attribute.builtin "classmethod" "property"))

; Builtin functions
((call
  function: (identifier) @function.builtin)
  (#any-of? @function.builtin
    "abs" "all" "any" "ascii" "bin" "bool" "breakpoint" "bytearray" "bytes" "callable" "chr"
    "classmethod" "compile" "complex" "delattr" "dict" "dir" "divmod" "enumerate" "eval" "exec"
    "fail" "filter" "float" "format" "frozenset" "getattr" "globals" "hasattr" "hash" "help" "hex"
    "id" "input" "int" "isinstance" "issubclass" "iter" "len" "list" "locals" "map" "max"
    "memoryview" "min" "next" "object" "oct" "open" "ord" "pow" "print" "property" "range" "repr"
    "reversed" "round" "set" "setattr" "slice" "sorted" "staticmethod" "str" "struct" "sum" "super"
    "tuple" "type" "vars" "zip" "__import__"))

; Function definitions
(function_definition
  name: (identifier) @function)

(type
  (identifier) @type)

(type
  (subscript
    (identifier) @type)) ; type subscript: Tuple[int]

((call
  function: (identifier) @_isinstance
  arguments: (argument_list
    (_)
    (identifier) @type))
  (#eq? @_isinstance "isinstance"))

((identifier) @type.builtin
  (#any-of? @type.builtin
    ; https://docs.python.org/3/library/exceptions.html
    "ArithmeticError" "BufferError" "LookupError" "AssertionError" "AttributeError" "EOFError"
    "FloatingPointError" "ModuleNotFoundError" "IndexError" "KeyError" "KeyboardInterrupt"
    "MemoryError" "NameError" "NotImplementedError" "OSError" "OverflowError" "RecursionError"
    "ReferenceError" "RuntimeError" "StopIteration" "StopAsyncIteration" "SyntaxError"
    "IndentationError" "TabError" "SystemError" "SystemExit" "TypeError" "UnboundLocalError"
    "UnicodeError" "UnicodeEncodeError" "UnicodeDecodeError" "UnicodeTranslateError" "ValueError"
    "ZeroDivisionError" "EnvironmentError" "IOError" "WindowsError" "BlockingIOError"
    "ChildProcessError" "ConnectionError" "BrokenPipeError" "ConnectionAbortedError"
    "ConnectionRefusedError" "ConnectionResetError" "FileExistsError" "FileNotFoundError"
    "InterruptedError" "IsADirectoryError" "NotADirectoryError" "PermissionError"
    "ProcessLookupError" "TimeoutError" "Warning" "UserWarning" "DeprecationWarning"
    "PendingDeprecationWarning" "SyntaxWarning" "RuntimeWarning" "FutureWarning" "UnicodeWarning"
    "BytesWarning" "ResourceWarning"
    ; https://docs.python.org/3/library/stdtypes.html
    "bool" "int" "float" "complex" "list" "tuple" "range" "str" "bytes" "bytearray" "memoryview"
    "set" "frozenset" "dict" "type"))

; Normal parameters
(parameters
  (identifier) @variable.parameter)

; Lambda parameters
(lambda_parameters
  (identifier) @variable.parameter)

(lambda_parameters
  (tuple_pattern
    (identifier) @variable.parameter))

; Default parameters
(keyword_argument
  name: (identifier) @variable.parameter)

; Naming parameters on call-site
(default_parameter
  name: (identifier) @variable.parameter)

(typed_parameter
  (identifier) @variable.parameter)

(typed_default_parameter
  (identifier) @variable.parameter)

; Variadic parameters *args, **kwargs
(parameters
  (list_splat_pattern
    ; *args
    (identifier) @variable.parameter))

(parameters
  (dictionary_splat_pattern
    ; **kwargs
    (identifier) @variable.parameter))

; Literals
(none) @constant.builtin

[
  (true)
  (false)
] @boolean

((identifier) @variable.builtin
  (#eq? @variable.builtin "self"))

((identifier) @variable.builtin
  (#eq? @variable.builtin "cls"))

(integer) @number

(float) @number.float

(comment) @comment @spell

((module
  .
  (comment) @keyword.directive @nospell)
  (#match? @keyword.directive "^#!/"))

(string) @string

[
  (escape_sequence)
  (escape_interpolation)
] @string.escape

; doc-strings
(module
  .
  (expression_statement
    (string) @string.documentation @spell))

(function_definition
  body: (block
    .
    (expression_statement
      (string) @string.documentation @spell)))

; Tokens
[
  "-"
  "-="
  ":="
  "!="
  "*"
  "**"
  "**="
  "*="
  "/"
  "//"
  "//="
  "/="
  "&"
  "&="
  "%"
  "%="
  "^"
  "^="
  "+"
  "+="
  "<"
  "<<"
  "<<="
  "<="
  "<>"
  "="
  "=="
  ">"
  ">="
  ">>"
  ">>="
  "@"
  "@="
  "|"
  "|="
  "~"
  "->"
] @operator

; Keywords
[
  "and"
  "in"
  "not"
  "or"
  "del"
] @keyword.operator

[
  "def"
  "lambda"
] @keyword.function

[
  "async"
  "exec"
  "pass"
  "print"
  "with"
  "as"
] @keyword

"async" @keyword.coroutine

"return" @keyword.return

((call
  function: (identifier) @keyword.import
  arguments: (argument_list
    (string) @string))
  (#eq? @keyword.import "load"))

[
  "if"
  "elif"
  "else"
  "match"
  "case"
] @keyword.conditional

[
  "for"
  "while"
  "break"
  "continue"
] @keyword.repeat

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

(interpolation
  "{" @punctuation.special
  "}" @punctuation.special)

(type_conversion) @function.macro

[
  ","
  "."
  ":"
  ";"
  (ellipsis)
] @punctuation.delimiter

; Starlark-specific
; Assertion calls
(assert_keyword) @keyword

(assert_builtin) @function.builtin

; Struct definitions
((call
  function: (identifier) @_func
  arguments: (argument_list
    (keyword_argument
      name: (identifier) @variable.member)))
  (#eq? @_func "struct"))

; Function calls
(call
  function: (identifier) @function.call)

(call
  function: (attribute
    attribute: (identifier) @function.method.call))

((call
  function: (identifier) @constructor)
  (#match? @constructor "^[A-Z]"))

((call
  function: (attribute
    attribute: (identifier) @constructor))
  (#match? @constructor "^[A-Z]"))
