# Język

Pozwoliłem sobie nadać nazwę językowi - **Wirthian**. Wynika to oczywiście z podobieństw składniowych do języków stworzonych przez Niklausa Wirtha.

## Odkryte problemy z gramatyką:

### Unarne operatory mają błędnie zdefiniowaną precedencję

Gramatyka źle definiuje priorytet operatorów unarnych, przez co mają one najniższy priorytet, zamiast najwyższego.

```
f_num_expr = "-" num_expr
```
Powoduje, że sparsowanie minusa będzie dopiero na samym końcu, więc np. `- 2 + 3` zostanie sparsowane jako `- ( 2 + 3 )` zamiast `-2 + 3`.

To samo aplikuje się do operatora `not` w przypadku

```
f_bool_exp = "not" bool_expr
```

### Niejednoznaczność powodująca konflikty reduce/reduce

```
assign_stat = IDENT ":=" num_expr | IDENT ":=" str_expr
output_stat = "print(" num_expr ")" | "print(" str_expr ")"
```

`num_expr` i `str_expr` są w stanie się zmaterializować do `IDENT`, co powoduje, że generator parserów nie jest w stanie wybrać żadnej ze ścieżek tworząc konflikt reduce/reduce.

### Niejednoznazcność powodująca dangling else (shift/reduce)


Mając definicję
```
if_stat = "if" bool_expr "then" simple_instr if_options ;
if_options = "elif" bool_expr "then" simple_instr if_options
           | "else" simple_instr
           | epsilon ;
```

Nie jesteśmy w stanie określić czy wyrażenie
```
if x then
    if y then
        win;
    else
        lose;
```

jest równoważne do:
```
if x then do;
    if y then win;
    else lose;
end;
```
czy do:
```
if x then do;
    if y then win;
    end;
else lose;
```

## Zmiany

Pierwotna gramatyka miałą bardzo ograniczone wsparcie dla wartości logicznych, więc rozszerzyłem ją, żeby każde wyrażenie mogło być traktowane jako wartość logiczna. Dopiero na poziomie checkera sprawdzam czy wyrażenie ma odpowiedni typ dopasowany do operacji (if, operatory, etc.).


## Krok po kroku

Pierwotnie zaimplementowany lexer polegał na typie `String`, który powoduje alokację pamięci na stercie. Zdecydowałem się na przejście do typu `&str`, który operuje na wycinkach tekstowych będących jedynie referencjami do oryginalnego kodu źródłowego. Pozwoliło to całkowicie wyeliminować narzut związany z kopiowaniem danych i dynamiczną alokacją pamięci.

```
lexer/generated_program/program_1_kib
                        time:   [1.7403 µs 1.7543 µs 1.7702 µs]
                        thrpt:  [640.55 MiB/s 646.38 MiB/s 651.55 MiB/s]
                 change:
                        time:   [−45.046% −44.703% −44.386%] (p = 0.00 < 0.05)
                        thrpt:  [+79.812% +80.843% +81.970%]
                        Performance has improved.

lexer/generated_program/program_16_kib
                        time:   [39.804 µs 39.979 µs 40.157 µs]
                        thrpt:  [390.62 MiB/s 392.35 MiB/s 394.08 MiB/s]
                 change:
                        time:   [−23.851% −23.226% −22.665%] (p = 0.00 < 0.05)
                        thrpt:  [+29.307% +30.252% +31.321%]
                        Performance has improved.
                
lexer/generated_program/program_256_kib
                        time:   [680.49 µs 682.94 µs 686.05 µs]
                        thrpt:  [365.00 MiB/s 366.66 MiB/s 367.98 MiB/s]
                  change:
                        time:   [−22.146% −21.789% −21.437%] (p = 0.00 < 0.05)
                        thrpt:  [+27.286% +27.859% +28.446%]
                        Performance has improved.
                        
lexer/generated_program/program_1_mib
                        time:   [2.7383 ms 2.7541 ms 2.7716 ms]
                        thrpt:  [360.80 MiB/s 363.10 MiB/s 365.19 MiB/s]
                 change:
                        time:   [−19.836% −19.308% −18.769%] (p = 0.00 < 0.05)
                        thrpt:  [+23.106% +23.928% +24.745%]
                        Performance has improved.
```
