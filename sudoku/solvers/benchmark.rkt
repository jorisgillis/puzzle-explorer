#lang racket

(require racket/cmdline
         racket/file
         "../grid.rkt"
         "../solvers.rkt")

;; Benchmark harness for the solver gallery (map #33, tickets #37/#41).
;; Protocol per puzzle x solver: 1 untimed warmup solve (captures nodes /
;; max depth / DLX link updates), then 5 timed runs, median wall time
;; reported. Validation: every solver must return a complete, valid
;; solution consistent with the givens, and (checked across the CSV) the
;; identical solution across solvers. Prints a markdown results table per
;; set to stdout and appends raw per-run + per-puzzle rows to a CSV
;; (default docs/assets/benchmark-results.csv).
;;
;; Usage: racket sudoku/solvers/benchmark.rkt [--set top95|royle17|extremes]
;;        [--skip SOLVER]... [--runs N] [--limit N] [--offset N] [--csv PATH]

(define repo-root
  (simplify-path (build-path (path-only (find-system-path 'run-file)) 'up 'up)))

(define fixture-dir (build-path repo-root "tests" "fixtures"))

(define sets
  '(("top95" . "bench-top95.txt")
    ("royle17" . "bench-royle17.txt")
    ("extremes" . "bench-extremes.txt")))

(define puzzle-limit (box #f))
(define puzzle-offset (box 0))
(define skipped-solvers (box '()))

(define (load-puzzles file)
  (define puzzles
    (for/list ([line (in-list (file->lines (build-path fixture-dir file)))]
               #:when (and (positive? (string-length (string-trim line)))
                           (not (string-prefix? (string-trim line) "#"))))
      (string->grid (string-trim line))))
  (define dropped (drop puzzles (unbox puzzle-offset)))
  (if (unbox puzzle-limit)
      (take dropped (unbox puzzle-limit))
      dropped))

(define (median xs)
  (define s (sort xs <))
  (define n (length s))
  (if (odd? n)
      (list-ref s (quotient n 2))
      (/ (+ (list-ref s (quotient n 2)) (list-ref s (sub1 (quotient n 2)))) 2)))

(define (p95 xs)
  (define s (sort xs <))
  (list-ref s (sub1 (max 1 (exact-ceiling (* 0.95 (length s)))))))

;; one puzzle x one solver: warmup (captures stats) + timed runs
(define (bench-puzzle solver-entry g runs)
  (match-define (list name solve-fn stats-fn _count-fn) solver-entry)
  (define-values (warm-result warm-stats) (stats-fn g))
  (define times
    (for/list ([_ (in-range runs)])
      (define t0 (current-inexact-milliseconds))
      (define result (solve-fn g))
      (define t1 (current-inexact-milliseconds))
      (unless (equal? (and result (grid->string result))
                      (and warm-result (grid->string warm-result)))
        (error 'benchmark "solver ~a produced different solutions across runs" name))
      (- t1 t0)))
  (hash 'solver name
        'solution (and warm-result (grid->string warm-result))
        'median-ms (median times)
        'nodes (hash-ref warm-stats 'nodes)
        'max-depth (hash-ref warm-stats 'max-depth)
        'link-updates (hash-ref warm-stats 'link-updates 0)
        'times times))

(define (check-solution! g results)
  ;; each solver's solution is complete/valid and consistent with givens
  (for ([r (in-list results)])
    (define sol-str (hash-ref r 'solution))
    (unless sol-str
      (error 'benchmark "no solution produced"))
    (define sol (string->grid sol-str))
    (unless (and (complete? sol) (valid? sol))
      (error 'benchmark "invalid solution produced"))
    (for ([i (in-range cell-count)]
          #:when (positive? (grid-ref g i)))
      (unless (= (grid-ref sol i) (grid-ref g i))
        (error 'benchmark "solution contradicts a given")))))

(define (aggregate set-name per-puzzle solver-name)
  (define rs
    (for*/list ([pp (in-list per-puzzle)]
                [r (in-list (hash-ref pp 'results))]
                #:when (equal? (hash-ref r 'solver) solver-name))
      r))
  (define meds (map (lambda (r) (hash-ref r 'median-ms)) rs))
  (define (avg key) (/ (apply + (map (lambda (r) (hash-ref r key)) rs)) (length rs)))
  (hash 'set set-name
        'solver solver-name
        'puzzles (length rs)
        'avg-ms (/ (apply + meds) (length meds))
        'p95-ms (p95 meds)
        'max-ms (apply max meds)
        'avg-nodes (avg 'nodes)
        'max-depth (apply max (map (lambda (r) (hash-ref r 'max-depth)) rs))
        'avg-updates (avg 'link-updates)))

(define csv-port #f)

(define (csv-open path)
  (make-directory* (path-only path))
  (define new? (not (file-exists? path)))
  (set! csv-port (open-output-file path #:exists 'append))
  (when new?
    (displayln "lang,set,puzzle,solver,run,wall_ms" csv-port)
    (displayln "# aggregates: lang,set,puzzle,solver,median_ms,nodes,max_depth,link_updates,solution" csv-port)))

(define (write-csv-row out r)
  (for ([t (in-list (hash-ref r 'times))] [i (in-naturals)])
    (fprintf out "racket,~a,~a,~a,~a,~a\n"
             (hash-ref r 'set) (hash-ref r 'puzzle) (hash-ref r 'solver) i (fmt t)))
  (fprintf out "racket,~a,~a,~a,#,~a,~a,~a,~a,~a\n"
           (hash-ref r 'set) (hash-ref r 'puzzle) (hash-ref r 'solver)
           (fmt (hash-ref r 'median-ms)) (hash-ref r 'nodes)
           (hash-ref r 'max-depth) (hash-ref r 'link-updates)
           (hash-ref r 'solution))
  (flush-output out))

(define (fmt x) (~r x #:precision 3))

(define (bench-set set-name file runs)
  (define puzzles (load-puzzles file))
  (define solvers
    (filter (lambda (s) (not (member (car s) (unbox skipped-solvers))))
            (all-solvers)))
  (define per-puzzle '())
  (define flat '())
  (for ([s (in-list solvers)])
    (for ([g (in-list puzzles)] [local-idx (in-naturals)])
      (define idx (+ (unbox puzzle-offset) local-idx))
      (define r (hash-set (hash-set (bench-puzzle s g runs) 'set set-name) 'puzzle idx))
      (check-solution! g (list r))
      (set! per-puzzle (append per-puzzle (list (hash 'puzzle idx 'results (list r)))))
      (set! flat (append flat (list r)))
      (write-csv-row csv-port r)))
  (values (for/list ([s (in-list solvers)])
            (aggregate set-name per-puzzle (car s)))
          flat))

(define (print-table rows extremes-rows)
  (displayln "| fixture set | solver | puzzles | avg wall ms | p95 wall ms | max wall ms | avg nodes | max depth | avg link updates |")
  (displayln "|---|---|---|---|---|---|---|---|---|")
  (for ([r (in-list rows)])
    (printf "| ~a | ~a | ~a | ~a | ~a | ~a | ~a | ~a | ~a |\n"
            (hash-ref r 'set) (hash-ref r 'solver) (hash-ref r 'puzzles)
            (fmt (hash-ref r 'avg-ms)) (fmt (hash-ref r 'p95-ms)) (fmt (hash-ref r 'max-ms))
            (fmt (hash-ref r 'avg-nodes)) (hash-ref r 'max-depth)
            (if (equal? (hash-ref r 'solver) 'dlx) (fmt (hash-ref r 'avg-updates)) "-")))
  (when extremes-rows
    (displayln "")
    (displayln "Per-puzzle rows for the extremes set:")
    (displayln "| puzzle | solver | median wall ms | nodes | max depth | link updates |")
    (displayln "|---|---|---|---|---|---|")
    (for* ([r (in-list extremes-rows)])
      (printf "| #~a | ~a | ~a | ~a | ~a | ~a |\n"
              (hash-ref r 'puzzle) (hash-ref r 'solver)
              (fmt (hash-ref r 'median-ms)) (hash-ref r 'nodes) (hash-ref r 'max-depth)
              (hash-ref r 'link-updates)))))

(define selected-sets (box '()))
(define runs (box 5))
(define csv-path (box (build-path repo-root "docs" "assets" "benchmark-results.csv")))

(command-line
 #:multi
 ["--set" s "fixture set to run (top95|royle17|extremes); repeatable, default all"
  (set-box! selected-sets (cons s (unbox selected-sets)))]
 ["--skip" s "skip this solver (repeatable)"
  (set-box! skipped-solvers (cons (string->symbol s) (unbox skipped-solvers)))]
 #:once-each
 ["--runs" n "timed runs per puzzle (default 5)" (set-box! runs (string->number n))]
 ["--csv" p "CSV output path" (set-box! csv-path (string->path p))]
 ["--limit" n "run only N puzzles of each set (after offset)" (set-box! puzzle-limit (string->number n))]
 ["--offset" n "skip the first N puzzles of each set" (set-box! puzzle-offset (string->number n))])

(define chosen (if (null? (unbox selected-sets)) (map car sets) (unbox selected-sets)))
(csv-open (unbox csv-path))
(for ([set (in-list sets)]
      #:when (member (car set) chosen))
  (define-values (rows flat) (bench-set (car set) (cdr set) (unbox runs)))
  (print-table rows (and (equal? (car set) "extremes") flat)))
(when csv-port (close-output-port csv-port))
(printf "CSV rows appended to ~a\n" (unbox csv-path))
