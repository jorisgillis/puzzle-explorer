#lang racket
;; SCRATCH experiment for wayfinder research ticket #27 (research only).
;; Racket cross-check: singles-only (propagation fixpoint) solvability rate of
;; generated puzzles per difficulty, plus generate+analyze throughput in the
;; Racket CLI implementation. Run from repo root:
;;   racket research/scratch/technique-cost.rkt

(require (only-in "../../sudoku/grid.rkt" grid-copy complete?)
         "../../sudoku/generator.rkt"
         "../../sudoku/solver.rkt"
         "../../sudoku/complexity.rkt")

(define args (current-command-line-arguments))
(define n-per-difficulty (if (>= (vector-length args) 2) (string->number (vector-ref args 1)) 30))
(define difficulties (if (>= (vector-length args) 1) (list (string->symbol (vector-ref args 0))) '(easy medium hard)))

(define (run difficulty)
  (define t0 (current-inexact-milliseconds))
  (define singles-only 0)
  (define unique 0)
  (define nodes-total 0)
  (for ([_ (in-range n-per-difficulty)])
    (define-values (puzzle solution) (generate-puzzle difficulty))
    (define w (grid-copy puzzle))
    (propagate w)
    (when (complete? w) (set! singles-only (add1 singles-only)))
    (when (= (count-solutions puzzle 2) 1) (set! unique (add1 unique)))
    (define-values (_sol stats) (solve/stats puzzle))
    (set! nodes-total (+ nodes-total (hash-ref stats 'nodes))))
  (define dt (- (current-inexact-milliseconds) t0))
  (printf "~a: n=~a  singles-only(propagate fixpoint)=~a (~a%)  unique=~a  avg nodes=~a  generate+analyze ~a ms/puz (~a puz/s)~n"
          difficulty
          n-per-difficulty
          singles-only
          (round (* 100 (/ singles-only n-per-difficulty)))
          unique
          (round (/ nodes-total n-per-difficulty))
          (round (/ dt n-per-difficulty))
          (round (/ (* 1000 n-per-difficulty) dt))))

(for ([d (in-list difficulties)])
  (run d))
