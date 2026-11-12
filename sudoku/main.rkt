#lang racket

(require "grid.rkt"
         "solver.rkt"
         "generator.rkt"
         "complexity.rkt")

(define (launch-ui)
  ((dynamic-require "ui.rkt" 'launch-ui)))

(define (usage)
  (displayln "Usage:
  racket main.rkt ui                        launch the GUI
  racket main.rkt generate <easy|medium|hard>
  racket main.rkt solve <81-char-string>
  racket main.rkt analyze <81-char-string>"))

(define args (vector->list (current-command-line-arguments)))

(define (parse-puzzle s)
  (string->grid s))

(cond
  [(null? args) (launch-ui)]
  [(string=? (first args) "ui") (launch-ui)]
  [(and (= (length args) 2) (string=? (first args) "generate"))
   (define difficulty (string->symbol (second args)))
   (define-values (puzzle solution) (generate-puzzle difficulty))
   (displayln (grid->pretty-string puzzle))
   (displayln (grid->string puzzle))
   (displayln (format-analysis (analyze puzzle)))]
  [(and (= (length args) 2) (string=? (first args) "solve"))
   (define result (solve (parse-puzzle (second args))))
   (if result
       (displayln (grid->pretty-string result))
       (displayln "No solution."))]
  [(and (= (length args) 2) (string=? (first args) "analyze"))
   (displayln (format-analysis (analyze (parse-puzzle (second args)))))]
  [else (usage)])
