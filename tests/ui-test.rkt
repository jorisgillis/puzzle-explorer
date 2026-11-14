#lang racket

(require rackunit
         "../sudoku/grid.rkt"
         "../sudoku/solver.rkt"
         "../sudoku/ui.rkt")

(test-case "heat-color matches the 9-step palette (map #20)"
  (define expected
    '([0 200 30 30]
      [1 150 233 150]
      [2 171 233 150]
      [3 191 233 150]
      [4 212 233 150]
      [5 233 233 150]
      [6 233 212 150]
      [7 233 191 150]
      [8 233 171 150]
      [9 233 150 150]))
  (for ([row (in-list expected)])
    (define n (first row))
    (define c (heat-color n))
    (check-equal? (send c red) (second row) (format "red of count ~a" n))
    (check-equal? (send c green) (third row) (format "green of count ~a" n))
    (check-equal? (send c blue) (fourth row) (format "blue of count ~a" n))))

(test-case "heat counts come from the current board"
  (define g (make-grid))
  (check-equal? (length (candidates-of g 0)) 9)
  (grid-set! g 1 5)
  (check-equal? (length (candidates-of g 0)) 8)
  (define dead (make-grid))
  (for ([p (in-list (take (peers-of 0) 9))]
        [v (in-naturals 1)])
    (grid-set! dead p v))
  (check-equal? (length (candidates-of dead 0)) 0))
