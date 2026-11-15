#lang racket

(require rackunit
         "../sudoku/grid.rkt"
         "../sudoku/solver.rkt"
         "../sudoku/solvers.rkt")

(define puzzle-string
  "530070000600195000098000060800060003400803001700020006060000280000419005000080079")
(define solved-string
  "534678912672195348198342567859761423426853791713924856961537284287419635345286179")
;; AI Escargot (Inkala 2006), a hard 17+-clue puzzle
(define escargot-string
  "100007090030020008009600500005300900010080002600004000300000010040000007007000300")

(define puzzle (string->grid puzzle-string))
(define escargot (string->grid escargot-string))
(define invalid (string->grid (string-append "11" (make-string 79 #\0))))

(test-case "registry lists the three gallery solvers in order"
  (check-equal? (solver-names) '(naive propagation-mrv dlx)))

(test-case "every gallery solver finds the known solution (cross-solver agreement)"
  (for ([name (in-list (solver-names))])
    (check-equal? (grid->string (solve-with name puzzle)) solved-string
                  (format "solver ~a" name))))

(test-case "every gallery solver solves AI Escargot and agrees with the production solver"
  (define expected (grid->string (solve escargot)))
  (for ([name (in-list (solver-names))])
    (check-equal? (grid->string (solve-with name escargot)) expected
                  (format "solver ~a" name))))

(test-case "solving does not mutate the input"
  (for ([name (in-list (solver-names))])
    (solve-with name puzzle)
    (check-equal? (grid->string puzzle) puzzle-string)))

(test-case "solve-with/stats reports positive nodes and max-depth for all solvers"
  (for ([name (in-list (solver-names))])
    (define-values (result stats) (solve-with/stats name puzzle))
    (check-true (complete? result))
    (check-true (positive? (hash-ref stats 'nodes)))
    (check-true (positive? (hash-ref stats 'max-depth)))))

(test-case "the DLX solver additionally reports link updates"
  (define-values (_ stats) (solve-with/stats 'dlx puzzle))
  (check-true (positive? (hash-ref stats 'link-updates))))

(test-case "invalid grids short-circuit: no solution, zero stats, zero count"
  (for ([name (in-list (solver-names))])
    (define-values (result stats) (solve-with/stats name invalid))
    (check-false result)
    (check-equal? (hash-ref stats 'nodes) 0)
    (check-equal? (hash-ref stats 'max-depth) 0)
    (check-equal? (count-solutions-with name invalid 2) 0)))

(test-case "count-solutions: unique puzzle has exactly 1, for every solver"
  (for ([name (in-list (solver-names))])
    (check-equal? (count-solutions-with name puzzle 2) 1)))

(test-case "count-solutions respects the cap on the empty grid, for every solver"
  (for ([name (in-list (solver-names))])
    (check-equal? (count-solutions-with name (make-grid) 2) 2)
    (check-equal? (count-solutions-with name (make-grid) 3) 3)))

(test-case "the propagation-mrv entry delegates to the production solver"
  (define-values (prod-result prod-stats) (solve/stats puzzle))
  (define-values (gallery-result gallery-stats) (solve-with/stats 'propagation-mrv puzzle))
  (check-equal? (grid->string gallery-result) (grid->string prod-result))
  (check-equal? gallery-stats prod-stats))

(test-case "lookup rejects unknown solver names"
  (check-exn exn:fail? (lambda () (solve-with 'sat puzzle))))
