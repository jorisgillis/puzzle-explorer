#lang racket

(require racket/gui
         "grid.rkt"
         "solver.rkt"
         "generator.rkt"
         "complexity.rkt")

(provide launch-ui
         heat-color)

;; Candidate heat overlay palette (rust/SPEC-UI.md, map #20): one discrete
;; color per candidate count 1..9 (few = green, many = red); 0 = error red.
(define (heat-color n)
  (case n
    [(0) (make-object color% 200 30 30)]
    [(1) (make-object color% 150 233 150)]
    [(2) (make-object color% 171 233 150)]
    [(3) (make-object color% 191 233 150)]
    [(4) (make-object color% 212 233 150)]
    [(5) (make-object color% 233 233 150)]
    [(6) (make-object color% 233 212 150)]
    [(7) (make-object color% 233 191 150)]
    [(8) (make-object color% 233 171 150)]
    [else (make-object color% 233 150 150)]))

(define cell-size 48)
(define margin 12)
(define board-size (+ (* 9 cell-size) (* 2 margin)))

(define given-color (make-object color% 20 40 160))
(define user-color (make-object color% 30 30 30))
(define error-color (make-object color% 200 30 30))
(define highlight-color (make-object color% 255 244 180))
(define selected-color (make-object color% 255 224 102))

;; pen% expects a color object, not a gray level integer
(define (gray-level n) (make-object color% n n n))

(define (cell-index x y)
  (define c (quotient (- x margin) cell-size))
  (define r (quotient (- y margin) cell-size))
  (and (<= 0 r 8) (<= 0 c 8) (+ (* r 9) c)))

(define sudoku-canvas%
  (class canvas%
    (init-field puzzle current selected show-pencils? [show-heat? #f] [on-change (lambda () (void))])
    (define/public (set-on-change! cb) (set! on-change cb))
    (define/public (set-puzzle! p)
      (set! puzzle p)
      (set! current (grid-copy p))
      (set! selected #f)
      (on-change)
      (send this refresh))
    (define/public (get-current) current)
    (define/public (get-puzzle) puzzle)
    (define/public (set-solution-and-fill! sol)
      (set! current (grid-copy sol))
      (on-change)
      (send this refresh))
    (define/public (toggle-pencils!)
      (set! show-pencils? (not show-pencils?))
      (send this refresh))
    (define/public (toggle-heat!)
      (set! show-heat? (not show-heat?))
      (send this refresh))
    (define/override (on-char event)
      (define code (send event get-key-code))
      (cond
        [(and selected (char? code) (char-numeric? code) (>= (- (char->integer code) #x30) 1))
         (define v (- (char->integer code) #x30))
         (when (and (<= 1 v 9) (zero? (grid-ref puzzle selected)))
           (grid-set! current selected v)
           (on-change)
           (send this refresh))]
        [(and selected (memv code (list #\0 #\backspace 'delete))
              (zero? (grid-ref puzzle selected)))
         (grid-set! current selected 0)
         (on-change)
         (send this refresh)]
        [(memv code '(left right up down))
         (move-selection code)]
        [else (void)]))
    (define/private (move-selection code)
      (define i (or selected 0))
      (define r (row-of i))
      (define c (col-of i))
      (define nr (case code [(up) (max 0 (sub1 r))] [(down) (min 8 (add1 r))] [else r]))
      (define nc (case code [(left) (max 0 (sub1 c))] [(right) (min 8 (add1 c))] [else c]))
      (set! selected (+ (* nr 9) nc))
      (send this refresh))
    (define/override (on-event event)
      (when (send event button-down?)
        (define i (cell-index (send event get-x) (send event get-y)))
        (when i
          (set! selected i)
          (send this refresh))))
    (define/override (on-paint)
      (define dc (send this get-dc))
      (send dc set-background (make-object color% 255 255 255))
      (send dc clear)
      (draw-highlights dc)
      (draw-grid-lines dc)
      (draw-cells dc))
    (define/private (draw-highlights dc)
      (send dc set-pen (make-object pen% (gray-level 0) 1 'transparent))
      (when selected
        (send dc set-brush (make-object brush% highlight-color 'solid))
        (for ([p (in-list (peers-of selected))])
          (send dc draw-rectangle
                (+ margin (* (col-of p) cell-size))
                (+ margin (* (row-of p) cell-size))
                cell-size cell-size)))
      ;; heat overlay on empty cells: over the peer highlight, under the
      ;; selected cell (rust/SPEC-UI.md, map #20)
      (when show-heat?
        (for ([i (in-range cell-count)])
          (when (zero? (grid-ref current i))
            (send dc set-brush
                  (make-object brush% (heat-color (length (candidates-of current i))) 'solid))
            (send dc draw-rectangle
                  (+ margin (* (col-of i) cell-size))
                  (+ margin (* (row-of i) cell-size))
                  cell-size cell-size))))
      (when selected
        (send dc set-brush (make-object brush% selected-color 'solid))
        (send dc draw-rectangle
              (+ margin (* (col-of selected) cell-size))
              (+ margin (* (row-of selected) cell-size))
              cell-size cell-size)))
    (define/private (draw-grid-lines dc)
      (for ([k (in-range 10)])
        (define thick? (zero? (remainder k 3)))
        (define w (if thick? 2 1))
        (define pos (+ margin (* k cell-size)))
        (send dc set-pen (make-object pen% (gray-level (if thick? 0 160)) w 'solid))
        (send dc draw-line pos margin pos (+ margin (* 9 cell-size)))
        (send dc draw-line margin pos (+ margin (* 9 cell-size)) pos))
      (send dc set-brush (make-object brush% (gray-level 0) 'transparent))
      (send dc set-pen (make-object pen% (gray-level 0) 2 'solid))
      (send dc draw-rectangle margin margin (* 9 cell-size) (* 9 cell-size)))
    (define/private (draw-centered dc str cell-x cell-y cell-w cell-h)
      (define-values (tw th _asc _desc) (send dc get-text-extent str))
      (send dc draw-text str
            (+ cell-x (/ (- cell-w tw) 2))
            (+ cell-y (/ (- cell-h th) 2))))
    (define/private (draw-cells dc)
      (define big-font (make-object font% 22 'default 'normal 'bold))
      (define small-font (make-object font% 10 'default 'normal 'normal))
      (define pencil-cell (quotient cell-size 3))
      (for ([i (in-range cell-count)])
        (define x (+ margin (* (col-of i) cell-size)))
        (define y (+ margin (* (row-of i) cell-size)))
        (define pv (grid-ref puzzle i))
        (define cv (grid-ref current i))
        (cond
          [(positive? pv)
           (send dc set-font big-font)
           (send dc set-text-foreground given-color)
           (draw-centered dc (number->string pv) x y cell-size cell-size)]
          [(positive? cv)
           (define conflict?
             (ormap (lambda (p)
                      (and (positive? (grid-ref current p))
                           (= (grid-ref current p) cv)
                           (not (= p i))))
                    (peers-of i)))
           (send dc set-font big-font)
           (send dc set-text-foreground (if conflict? error-color user-color))
           (draw-centered dc (number->string cv) x y cell-size cell-size)]
          [show-pencils?
           (define cs (candidates-of current i))
           (send dc set-font small-font)
           (send dc set-text-foreground (make-object color% 120 120 120))
           (for ([v (in-list cs)]
                 [k (in-naturals)])
             (draw-centered dc (number->string v)
                            (+ x (* (remainder k 3) pencil-cell))
                            (+ y (* (quotient k 3) pencil-cell))
                            pencil-cell pencil-cell))])))
    (super-new)))

(define (launch-ui)
  (define frame
    (new frame% [label "Puzzle Explorer - Sudoku"] [width (+ board-size 40)] [height (+ board-size 220)]))
  (define board-panel (new vertical-panel% [parent frame] [alignment '(center center)]))
  (define canvas
    (new sudoku-canvas%
         [parent board-panel]
         [puzzle (make-grid)]
         [current (make-grid)]
         [selected #f]
         [show-pencils? #f]
         [show-heat? #f]
         [min-width board-size]
         [min-height board-size]))
  (define controls (new horizontal-panel% [parent frame] [alignment '(center center)] [stretchable-width #t]))
  (define status (new message% [parent frame] [label "Generate a puzzle to begin"] [min-width 400]))
  (define solution #f)

  ;; Live analysis panel, hosted in the main window (dialog% show can hang
  ;; on Wayland setups). Recomputes the report from the current board
  ;; (givens + your entries) every time the grid changes.
  (define analysis-text (new text%))
  (define analysis-view
    (new editor-canvas% [parent frame] [editor analysis-text]
         [min-width 460] [min-height 132] [style '(no-focus no-hscroll)]))
  (send analysis-text lock #t)
  (send analysis-view show #f)

  (define (analysis-report)
    (define cur (send canvas get-current))
    (string-append
     (format "Live analysis of the current board (~a)\n"
             (cond [(not (valid? cur)) "conflicting entries present"]
                   [(and solution (equal? cur solution)) "solved correctly"]
                   [else "no conflicts so far"]))
     (format-analysis (analyze cur))))

  (define (update-analysis!)
    (when (send analysis-view is-shown?)
      (send analysis-text lock #f) ; lock makes the editor ignore even programmatic edits
      (send analysis-text begin-edit-sequence)
      (send analysis-text delete 0 (send analysis-text last-position))
      (send analysis-text insert (analysis-report) 0)
      (send analysis-text end-edit-sequence)
      (send analysis-text lock #t)))

  (send canvas set-on-change! (lambda () (update-analysis!)))

  (define (new-puzzle difficulty)
    (define-values (puzzle sol) (generate-puzzle difficulty))
    (set! solution sol)
    (send canvas set-puzzle! puzzle)
    (send status set-label (format "New ~a puzzle (~a givens)" difficulty
                                   (count (lambda (v) (not (zero? v))) (vector->list puzzle)))))

  (define (difficulty-button label difficulty)
    (new button% [parent controls] [label label]
         [callback (lambda (b e) (new-puzzle difficulty))]))

  (difficulty-button "New Easy" 'easy)
  (difficulty-button "New Medium" 'medium)
  (difficulty-button "New Hard" 'hard)

  (new button% [parent controls] [label "Solve"]
       [callback (lambda (b e)
                   (when solution
                     (send canvas set-solution-and-fill! solution)
                     (send status set-label "Solved.")))])
  (new button% [parent controls] [label "Hint"]
       [callback (lambda (b e)
                   (when solution
                     (define cur (send canvas get-current))
                     (define target
                       (or (for/first ([i (in-range cell-count)]
                                       #:when (zero? (grid-ref cur i)))
                             i)
                           (for/first ([i (in-range cell-count)]
                                       #:when (not (= (grid-ref cur i) (grid-ref solution i))))
                             i)))
                     (when target
                       (define puz (send canvas get-puzzle))
                       (unless (positive? (grid-ref puz target))
                         (grid-set! cur target (grid-ref solution target))
                         (update-analysis!)
                         (send canvas refresh))
                       (send status set-label (format "Hint: cell ~a = ~a"
                                                      (add1 target) (grid-ref solution target))))))])
  (new button% [parent controls] [label "Check"]
       [callback (lambda (b e)
                   (define cur (send canvas get-current))
                   (cond
                     [(not (valid? cur)) (send status set-label "There are conflicting entries.")]
                     [(and solution (equal? cur solution)) (send status set-label "Solved correctly!")]
                     [else (send status set-label "No conflicts so far.")]))])
  (new button% [parent controls] [label "Analyze"]
       [callback (lambda (b e)
                   (if (send analysis-view is-shown?)
                       (send analysis-view show #f)
                       (begin (send analysis-view show #t)
                              (update-analysis!))))])
  (new check-box% [parent controls] [label "Pencils"]
       [callback (lambda (b e) (send canvas toggle-pencils!))])
  (new check-box% [parent controls] [label "Heat"]
       [callback (lambda (b e) (send canvas toggle-heat!))])

  (new-puzzle 'medium)
  (send frame show #t))
