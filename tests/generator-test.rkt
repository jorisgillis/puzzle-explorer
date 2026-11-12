#lang racket

(require rackunit
         "../sudoku/grid.rkt"
         "../sudoku/solver.rkt"
         "../sudoku/generator.rkt")

(test-case "generate-complete-grid yields a complete valid grid"
  (define g (generate-complete-grid))
  (check-true (complete? g)))

(test-case "generated grids vary"
  (check-not-equal? (grid->string (generate-complete-grid))
                    (grid->string (generate-complete-grid))))

(test-case "generate-puzzle has a unique solution matching the solution grid"
  (define-values (puzzle solution) (generate-puzzle 'easy))
  (check-equal? (count-solutions puzzle 2) 1)
  (check-equal? (solve puzzle) solution)
  (check-true (valid? puzzle)))

(test-case "givens count respects the difficulty target"
  (define (givens g) (count (lambda (v) (not (zero? v))) (vector->list g)))
  (define-values (easy _e) (generate-puzzle 'easy))
  (define-values (medium _m) (generate-puzzle 'medium))
  (define-values (hard _h) (generate-puzzle 'hard))
  (check-true (<= (givens easy) (difficulty-givens 'easy)))
  (check-true (<= (givens medium) (difficulty-givens 'medium)))
  (check-true (<= (givens hard) (difficulty-givens 'hard)))
  (check-true (< (givens hard) (givens easy))))

(test-case "puzzle is a subset of its solution"
  (define-values (puzzle solution) (generate-puzzle 'medium))
  (for ([i (in-range cell-count)])
    (define pv (grid-ref puzzle i))
    (when (positive? pv)
      (check-equal? pv (grid-ref solution i)))))

(test-case "invalid difficulty raises"
  (check-exn exn:fail? (lambda () (generate-puzzle 'impossible))))
