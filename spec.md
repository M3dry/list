# types
- string
- char
- bool
## integers
## signed
- i8
- i16
- i32
- i64
## unsigned
- u8
- u16
- u32
- u64

# if
## syntax
    ```
    (if (case)
        (true branch)
        (false branch))
    ```
## parser
    IF => OpenParen 'if' EXP EXP EXP CloseParen
## examples
    ```
    (if (> 5 1)
        (- 5 1) -- this case will be executed
        (+ 5 1))
    ```

# match
    - patter matching
## syntax
    ```
    (match (value)
        ((case1) (case1do))
        ((case2) (case2do))
        ((case4 if (case4conditional)) (case4do))
        ((_) (case3do)))
    ```
## parser
    MATCH => OpenParen 'match' EXP {BRANCH} CloseParen
    BRANCH => OpenParen PATTERN EXP CloseParen
    PATTERN => OpenParen? (_ | VALUE) CloseParen? | OpenParen VALUE 'if' EXP CloseParen
## examples
    ```
    (match (1 2 3)
        ((arr if (= (len arr) 3))
            (+ arr 4))
        (arr
            (+ arr (+ (last arr) 1))))
    ```

# function definition
## syntax
    ```
    (define (scope) (name) (args)->(return type)(args)
        (body))
    ```
## parser
    FUNCTION_DEF => OpenParen 'our'? 'define' NAME HEADER EXP CloseParen
    HEADER => ARG_TYPES '->' TYPE OpenParen ARGS CloseParen
    ARG_TYPES => TYPE | OpenParen {TYPE}+ CloseParen
## examples
    ```
    (define multiply (x->i32 y->i32)->i32
        (* x y))
    (pub define multiply (x->i32 y->i32)->i32
        (* x y))
    ```

# lambda
## syntax
    ```
    (lambda (args) (body))
    ```
## parser
    LAMBDA => OpenParen 'lambda' ARGS EXP CloseParen
## examples
    ```
    ((lambda (x y) (* x y)) 10 20)
    ```

# parser
    file => {FUNCTION_DEF}? EXP
    EXP => IF | MATCH | LAMBDA | LITERAL
    LITERAL => int_literal | string_literal | char_literal | 'true' | 'false'
    NAME => [a-zA-Z0-9!@#$%^&*-_=+,.<>?]*
    ARGS => OpenParen {NAME}+ CloseParen
    TYPE => INTEGER | WORDISH | 'bool' | COMPLEX_TYPE | FUNCTION_HEADER_TYPE
    WORDISH => 'string' | 'char'
    COMPLEX_TYPE => OpenParen complex_type_name (TYPE | COMPLEX_TYPE) CloseParen
    FUNCTION_HEADER_TYPE => OpenParen ARG_TYPES '->' TYPE CloseParen
    INTEGER => 'i8' | 'i16' | 'i32' | 'i64' | 'u8' | 'u16' | 'u32' | 'u64'
## example
    ```
    ((lambda (x y) (= x y))
        (if (= 5 4) 4 5)
        (match 4
            (3 -5)
            (n if (= n 4) (+ n 20))
            (_ 10)))
    ```
# struct
## syntax
    ```
    (struct name { field type })
    ```
## example
    ```
    (struct Person { name String age i32 })
    ```
