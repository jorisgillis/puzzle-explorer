#lang racket

(require rackunit
         "../sudoku/grid.rkt")

(define solved-string
  "534678912672195348198342567859761423426853791713924856961537284287419635345286179")

(define solved (string->grid solved-string))

(test-case "string round-trip"
  (check-equal? (grid->string solved) solved-string))

(test-case "string->grid accepts dots as empty"
  (define g (string->grid (regexp-replace #rx"0" solved-string ".")))
  (check-equal? (grid->string g) solved-string))

(test-case "string->grid rejects wrong length"
  (check-exn exn:fail? (lambda () (string->grid "123"))))

(test-case "peers: 20 distinct peers per cell, none is the cell itself"
  (for ([i (in-range cell-count)])
    (define ps (peers-of i))
    (check-equal? (length ps) 20)
    (check-false (memv i ps))
    (check-equal? (length (remove-duplicates ps)) 20)))

(test-case "peers of corner cell 0"
  (check-equal? (sort (peers-of 0) <)
                (sort '(1 2 3 4 5 6 7 8 9 10 11 18 19 20 27 36 45 54 63 72) <)))

(test-case "row/col/box of cells"
  (check-equal? (row-of 0) 0)
  (check-equal? (col-of 80) 8)
  (check-equal? (box-of 0) 0)
  (check-equal? (box-of 40) 4)
  (check-equal? (box-of 80) 8))

(test-case "valid? and complete?"
  (check-true (valid? solved))
  (check-true (complete? solved))
  (define broken (grid-copy solved))
  (grid-set! broken 0 3)
  (check-false (valid? broken))
  (define empty (make-grid))
  (check-true (valid? empty))
  (check-false (complete? empty)))

(test-case "pretty string has 13 lines"
  (check-equal? (length (string-split (grid->pretty-string solved) "\n")) 13))
