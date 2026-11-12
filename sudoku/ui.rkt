#lang racket

(require racket/gui
         "grid.rkt"
         "solver.rkt"
         "generator.rkt"
         "complexity.rkt")

(provide launch-ui)

(define cell-size 48)
(define margin 12)
(define board-size (+ (* 9 cell-size) (* 2 margin)))

(define given-color (make-object color% 20 40 160))
(define user-color (make-object color% 30 30 30))
(define error-color (make-object color% 200 30 30))
(define highlight-color (make-object color% 255 244 180))
(define selected-color (make-object color% 255 224 102))

(define (cell-index x y)
  (define c (quotient (- x margin) cell-size))
  (define r (quotient (- y margin) cell-size))
  (and (<= 0 r 8) (<= 0 c 8) (+ (* r 9) c)))

(define sudoku-canvas%
  (class canvas%
    (init-field puzzle current selected show-pencils?)
    (define/public (set-puzzle! p)
      (set! puzzle p)
      (set! current (grid-copy p))
      (set! selected #f)
      (send this refresh))
    (define/public (get-current) current)
    (define/public (get-puzzle) puzzle)
    (define/public (set-solution-and-fill! sol)
      (set! current (grid-copy sol))
      (send this refresh))
    (define/public (toggle-pencils!)
      (set! show-pencils? (not show-pencils?))
      (send this refresh))
    (define/override (on-char event)
      (define code (send event get-key-code))
      (cond
        [(and selected (char? code) (char-numeric? code) (>= (- (char->integer code) #x30) 1))
         (define v (- (char->integer code) #x30))
         (when (and (<= 1 v 9) (zero? (grid-ref puzzle selected)))
           (grid-set! current selected v)
           (send this refresh))]
        [(and selected (memv code (list #\0 #\backspace 'delete))
              (zero? (grid-ref puzzle selected)))
         (grid-set! current selected 0)
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
      (when selected
        (send dc set-pen (make-object pen% 0 'transparent))
        (send dc set-brush (make-object brush% highlight-color 'solid))
        (for ([p (in-list (peers-of selected))])
          (send dc draw-rectangle
                (+ margin (* (col-of p) cell-size))
                (+ margin (* (row-of p) cell-size))
                cell-size cell-size))
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
        (send dc set-pen (make-object pen% (if thick? 0 160) w 'solid))
        (send dc draw-line pos margin pos (+ margin (* 9 cell-size)))
        (send dc draw-line margin pos (+ margin (* 9 cell-size)) pos))
      (send dc set-pen (make-object pen% 0 2 'solid))
      (send dc draw-rectangle margin margin (* 9 cell-size) (* 9 cell-size)))
    (define/private (draw-cells dc)
      (define big-font (make-object font% 22 'default 'normal 'bold))
      (define small-font (make-object font% 10 'default 'normal 'normal))
      (for ([i (in-range cell-count)])
        (define x (+ margin (* (col-of i) cell-size)))
        (define y (+ margin (* (row-of i) cell-size)))
        (define pv (grid-ref puzzle i))
        (define cv (grid-ref current i))
        (cond
          [(positive? pv)
           (send dc set-font big-font)
           (send dc set-text-foreground given-color)
           (send dc draw-text (number->string pv)
                 (+ x 16) (+ y 10))]
          [(positive? cv)
           (define conflict?
             (ormap (lambda (p)
                      (and (positive? (grid-ref current p))
                           (= (grid-ref current p) cv)
                           (not (= p i))))
                    (peers-of i)))
           (send dc set-font big-font)
           (send dc set-text-foreground (if conflict? error-color user-color))
           (send dc draw-text (number->string cv)
                 (+ x 16) (+ y 10))]
          [show-pencils?
           (define cs (candidates-of current i))
           (send dc set-font small-font)
           (send dc set-text-foreground (make-object color% 120 120 120))
           (for ([v (in-list cs)]
                 [k (in-naturals)])
             (send dc draw-text (number->string v)
                   (+ x 4 (* (remainder k 3) 15))
                   (+ y 2 (* (quotient k 3) 15))))])))
    (super-new)))

(define (launch-ui)
  (define frame
    (new frame% [label "Puzzle Explorer - Sudoku"] [width (+ board-size 40)] [height (+ board-size 110)]))
  (define board-panel (new vertical-panel% [parent frame] [alignment '(center center)]))
  (define canvas
    (new sudoku-canvas%
         [parent board-panel]
         [puzzle (make-grid)]
         [current (make-grid)]
         [selected #f]
         [show-pencils? #f]
         [min-width board-size]
         [min-height board-size]))
  (define controls (new horizontal-panel% [parent frame] [alignment '(center center)] [stretchable-width #t]))
  (define status (new message% [parent frame] [label "Generate a puzzle to begin"] [min-width 400]))
  (define solution #f)

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
                   (define report (format-analysis (analyze (send canvas get-puzzle))))
                   (define dlg (new dialog% [label "Complexity Analysis"] [parent frame]
                                    [width 420] [height 340]))
                   (define vp (new vertical-panel% [parent dlg] [alignment '(left top)]))
                   (for ([line (in-list (string-split report "\n"))])
                     (new message% [parent vp] [label line]))
                   (send dlg show #t))])
  (new check-box% [parent controls] [label "Pencils"]
       [callback (lambda (b e) (send canvas toggle-pencils!))])

  (new-puzzle 'medium)
  (send frame show #t))
