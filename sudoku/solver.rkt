#lang racket

(require "grid.rkt")

(provide candidates-of
         propagate
         solve
         solve/stats
         count-solutions)

;; Digits 1-9 that can legally go in cell i: every value not already taken
;; by i's peers (same row, column, or box). Empty peers (value 0) are
;; ignored. Returns '() when no digit fits.
(define (candidates-of g i)
  (define used
    (for/fold ([s (set)])
              ([p (in-list (peers-of i))])
      (define v (grid-ref g p))
      (if (zero? v) s (set-add s v))))
  (for/list ([v (in-range 1 10)] #:unless (set-member? used v)) v))

;; Fill every forced cell until a fixpoint, mutating g in place and
;; returning the number of cells filled. Two kinds of forced moves:
;; - naked single: an empty cell with exactly one candidate
;; - hidden single: a digit that fits in exactly one empty cell of a unit
(define (propagate g)
  (let loop ([filled 0])
    (define progressed 0)
    ;; naked singles
    (for ([i (in-range cell-count)]
          #:when (empty-cell? g i))
      (define cs (candidates-of g i))
      (when (= (length cs) 1)
        (grid-set! g i (first cs))
        (set! progressed (add1 progressed))))
    ;; hidden singles
    (for ([unit (in-list units)])
      (for ([v (in-range 1 10)])
        (define spots
          (for/list ([i (in-list unit)]
                     #:when (and (empty-cell? g i)
                                 (memv v (candidates-of g i))))
            i))
        (when (= (length spots) 1)
          (grid-set! g (first spots) v)
          (set! progressed (add1 progressed)))))
    (if (zero? progressed)
        filled
        (loop (+ filled progressed)))))

;; MRV heuristic: the empty cell with the fewest candidates (most
;; constrained first). Returns (values -1 '()) as a dead-end signal when
;; some empty cell has no candidates left.
(define (pick-cell g)
  (define best-i -1)
  (define best-cs '())
  (define dead? #f)
  (for ([i (in-range cell-count)]
        #:when (and (not dead?) (empty-cell? g i)))
    (define cs (candidates-of g i))
    (cond [(null? cs) (set! dead? #t)]
          [(or (= best-i -1) (< (length cs) (length best-cs)))
           (set! best-i i)
           (set! best-cs cs)]))
  (if dead? (values -1 '()) (values best-i best-cs)))

;; Solve g with MRV backtracking search. Returns two values: the solved
;; grid (or #f if g is invalid or unsolvable) and a hash of search stats
;; ('nodes visited, 'max-depth reached). g itself is not mutated; the
;; search runs on a copy.
(define (solve/stats g)
  (define nodes 0)
  (define max-depth 0)
  (define (search g depth)
    (set! nodes (add1 nodes))
    (set! max-depth (max max-depth depth))
    (cond
      [(complete? g) g]
      [else
       (define-values (i cs) (pick-cell g))
       (if (= i -1)
           #f
           (for/or ([v (in-list cs)])
             (grid-set! g i v)
             (define result (search g (add1 depth)))
             (cond [result result]
                   ;; backtrack: undo the guess before trying the next digit
                   [else (grid-set! g i 0) #f])))]))
  (define result (and (valid? g) (search (grid-copy g) 0)))
  (values result (hash 'nodes nodes 'max-depth max-depth)))

;; Like solve/stats, but discards the search stats.
(define (solve g)
  (define-values (result _) (solve/stats g))
  result)

;; Count the solutions of g, stopping early once the count reaches cap
;; (default 2, enough to decide uniqueness). Returns 0 for invalid grids.
(define (count-solutions g [cap 2])
  (define count 0)
  (define (search g)
    (cond
      [(>= count cap) #t]
      [(complete? g) (set! count (add1 count)) #t]
      [else
       (define-values (i cs) (pick-cell g))
       (if (= i -1)
           #f
           ;; for/and short-circuits the whole search once cap is reached
           (for/and ([v (in-list cs)])
             (grid-set! g i v)
             (define keep-going? (search g))
             (grid-set! g i 0)
             keep-going?))]))
  (when (valid? g)
    (search (grid-copy g)))
  count)
