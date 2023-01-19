; opposite of expand

; factor
(=>
    (+ (* a b) (* a c) ..)
    (+ (* a (+ b c)) ..)
)
