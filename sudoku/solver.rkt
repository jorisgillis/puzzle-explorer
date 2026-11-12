#lang racket

(require "grid.rkt")

(provide candidates-of
         propagate
         solve
         solve/stats
         count-solutions)

(define (candidates-of g i)
  (define used
    (for/fold ([s (set)])
              ([p (in-list (peers-of i))])
      (define v (grid-ref g p))
      (if (zero? v) s (set-add s v))))
  (for/list ([v (in-range 1 10)] #:unless (set-member? used v)) v))

(define (propagate g)
  (let loop ([filled 0])
    (define progressed 0)
    (for ([i (in-range cell-count)]
          #:when (empty-cell? g i))
      (define cs (candidates-of g i))
      (when (= (length cs) 1)
        (grid-set! g i (first cs))
        (set! progressed (add1 progressed))))
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
                   [else (grid-set! g i 0) #f])))]))
  (define result (and (valid? g) (search (grid-copy g) 0)))
  (values result (hash 'nodes nodes 'max-depth max-depth)))

(define (solve g)
  (define-values (result _) (solve/stats g))
  result)

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
           (for/and ([v (in-list cs)])
             (grid-set! g i v)
             (define keep-going? (search g))
             (grid-set! g i 0)
             keep-going?))]))
  (when (valid? g)
    (search (grid-copy g)))
  count)
