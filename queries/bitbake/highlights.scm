; Includes
[
  "inherit"
  "include"
  "require"
  "export"
  "import"
] @keyword.import

; Keywords
[
  "unset"
  "EXPORT_FUNCTIONS"
  "python"
  "assert"
  "exec"
  "global"
  "nonlocal"
  "pass"
  "print"
  "with"
  "as"
] @keyword

[
  "async"
  "await"
] @keyword.coroutine

[
  "return"
  "yield"
] @keyword.return

(yield
  "from" @keyword.return)

(future_import_statement
  "from" @keyword.import
  "__future__" @constant.builtin)

(import_from_statement
  "from" @keyword.import)

"import" @keyword.import

(aliased_import
  "as" @keyword.import)

[
  "if"
  "elif"
  "else"
] @keyword.conditional

[
  "for"
  "while"
  "break"
  "continue"
] @keyword.repeat

[
  "try"
  "except"
  "except*"
  "raise"
  "finally"
] @keyword.exception

(raise_statement
  "from" @keyword.exception)

(try_statement
  (else_clause
    "else" @keyword.exception))

[
  "addtask"
  "deltask"
  "addhandler"
  "def"
  "lambda"
] @keyword.function

[
  "before"
  "after"
] @keyword.modifier

[
  "append"
  "prepend"
  "remove"
] @keyword.modifier

; Variables
[
  (identifier)
  (python_identifier)
] @variable

[
  "noexec"
  "OVERRIDES"
  "$BB_ENV_PASSTHROUGH"
  "$BB_ENV_PASSTHROUGH_ADDITIONS"
] @variable.builtin

; Reset highlighting in f-string interpolations
(interpolation) @none

; Identifier naming conventions
((python_identifier) @type
  (#match? @type "^[A-Z].*[a-z]"))

([
  (identifier)
  (python_identifier)
] @constant
  (#match? @constant "^[A-Z][A-Z_0-9]*$"))

((python_identifier) @constant.builtin
  (#match? @constant.builtin "^__[a-zA-Z0-9_]*__$"))

((python_identifier) @constant.builtin
  (#any-of? @constant.builtin
    ; https://docs.python.org/3/library/constants.html
    "NotImplemented" "Ellipsis" "quit" "exit" "copyright" "credits" "license"))

((assignment
  left: (python_identifier) @type.definition
  (type
    (python_identifier) @_annotation))
  (#eq? @_annotation "TypeAlias"))

((assignment
  left: (python_identifier) @type.definition
  right: (call
    function: (python_identifier) @_func))
  (#any-of? @_func "TypeVar" "NewType"))

; Fields
(flag) @variable.member

((attribute
  attribute: (python_identifier) @variable.member)
  (#match? @variable.member "^[%l_].*$"))

; Functions
(call
  function: (python_identifier) @function.call)

(call
  function: (attribute
    attribute: (python_identifier) @function.method.call))

((call
  function: (python_identifier) @constructor)
  (#match? @constructor "^%u"))

((call
  function: (attribute
    attribute: (python_identifier) @constructor))
  (#match? @constructor "^%u"))

((call
  function: (python_identifier) @function.builtin)
  (#any-of? @function.builtin
    "abs" "all" "any" "ascii" "bin" "bool" "breakpoint" "bytearray" "bytes" "callable" "chr"
    "classmethod" "compile" "complex" "delattr" "dict" "dir" "divmod" "enumerate" "eval" "exec"
    "filter" "float" "format" "frozenset" "getattr" "globals" "hasattr" "hash" "help" "hex" "id"
    "input" "int" "isinstance" "issubclass" "iter" "len" "list" "locals" "map" "max" "memoryview"
    "min" "next" "object" "oct" "open" "ord" "pow" "print" "property" "range" "repr" "reversed"
    "round" "set" "setattr" "slice" "sorted" "staticmethod" "str" "sum" "super" "tuple" "type"
    "vars" "zip" "__import__"))

(python_function_definition
  name: (python_identifier) @function)

(type
  (python_identifier) @type)

(type
  (subscript
    (python_identifier) @type)) ; type subscript: Tuple[int]

((call
  function: (python_identifier) @_isinstance
  arguments: (argument_list
    (_)
    (python_identifier) @type))
  (#eq? @_isinstance "isinstance"))

(anonymous_python_function
  (identifier) @function)

(function_definition
  (identifier) @function)

(addtask_statement
  (identifier) @function)

(deltask_statement
  (identifier) @function)

(export_functions_statement
  (identifier) @function)

(addhandler_statement
  (identifier) @function)

(python_function_definition
  body: (block
    .
    (expression_statement
      (python_string) @string.documentation @spell)))

; Namespace
(inherit_path) @module

; Normal parameters
(parameters
  (python_identifier) @variable.parameter)

; Lambda parameters
(lambda_parameters
  (python_identifier) @variable.parameter)

(lambda_parameters
  (tuple_pattern
    (python_identifier) @variable.parameter))

; Default parameters
(keyword_argument
  name: (python_identifier) @variable.parameter)

; Naming parameters on call-site
(default_parameter
  name: (python_identifier) @variable.parameter)

(typed_parameter
  (python_identifier) @variable.parameter)

(typed_default_parameter
  (python_identifier) @variable.parameter)

; Variadic parameters *args, **kwargs
(parameters
  (list_splat_pattern
    ; *args
    (python_identifier) @variable.parameter))

(parameters
  (dictionary_splat_pattern
    ; **kwargs
    (python_identifier) @variable.parameter))

; Literals
(none) @constant.builtin

[
  (true)
  (false)
] @boolean

((python_identifier) @variable.builtin
  (#any-of? @variable.builtin "self" "cls"))

(integer) @number

(float) @number.float

; Operators
[
  "?="
  "??="
  ":="
  "=+"
  ".="
  "=."
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

[
  "and"
  "in"
  "is"
  "not"
  "or"
  "is not"
  "not in"
  "del"
] @keyword.operator

; Literals
[
  (string)
  (python_string)
  "\""
] @string

(include_path) @string.special.path

[
  (escape_sequence)
  (escape_interpolation)
] @string.escape

; Punctuation
[
  "("
  ")"
  "{"
  "}"
  "["
  "]"
] @punctuation.bracket

[
  ":"
  "->"
  ";"
  "."
  ","
  (ellipsis)
] @punctuation.delimiter

(variable_expansion
  [
    "${"
    "}"
  ] @punctuation.special)

(inline_python
  [
    "${@"
    "}"
  ] @punctuation.special)

(interpolation
  "{" @punctuation.special
  "}" @punctuation.special)

(type_conversion) @function.macro

([
  (identifier)
  (python_identifier)
] @type.builtin
  (#any-of? @type.builtin
    ; https://docs.python.org/3/library/exceptions.html
    "BaseException" "Exception" "ArithmeticError" "BufferError" "LookupError" "AssertionError"
    "AttributeError" "EOFError" "FloatingPointError" "GeneratorExit" "ImportError"
    "ModuleNotFoundError" "IndexError" "KeyError" "KeyboardInterrupt" "MemoryError" "NameError"
    "NotImplementedError" "OSError" "OverflowError" "RecursionError" "ReferenceError" "RuntimeError"
    "StopIteration" "StopAsyncIteration" "SyntaxError" "IndentationError" "TabError" "SystemError"
    "SystemExit" "TypeError" "UnboundLocalError" "UnicodeError" "UnicodeEncodeError"
    "UnicodeDecodeError" "UnicodeTranslateError" "ValueError" "ZeroDivisionError" "EnvironmentError"
    "IOError" "WindowsError" "BlockingIOError" "ChildProcessError" "ConnectionError"
    "BrokenPipeError" "ConnectionAbortedError" "ConnectionRefusedError" "ConnectionResetError"
    "FileExistsError" "FileNotFoundError" "InterruptedError" "IsADirectoryError"
    "NotADirectoryError" "PermissionError" "ProcessLookupError" "TimeoutError" "Warning"
    "UserWarning" "DeprecationWarning" "PendingDeprecationWarning" "SyntaxWarning" "RuntimeWarning"
    "FutureWarning" "ImportWarning" "UnicodeWarning" "BytesWarning" "ResourceWarning"
    ; https://docs.python.org/3/library/stdtypes.html
    "bool" "int" "float" "complex" "list" "tuple" "range" "str" "bytes" "bytearray" "memoryview"
    "set" "frozenset" "dict" "type" "object"))

(comment) @comment @spell
