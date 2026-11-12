#lang racket

(provide cell-count
         row-of col-of box-of
         rows cols boxes units
         peers-of
         make-grid grid-copy grid-ref grid-set!
         empty-cell? valid? complete?
         grid->string string->grid grid->pretty-string)

(define cell-count 81)

(define (row-of i) (quotient i 9))
(define (col-of i) (remainder i 9))
(define (box-of i)
  (+ (* 3 (quotient (row-of i) 3)) (quotient (col-of i) 3)))

(define rows
  (for/list ([r (in-range 9)])
    (for/list ([c (in-range 9)]) (+ (* r 9) c))))

(define cols
  (for/list ([c (in-range 9)])
    (for/list ([r (in-range 9)]) (+ (* r 9) c))))

(define boxes
  (for/list ([b (in-range 9)])
    (for*/list ([dr (in-range 3)] [dc (in-range 3)])
      (+ (* (+ (* 3 (quotient b 3)) dr) 9) (+ (* 3 (remainder b 3)) dc)))))

(define units (append rows cols boxes))

(define peers-table
  (for/vector ([i (in-range cell-count)])
    (list->vector
     (remove i (remove-duplicates
                (append (list-ref rows (row-of i))
                        (list-ref cols (col-of i))
                        (list-ref boxes (box-of i))))))))

(define (peers-of i) (vector->list (vector-ref peers-table i)))

(define (make-grid) (make-vector cell-count 0))

(define (grid-copy g) (vector-copy g))

(define (grid-ref g i) (vector-ref g i))

(define (grid-set! g i v) (vector-set! g i v))

(define (empty-cell? g i) (zero? (vector-ref g i)))

(define (valid? g)
  (andmap (lambda (unit)
            (let loop ([cells unit] [seen (set)])
              (cond [(null? cells) #t]
                    [else
                     (define v (vector-ref g (car cells)))
                     (and (or (zero? v) (not (set-member? seen v)))
                          (loop (cdr cells) (if (zero? v) seen (set-add seen v))))])))
          units))

(define (complete? g)
  (and (andmap (lambda (v) (and (integer? v) (<= 1 v 9))) (vector->list g))
       (valid? g)))

(define (grid->string g)
  (list->string
   (for/list ([v (in-vector g)])
     (if (zero? v) #\0 (integer->char (+ v (char->integer #\0)))))))

(define (string->grid s)
  (define allowed (string->list "0123456789."))
  (define clean (for/list ([c (in-string s)] #:when (memv c allowed)) c))
  (unless (= (length clean) cell-count)
    (raise-argument-error 'string->grid "81-character string of digits 0-9" s))
  (list->vector
   (for/list ([c (in-list clean)])
     (cond [(char=? c #\.) 0]
           [else (- (char->integer c) #x30)]))))

(define (grid->pretty-string g)
  (define h-sep "+-------+-------+-------+")
  (define (cell-str i)
    (define v (vector-ref g i))
    (if (zero? v) "." (string (integer->char (+ v #x30)))))
  (define (row-line r)
    (string-append
     "| "
     (string-join
      (for/list ([b (in-range 3)])
        (string-join
         (for/list ([c (in-range 3)])
           (cell-str (+ (* r 9) (* b 3) c)))
         " "))
      " | ")
     " |"))
  (string-append
   (string-join
    (for/list ([r (in-range 9)])
      (string-append
       (if (zero? (remainder r 3)) (string-append h-sep "\n") "")
       (row-line r)))
    "\n")
   "\n" h-sep))
