(comment) @comment

(generated_comment) @comment

(title) @markup.heading

; (text) @none
(branch) @markup.link

(change
  (new) @diff.plus)
(change
  [
    (modified)
    (typechange)
  ]  @diff.delta)
(change
  (renamed) @diff.delta.renamed)
(change
  (deleted) @diff.minus)

(filepath) @string.special.path

(arrow) @punctuation.delimiter

(subject) @markup.heading @spell

(subject
  (subject_prefix) @function @nospell)

(prefix
  (type) @keyword @nospell)

(prefix
  (scope) @variable.parameter @nospell)

(prefix
  [
    "("
    ")"
    ":"
  ] @punctuation.delimiter)

(prefix
  "!" @punctuation.special)

(message) @spell

(trailer
  (token) @label)

; (trailer (value) @none)
(breaking_change
  (token) @comment.error)

(breaking_change
  (value) @none @spell)

(scissor) @comment
