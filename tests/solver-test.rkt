#lang racket

(require rackunit
         "../sudoku/grid.rkt"
         "../sudoku/solver.rkt")

(define puzzle-string
  "530070000600195000098000060800060003400803001700020006060000280000419005000080079")
(define solved-string
  "534678912672195348198342567859761423426853791713924856961537284287419635345286179")

(define puzzle (string->grid puzzle-string))

(test-case "solve finds the known solution"
  (check-equal? (grid->string (solve puzzle)) solved-string))

(test-case "solve does not mutate the input"
  (solve puzzle)
  (check-equal? (grid->string puzzle) puzzle-string))

(test-case "solve/stats reports nodes and depth"
  (define-values (result stats) (solve/stats puzzle))
  (check-true (complete? result))
  (check-true (positive? (hash-ref stats 'nodes)))
  (check-true (positive? (hash-ref stats 'max-depth))))

(test-case "unsolvable puzzle returns #f"
  (define bad (string->grid (string-append "11" (make-string 79 #\0))))
  (check-false (solve bad)))

(test-case "count-solutions: unique puzzle has exactly 1"
  (check-equal? (count-solutions puzzle) 1))

(test-case "count-solutions respects the cap"
  (check-equal? (count-solutions (make-grid) 2) 2))

(test-case "empty grid has many solutions (capped at 2)"
  (check-equal? (count-solutions (make-grid) 3) 3))

(test-case "candidates-of an empty corner in the puzzle"
  (check-equal? (candidates-of puzzle 2) '(1 2 4)))

(test-case "propagate fills naked/hidden singles without guessing"
  (define g (grid-copy puzzle))
  (define filled (propagate g))
  (check-true (positive? filled))
  (check-true (valid? g)))
