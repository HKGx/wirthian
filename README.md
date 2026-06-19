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

Następnie zająłem się optymalizacją parsera. LALRPOP pozwala oznaczać produkcje atrybutem `#[inline]`, który wkleja zawartość reguły bezpośrednio w miejscach jej użycia, eliminując niepotrzebne kroki redukcji w tabeli parsera LR. Oznaczyłem w ten sposób cienkie reguły opakowujące — `SimpleInstr`, `BaseInstr`, `AssignStat`, `OutputStat` oraz `CompareExpr` — które były jedynie przekaźnikami do innych produkcji.

Jednocześnie usunąłem zbędne nieterminały `BoolExpr`, `NumExpr` i `StrExpr`, będące aliasami do `Expr`/`OrExpr`/`AddExpr`. Ich separacja powodowała opisane wcześniej konflikty reduce/reduce, a typechecking przeniosłem w całości do checkera.

```
parser/parse/1_kib
                        time:   [13.81 µs 13.99 µs 15.18 µs]
                        thrpt:  [67.83 MB/s 73.57 MB/s 74.53 MB/s]
                 change:
                        time:   [−5.3%]
                        thrpt:  [+5.6%]
                        Performance has improved.

parser/parse/16_kib
                        time:   [265.6 µs 279.4 µs 287.8 µs]
                        thrpt:  [57.63 MB/s 59.36 MB/s 62.46 MB/s]
                 change:
                        time:   [−4.3%]
                        thrpt:  [+4.5%]
                        Performance has improved.

parser/parse/256_kib
                        time:   [5.042 ms 5.242 ms 5.264 ms]
                        thrpt:  [49.79 MB/s 50.00 MB/s 51.99 MB/s]
                 change:
                        time:   [−4.4%]
                        thrpt:  [+4.6%]
                        Performance has improved.

parser/parse/1_mib
                        time:   [20.65 ms 21.81 ms 22.12 ms]
                        thrpt:  [47.38 MB/s 48.06 MB/s 50.77 MB/s]
                 change:
                        time:   [−6.0%]
                        thrpt:  [+6.3%]
                        Performance has improved.
```

Atrybut `#[inline]` eliminuje jedynie wywołanie funkcji akcji. Sam krok redukcji w tabeli LR nadal ma miejsce. Profilowanie przy pomocy `samply` na programach o rozmiarze 1 MiB ujawniło, że głównym wąskim gardłem parsera był narzut związany z przechodzeniem przez kolejne poziomy nieterminałów w gramatyce wyrażeń.

Wyrażenia były zdefiniowane jako łańcuch ośmiu nieterminałów:

```
Expr → OrExpr → AndExpr → NotExpr → CompareExpr → AddExpr → MulExpr → UnaryExpr → PrimaryExpr
```

Każdy atom wyrażenia przechodził przez dziewięć kroków redukcji - jeden dla stworzenia atomu oraz osiem dalszych, będących zwykłymi przekazanami, gdzie każdy z nich zdejmuje z wewnętrznego stosu symbol, wyciąga z niego `Expr`, owija go w nowy symbol i odkłada z powrotem.

LALRPOP posiada wbudowany system poziomów precedencji, który automatycznie generuje warstwy nieterminałów zamiast ręcznego pisania ciągów. Zastąpiłem osiem nieterminałów jedną produkcją `Expr` z siedmioma poziomami precedencji oznaczonymi atrybutami `#[precedence(level="N")]` oraz `#[assoc(side="...")]`:

Ponieważ system precedencji zastępuje rekursywne odwołania do `Expr` wersjami ograniczonymi do odpowiedniego poziomu (`Expr1`, `Expr2`, ...), wyrażenia w nawiasach oraz argumenty funkcji wbudowanych (`length`, `position`, itd.) wymagają nieterminału akceptującego pełne wyrażenia.

Zmiana ta eliminuje dwa kroki na każdy atom wyrażenia (scalenie `Expr→OrExpr` na szczycie oraz `UnaryExpr→PrimaryExpr` na dnie), zachowując istniejąca poprawność precedencji operatorów.

```
parser/parse/1_kib
                        time:   [14.71 µs → 12.21 µs]
                        thrpt:  [69.98 MB/s → 84.28 MB/s]
                 change:
                        time:   [−17.0%]
                        thrpt:  [+20.4%]
                        Performance has improved.

parser/parse/16_kib
                        time:   [272.0 µs → 234.7 µs]
                        thrpt:  [60.97 MB/s → 70.68 MB/s]
                 change:
                        time:   [−13.7%]
                        thrpt:  [+15.9%]
                        Performance has improved.

parser/parse/256_kib
                        time:   [5.168 ms → 4.519 ms]
                        thrpt:  [50.72 MB/s → 58.00 MB/s]
                 change:
                        time:   [−12.6%]
                        thrpt:  [+14.4%]
                        Performance has improved.

parser/parse/1_mib
                        time:   [20.84 ms → 18.30 ms]
                        thrpt:  [50.31 MB/s → 57.29 MB/s]
                 change:
                        time:   [−12.2%]
                        thrpt:  [+13.9%]
                        Performance has improved.
```
