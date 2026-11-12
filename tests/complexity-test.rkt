#lang racket

(require rackunit
         "../sudoku/grid.rkt"
         "../sudoku/solver.rkt"
         "../sudoku/generator.rkt"
         "../sudoku/complexity.rkt")

(define puzzle-string
  "530070000600195000098000060800060003400803001700020006060000280000419005000080079")
(define puzzle (string->grid puzzle-string))

(test-case "analyze returns all expected metrics"
  (define a (analyze puzzle))
  (check-equal? (hash-ref a 'givens) 30)
  (check-equal? (hash-ref a 'empty-cells) 51)
  (check-true (positive? (hash-ref a 'total-candidates)))
  (check-true (positive? (hash-ref a 'candidate-entropy)))
  (check-true (positive? (hash-ref a 'solver-nodes)))
  (check-true (hash-ref a 'unique?))
  (check-true (hash-ref a 'solvable?))
  (check-true (<= 0 (hash-ref a 'difficulty) 100)))

(test-case "harder puzzles score higher on search effort"
  (define (nodes g) (hash-ref (analyze g) 'solver-nodes))
  (define-values (hard-p _) (generate-puzzle 'hard))
  (check-true (> (nodes hard-p) (nodes puzzle))))

(test-case "difficulty score is bounded"
  (check-equal? (difficulty-score 81 0) 0)
  (check-true (<= 90 (difficulty-score 17 1000000) 100)))

(test-case "format-analysis produces a readable report"
  (define report (format-analysis (analyze puzzle)))
  (check-true (string-contains? report "difficulty score"))
  (check-true (string-contains? report "solver nodes")))
