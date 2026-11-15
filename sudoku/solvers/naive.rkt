#lang racket

(require "../grid.rkt"
         ;; candidates-of only; solver.rkt itself is not modified
         (only-in "../solver.rkt" candidates-of))

(provide solver-name
         solve
         solve/stats
         count-solutions)

(define solver-name 'naive)

;; Naive backtracking solver: scan cells 0-80 in order, try digits 1-9 in
;; order using grid candidates. No constraint propagation, no MRV.
;; Invalid input never enters the search: (values #f zero-stats) / 0.

(define (solve/stats g)
  (define nodes 0)
  (define max-depth 0)
  (define (search g depth)
    (set! nodes (add1 nodes))
    (set! max-depth (max max-depth depth))
    (cond
      [(complete? g) g]
      [else
       (define i (for/first ([i (in-range cell-count)]
                             #:when (empty-cell? g i))
                   i))
       (for/or ([v (in-list (candidates-of g i))])
         (grid-set! g i v)
         (define result (search g (add1 depth)))
         (cond [result result]
               ;; backtrack: undo the guess before trying the next digit
               [else (grid-set! g i 0) #f]))]))
  (define result (and (valid? g) (search (grid-copy g) 0)))
  (values result (hash 'nodes nodes 'max-depth max-depth)))

(define (solve g)
  (define-values (result _) (solve/stats g))
  result)

;; Count the solutions of g, stopping early once the count reaches cap
;; (default 2, enough to decide uniqueness). Returns 0 for invalid grids.
;; A dead-end branch ends only that branch; the search stops as soon as
;; the cap is reached.
(define (count-solutions g [cap 2])
  (define count 0)
  ;; returns #t when the whole search should stop (cap reached)
  (define (search g)
    (cond
      [(>= count cap) #t]
      [(complete? g) (set! count (add1 count)) (>= count cap)]
      [else
       (define i (for/first ([i (in-range cell-count)]
                             #:when (empty-cell? g i))
                   i))
       (let loop ([cs (candidates-of g i)])
         (cond
           [(null? cs) #f]
           [(>= count cap) #t]
           [else
            (grid-set! g i (car cs))
            (define stop? (search g))
            (grid-set! g i 0)
            (if stop? #t (loop (cdr cs)))]))]))
  (when (valid? g)
    (search (grid-copy g)))
  count)
