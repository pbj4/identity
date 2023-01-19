; algebraic term flattening or elimination

; associativity
(=> (* (* a) b) (* a b))
(=> (+ (+ a) b) (+ a b))

; identity
(=> (* a 1) a)
(=> (+ a 0) a)
(=> (* a (/ 1)) a)
(=> (* a) a)
(=> (+ a) a)

; absorption
(=> (* a 0) 0)

; inverse
(=> (/ (/ a)) a)
(=> (- (- a)) a)

; zero argument
(=> (*) 1)
(=> (+) 0)

; canonicalization
(=> (* n (/ m1) (/ m2)) (* n (/ (* m1 m2))))
(=> (* a (- b)) (- (* a b)))
(=> (/ (- a)) (- (/ a)))
(=> (/ (* d (/ n))) (* n (/ d)))
