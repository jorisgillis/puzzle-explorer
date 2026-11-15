#lang racket

(require "../grid.rkt")

(provide solver-name
         solve
         solve/stats
         count-solutions)

(define solver-name 'dlx)

;; DLX solver: Knuth's Algorithm X with Dancing Links (arXiv:cs/0011047)
;; over the canonical exact-cover matrix of 9x9 sudoku: 729 placement rows
;; (r,c,d) x 324 primary columns = four constraint groups of 81:
;;   rc(r,c) cell, rn(r,d) row-digit, cn(c,d) column-digit, bn(b,d) box-digit.
;; Each row has exactly four 1s, one per group. Givens are pre-placed by
;; selecting their placement row (covering its four columns). The search
;; branches on the minimum-size column (MRV over constraints); cover and
;; uncover are O(1) per link via circular doubly linked L/R/U/D lists.
;; Stats are the uniform 'nodes and 'max-depth plus 'link-updates, Knuth's
;; machine-independent cost metric. Invalid input never enters the search:
;; (values #f zero-stats) / 0.

(define col-count 324)
(define row-count 729)
(define header 0)
(define first-node (+ 1 col-count)) ; node 0 = header, nodes 1..324 = columns

(define (rc-col r c) (+ 1 (* r 9) c))
(define (rn-col r d) (+ 1 81 (* r 9) (- d 1)))
(define (cn-col c d) (+ 1 162 (* c 9) (- d 1)))
(define (bn-col b d) (+ 1 243 (* b 9) (- d 1)))

;; placement row id for cell (r,c) with digit d
(define (placement-row r c d) (+ (* (+ (* r 9) c) 9) (- d 1)))

(struct state (L R U D S C node-row rid-node ; matrix links, column sizes, node->col, node->row, row->first node
               chosen                       ; stack of selected placement rows
               nodes max-depth updates      ; stats
               mode cap count solution)     ; 'solve or 'count
  #:mutable)

(define (build g mode cap)
  (define size (+ first-node (* 4 row-count)))
  (define L (make-vector size header))
  (define R (make-vector size header))
  (define U (make-vector size header))
  (define D (make-vector size header))
  (define S (make-vector (+ 1 col-count) 0))
  (define C (make-vector size header))
  (define node-row (make-vector size 0))
  (define rid-node (make-vector row-count 0))
  ;; header <-> column headers, circular horizontally
  (let loop ([c col-count])
    (when (positive? c)
      (vector-set! L c (sub1 c))
      (vector-set! R (sub1 c) c)
      (loop (sub1 c))))
  (vector-set! L header col-count)
  (vector-set! R col-count header)
  ;; column vertical lists start empty: U[h] = D[h] = h
  (for ([c (in-range 1 (add1 col-count))])
    (vector-set! U c c)
    (vector-set! D c c))
  ;; placement rows: four nodes each, circular horizontally, appended to
  ;; their column's vertical list
  (define next first-node)
  (for* ([r (in-range 9)] [c (in-range 9)] [d (in-range 1 10)])
    (define rid (placement-row r c d))
    (define cols (list (rc-col r c)
                       (rn-col r d)
                       (cn-col c d)
                       (bn-col (box-of (+ (* r 9) c)) d)))
    (define ns
      (for/list ([col (in-list cols)])
        (define n next)
        (set! next (add1 next))
        (vector-set! C n col)
        (vector-set! node-row n rid)
        (vector-set! S col (add1 (vector-ref S col)))
        ;; append n to column col's vertical list (insert just above header)
        (define last (vector-ref U col))
        (vector-set! U n last)
        (vector-set! D n col)
        (vector-set! U col n)
        (vector-set! D last n)
        n))
    (vector-set! rid-node rid (first ns))
    ;; circular horizontal links within the row
    (for ([n (in-list ns)]
          [n2 (in-list (append (cdr ns) (list (first ns))))])
      (vector-set! R n n2)
      (vector-set! L n2 n)))
  (define st (state L R U D S C node-row rid-node '() 0 0 0 mode cap 0 #f))
  ;; pre-place givens: select each given's placement row, covering ALL
  ;; four of its columns (select-row skips the branch column, which the
  ;; search already covered — not true here)
  (for ([i (in-range cell-count)])
    (define v (grid-ref g i))
    (unless (zero? v)
      (define rid (placement-row (row-of i) (col-of i) v))
      (place-given st rid)))
  st)

;; select a placement row as a given: push it and cover all four of its
;; columns (unlike select-row, which leaves the branch column to the caller)
(define (place-given st rid)
  (set-state-chosen! st (cons rid (state-chosen st)))
  (define n0 (vector-ref (state-rid-node st) rid))
  (let loop ([j n0])
    (cover st (vector-ref (state-C st) j))
    (define j2 (vector-ref (state-R st) j))
    (unless (= j2 n0)
      (loop j2))))

(define (cover st c)
  (define L (state-L st)) (define R (state-R st))
  (define U (state-U st)) (define D (state-D st))
  (define S (state-S st)) (define C (state-C st))
  (vector-set! L (vector-ref R c) (vector-ref L c))
  (vector-set! R (vector-ref L c) (vector-ref R c))
  (let loop-i ([i (vector-ref D c)])
    (unless (= i c)
      (let loop-j ([j (vector-ref R i)])
        (unless (= j i)
          (vector-set! D (vector-ref U j) (vector-ref D j))
          (vector-set! U (vector-ref D j) (vector-ref U j))
          (define col (vector-ref C j))
          (vector-set! S col (sub1 (vector-ref S col)))
          (set-state-updates! st (add1 (state-updates st)))
          (loop-j (vector-ref R j))))
      (loop-i (vector-ref D i)))))

(define (uncover st c)
  (define L (state-L st)) (define R (state-R st))
  (define U (state-U st)) (define D (state-D st))
  (define S (state-S st)) (define C (state-C st))
  (let loop-i ([i (vector-ref U c)])
    (unless (= i c)
      (let loop-j ([j (vector-ref L i)])
        (unless (= j i)
          (define col (vector-ref C j))
          (vector-set! S col (add1 (vector-ref S col)))
          (vector-set! U (vector-ref D j) j)
          (vector-set! D (vector-ref U j) j)
          (set-state-updates! st (add1 (state-updates st)))
          (loop-j (vector-ref L j))))
      (loop-i (vector-ref U i))))
  (vector-set! L (vector-ref R c) c)
  (vector-set! R (vector-ref L c) c))

;; select a row: cover every column it touches, remember it on the stack
(define (select-row st i)
  (set-state-chosen! st (cons (vector-ref (state-node-row st) i) (state-chosen st)))
  (let loop ([j (vector-ref (state-R st) i)])
    (unless (= j i)
      (cover st (vector-ref (state-C st) j))
      (loop (vector-ref (state-R st) j)))))

(define (deselect-row st i)
  (let loop ([j (vector-ref (state-L st) i)])
    (unless (= j i)
      (uncover st (vector-ref (state-C st) j))
      (loop (vector-ref (state-L st) j))))
  (set-state-chosen! st (cdr (state-chosen st))))

;; the uncovered column with the fewest remaining rows (min S = MRV)
(define (min-column st)
  (define R (state-R st)) (define S (state-S st))
  (define best header)
  (define best-s +inf.0)
  (let loop ([c (vector-ref R header)])
    (unless (= c header)
      (define s (vector-ref S c))
      (when (< s best-s)
        (set! best c)
        (set! best-s s))
      (loop (vector-ref R c))))
  best)

(define (extract-solution st)
  (define g (make-grid))
  (for ([rid (in-list (state-chosen st))])
    (define r (quotient rid 81))
    (define rem (remainder rid 81))
    (grid-set! g (+ (* r 9) (quotient rem 9)) (add1 (remainder rem 9))))
  g)

;; Returns #t when the search should stop early: solution found in 'solve
;; mode, cap reached in 'count mode.
(define (search st depth)
  (set-state-nodes! st (add1 (state-nodes st)))
  (set-state-max-depth! st (max depth (state-max-depth st)))
  (if (= (vector-ref (state-R st) header) header)
      ;; every column covered exactly once: a complete grid
      (cond
        [(eq? (state-mode st) 'count)
         (set-state-count! st (add1 (state-count st)))
         (>= (state-count st) (state-cap st))]
        [else
         (set-state-solution! st (extract-solution st))
         #t])
      (let ([c (min-column st)])
        (cover st c)
        (define stop? (search-column st c depth))
        (uncover st c)
        stop?)))

;; Try every row of column c in turn. Returns #t when the search should
;; stop early: solution found in 'solve mode, cap reached in 'count mode.
(define (search-column st c depth)
  (let loop ([i (vector-ref (state-D st) c)])
    (if (= i c)
        #f
        (let ()
          (select-row st i)
          (define stop? (search st (add1 depth)))
          (deselect-row st i)
          (if stop? #t (loop (vector-ref (state-D st) i)))))))

(define (solve/stats g)
  (if (valid? g)
      (let ([st (build g 'solve 0)])
        (search st 0)
        (values (state-solution st)
                (hash 'nodes (state-nodes st)
                      'max-depth (state-max-depth st)
                      'link-updates (state-updates st))))
      (values #f (hash 'nodes 0 'max-depth 0 'link-updates 0))))

(define (solve g)
  (define-values (result _) (solve/stats g))
  result)

(define (count-solutions g [cap 2])
  (if (valid? g)
      (let ([st (build g 'count cap)])
        (search st 0)
        (state-count st))
      0))
