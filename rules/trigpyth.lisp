#map trigexpand simplify

; Pythagorean identities
(=> (+ (* (sin u) (sin u)) (* (cos u) (cos u)) ..) (+ 1 ..))
(=> (+ 1 (* (tan u) (tan u)) ..) (+ (* (sec u) (sec u)) ..))
(=> (+ 1 (* (cot u) (cot u)) ..) (+ (* (csc u) (csc u)) ..))

(=> (+ 1 (- (* (cos u) (cos u))) ..) (+ (* (sin u) (sin u)) ..))
(=> (+ 1 (- (* (sin u) (sin u))) ..) (+ (* (cos u) (cos u)) ..))

(=> (+ (* (sec u) (sec u)) (- 1) ..) (+ (* (tan u) (tan u)) ..))
(=> (+ (* (sec u) (sec u)) (- (* (tan u) (tan u))) ..) (+ 1 ..))

(=> (+ (* (csc u) (csc u)) (- (* (cot u) (cot u))) ..) (+ 1 ..))
(=> (+ (* (csc u) (csc u)) (- 1) ..) (+ (* (cot u) (cot u)) ..))
