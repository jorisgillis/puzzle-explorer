#lang racket

(require (prefix-in prod: "../solver.rkt"))

(provide solver-name
         solve
         solve/stats
         count-solutions)

(define solver-name 'propagation-mrv)

;; Adapter: the production propagation + MRV solver (sudoku/solver.rkt),
;; exposed as a gallery entry. Delegates without duplicating or modifying
;; the production module.

(define (solve g)
  (prod:solve g))

(define (solve/stats g)
  (prod:solve/stats g))

(define (count-solutions g [cap 2])
  (prod:count-solutions g cap))
