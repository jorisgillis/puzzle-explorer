#lang racket

(require (prefix-in naive: "solvers/naive.rkt")
         (prefix-in pm: "solvers/propagation-mrv.rkt")
         (prefix-in dlx: "solvers/dlx.rkt"))

(provide all-solvers
         solver-names
         solve-with
         solve-with/stats
         count-solutions-with)

;; Solver gallery aggregator. Each solver module implements the solver
;; contract: solver-name, solve, solve/stats, count-solutions. The registry
;; lists every gallery solver as (name solve solve/stats count-solutions),
;; so harnesses (e.g. the benchmark) can iterate all solvers uniformly.

(define (all-solvers)
  (list (list naive:solver-name naive:solve naive:solve/stats naive:count-solutions)
        (list pm:solver-name pm:solve pm:solve/stats pm:count-solutions)
        (list dlx:solver-name dlx:solve dlx:solve/stats dlx:count-solutions)))

(define (solver-names)
  (map car (all-solvers)))

(define (lookup name)
  (define entry (assoc name (all-solvers)))
  (unless entry
    (raise-argument-error 'lookup "registered solver name" name))
  entry)

(define (solve-with name g)
  ((cadr (lookup name)) g))

(define (solve-with/stats name g)
  ((caddr (lookup name)) g))

(define (count-solutions-with name g [cap 2])
  ((cadddr (lookup name)) g cap))
