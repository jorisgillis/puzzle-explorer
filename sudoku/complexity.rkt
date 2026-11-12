#lang racket

(require "grid.rkt"
         "solver.rkt")

(provide analyze
         format-analysis
         difficulty-score)

(define node-benchmark 5000)

(define (difficulty-score givens nodes)
  (define givens-part (- 1 (/ givens cell-count)))
  (define nodes-part (min 1 (/ nodes node-benchmark)))
  (inexact->exact
   (round (* 100 (+ (* 0.6 nodes-part) (* 0.4 givens-part))))))

(define (analyze g)
  (define givens (count (lambda (v) (not (zero? v))) (vector->list g)))
  (define total-candidates
    (for/sum ([i (in-range cell-count)]
              #:when (empty-cell? g i))
      (length (candidates-of g i))))
  (define candidate-entropy
    (for/sum ([i (in-range cell-count)]
              #:when (empty-cell? g i))
      (define n (length (candidates-of g i)))
      (if (positive? n) (log n 2) 0)))
  (define g-copy (grid-copy g))
  (define propagated (propagate g-copy))
  (define-values (solution stats) (solve/stats g))
  (define nodes (hash-ref stats 'nodes))
  (define max-depth (hash-ref stats 'max-depth))
  (define unique? (= (count-solutions g 2) 1))
  (hash 'givens givens
        'empty-cells (- cell-count givens)
        'total-candidates total-candidates
        'candidate-entropy (exact->inexact candidate-entropy)
        'propagated propagated
        'solver-nodes nodes
        'max-depth max-depth
        'unique? unique?
        'solvable? (and solution #t)
        'difficulty (difficulty-score givens nodes)))

(define (format-analysis a)
  (define (row label value) (format "  ~a: ~a" label value))
  (string-join
   (list "Complexity analysis"
         (row "givens" (hash-ref a 'givens))
         (row "empty cells" (hash-ref a 'empty-cells))
         (row "total candidates" (hash-ref a 'total-candidates))
         (row "candidate entropy (bits)" (~r (hash-ref a 'candidate-entropy) #:precision 2))
         (row "cells filled by propagation" (hash-ref a 'propagated))
         (row "solver nodes explored" (hash-ref a 'solver-nodes))
         (row "max search depth" (hash-ref a 'max-depth))
         (row "unique solution" (hash-ref a 'unique?))
         (row "solvable" (hash-ref a 'solvable?))
         (row "difficulty score (0-100)" (hash-ref a 'difficulty)))
   "\n"))
