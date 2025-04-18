((function_call
  name: [
    (identifier) @_cdef_identifier
    (_
      _
      (identifier) @_cdef_identifier)
  ]
  arguments: (arguments
    (string
      content: _ @injection.content)))
  (#eq? @_cdef_identifier "cdef")
  (#set! injection.language "c"))

((comment) @injection.content
  (#match? @injection.content "[-][-][-][%s]*@")
  (#offset! @injection.content 0 3 0 0)
  (#set! injection.language "luadoc"))

; string.match("123", "%d+")
(function_call
  (dot_index_expression
    field: (identifier) @_method
    (#any-of? @_method "find" "format" "match" "gmatch" "gsub"))
  arguments: (arguments
    .
    (_)
    .
    (string
      content: _ @injection.content))
  (#set! injection.language "luap"))

; ("123"):match("%d+")
(function_call
  (method_index_expression
    method: (identifier) @_method
    (#any-of? @_method "find" "format" "match" "gmatch" "gsub"))
  arguments: (arguments
    .
    (string
      content: _ @injection.content))
  (#set! injection.language "luap"))

((comment) @injection.content
  (#set! injection.language "comment"))
