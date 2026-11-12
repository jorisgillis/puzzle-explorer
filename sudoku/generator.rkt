#lang racket

(require "grid.rkt"
         "solver.rkt")

(provide generate-complete-grid
         generate-puzzle
         difficulty-givens)

(define (difficulty-givens difficulty)
  (case difficulty
    [(easy) 40]
    [(medium) 32]
    [(hard) 26]
    [else (raise-argument-error 'difficulty-givens "'easy, 'medium or 'hard" difficulty)]))

(define (generate-complete-grid)
  (define g (make-grid))
  (define (fill i)
    (cond
      [(= i cell-count) #t]
      [(not (empty-cell? g i)) (fill (add1 i))]
      [else
       (for/or ([v (in-list (shuffle (candidates-of g i)))])
         (grid-set! g i v)
         (define ok? (fill (add1 i)))
         (cond [ok? ok?]
               [else (grid-set! g i 0) #f]))]))
  (fill 0)
  g)

(define (remove-clues solution target order)
  (define puzzle (grid-copy solution))
  (define (givens) (count (lambda (v) (not (zero? v))) (vector->list puzzle)))
  (for ([pair (in-list order)]
        #:break (<= (givens) target))
    (for ([i (in-list (remove-duplicates (list (car pair) (cdr pair))))])
      (when (positive? (grid-ref puzzle i))
        (define saved (grid-ref puzzle i))
        (grid-set! puzzle i 0)
        (unless (= (count-solutions puzzle 2) 1)
          (grid-set! puzzle i saved)))))
  puzzle)

(define (generate-puzzle difficulty)
  (define target (difficulty-givens difficulty))
  (define solution (generate-complete-grid))
  (define pairs (for/list ([i (in-range 41)]) (cons i (- cell-count 1 i))))
  (define (givens g) (count (lambda (v) (not (zero? v))) (vector->list g)))
  (define best
    (for/fold ([best (remove-clues solution target (shuffle pairs))])
              ([attempt (in-range 10)])
      (define candidate (remove-clues solution target (shuffle pairs)))
      (if (< (givens candidate) (givens best)) candidate best)))
  (values best solution))
