use std ellie
use nu-unit-test *

#[test]
def 'ellie ok' [] {
  return {
    name: 'ellie ok'
    result: ( unit assert equal
"     __  ,
 .--()°'.'
'|, . ,'
 !_-(_\\"
      (ellie | ansi strip)
    ) 
  }
}

#[test]
def "option nopad" [ ] {
  return {
    name: "option nopad"
    result: ( unit assert equal
      (
        "hello"
        | ^cargo run --bin scroller -- --width 10 --duration_ms 50 --interval_ms 50 --nopad --start-delay 0 --switch-delay 0
        | complete
        | get stdout
      ) "     hello
"
    )
  }
}

#[test]
def "dicter default short" [] {
  return {
    name: "dicter default short"
    result: ( unit assert equal
      (
        "hello"
        | ^cargo run --bin scroller -- --width 10 --duration_ms 100 --interval_ms 5 --mode dicter-default --start-delay 0 --switch-delay 0
        | complete
        | get stdout
      ) "▌  hello ▐
"
    )
  }
}

#[test]
def "dicter default long" [] {
  return {
    name: "dicter default long"
    result: ( unit assert equal
      (
        "abcdefghijklmno"
        | ^cargo run --bin scroller -- --width 10 --duration_ms 40 --interval_ms 3 --mode dicter-default --start-delay 0 --switch-delay 0
        | complete
        | get stdout
      ) "▌ abcdef ▐
▌ bcdefg ▐
▌ cdefgh ▐
▌ defghi ▐
▌ efghij ▐
▌ fghijk ▐
▌ ghijkl ▐
▌ hijklm ▐
▌ ijklmn ▐
▌ jklmno ▐
▌ jklmno ▐
▌ ijklmn ▐
▌ hijklm ▐
"
    )
  }
}

#[test]
def "inner bounce short" [] {
  return {
    name: "inner bounce short"
    result: ( unit assert equal
      (
        "hello"
        | ^cargo run --bin scroller -- --width 20 --duration_ms 102 --interval_ms 4 --mode inner-bounce --start-delay 0 --switch-delay 0
        | complete
        | get stdout
      ) "▌ hello            ▐
▌  hello           ▐
▌   hello          ▐
▌    hello         ▐
▌     hello        ▐
▌      hello       ▐
▌       hello      ▐
▌        hello     ▐
▌         hello    ▐
▌          hello   ▐
▌           hello  ▐
▌            hello ▐
▌            hello ▐
▌           hello  ▐
▌          hello   ▐
▌         hello    ▐
▌        hello     ▐
▌       hello      ▐
▌      hello       ▐
▌     hello        ▐
▌    hello         ▐
▌   hello          ▐
▌  hello           ▐
▌ hello            ▐
▌ hello            ▐
"
    )
  }
}

#[test]
def "inner bounce long" [] {
  return {
    name: "inner bounce long"
    result: ( unit assert equal
      (
        "bienvenue"
        | ^cargo run --bin scroller -- --width 8 --duration_ms 160 --interval_ms 10 --mode inner-bounce --start-delay 0 --switch-delay 0
        | complete
        | get stdout
      ) "▌ bien ▐
▌ ienv ▐
▌ enve ▐
▌ nven ▐
▌ venu ▐
▌ enue ▐
▌ enue ▐
▌ venu ▐
▌ nven ▐
▌ enve ▐
▌ ienv ▐
▌ bien ▐
▌ bien ▐
▌ ienv ▐
▌ enve ▐
▌ nven ▐
"
    )
  }
}

#[test]
def "scroll mode marquee" [ ] {

  return {
    name:  "scroll mode marquee"
    result: ( unit assert equal   (
      "hello" 
      | ^cargo run --bin scroller -- --width 10  --duration_ms 85 --interval_ms 5  --mode marquee  --start-delay 0  --switch-delay 0
      | complete
      | get stdout
    ) "▌        ▐
▌      h ▐
▌     he ▐
▌    hel ▐
▌   hell ▐
▌  hello ▐
▌ hello  ▐
▌ ello   ▐
▌ llo    ▐
▌ lo     ▐
▌ o      ▐
▌        ▐
▌      h ▐
▌     he ▐
▌    hel ▐
▌   hell ▐
▌  hello ▐
▌ hello  ▐
"
)
 }
}

let tests = [
  {|| ellie ok }
  {|| scroll mode marquee }
  {|| option nopad }
  {|| dicter default short }
  {|| dicter default long }
  {|| inner bounce short }
  {|| inner bounce long }
]

export def main [  ] {
  let results = test run $tests
  $results | flatten | flatten | flatten | table -e -t light  | print
  $results | flatten | flatten | select name pass | table -e -t light | print
}
