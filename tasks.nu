
#[task]
def "clean" [
  --info
  ] {
  let cleanCode =  {||
    rm -rf target
    return "done cleaning" }
  return {
    name: "clean"
    description: "delete old builds"
    closure: $cleanCode 
    result: ( if not $info { do  $cleanCode }  )
  }
}

#[task]
def "run dicter" [
  --info 
   ] {
  let closure = {||
    ( ^cargo run --bin dicter --
      --interval 1
      --dict freedict-fra-eng
      --duration_s 10 )
    return "finished running dicter"
  }
  return {
    name: "run dicter"
    description: "show some random definitions"
    closure: $closure 
    result: ( if not $info { do $closure } )
  }
}

#[task]
def "debug dicter" [
  --info
  ] {
  let closure = {||
    $env.DEBUG = 1 
    ( ^cargo run --bin dicter --
      --interval 1
      --dict freedict-fra-eng
      --duration_s 10 )
    return "finished debuging dicter"
  }
  return {
    name: "debug dicter"
    description: "show some random definitions with debug output"
    closure: $closure
    result: ( if not $info { do $closure } )
  }
}

#[task]
def "build release" [
  --info
  ] {
  let closure = {||
    ^cargo build --release --workspace
    return "finished building release"
  }
  return {
    name: "build release"
    description: "build release binaries"
    closure: $closure
    result: ( if not $info { do $closure } )
  }
}

#[task]
def "run scroller" [
  --info
  ] {
  let closure = {||
    ( ^cargo run --bin dicter --
      # --pronounce true
      --interval 1
      --duration_s 20 )      
    | ( ^cargo run --bin scroller --
      --width 20
      --interval_ms 50
      --duration_ms 2000 )
    return "finished running scroller"
  }
  return {
    name: "run scroller"
    description: "run dicter piped through scroller"
    closure: $closure
    result: ( if not $info { do $closure } )
  }
}

#[task]
def "run slowcat" [
  --info
  ] {
  let closure = {||
    "the quick brown fox jumped over the lazy dog"
    | ^cargo run --bin slowcat -- --linelen 10 --interval 2 --no-repeat
    return "finished running slowcat"
  }
  return {
    name: "run slowcat"
    description: "print text slowly through slowcat"
    closure: $closure
    result: ( if not $info { do $closure } )
  }
}

#[task]
def "run slowcat with scroller" [
  --info
  ] {
  let closure = {||
    "the quick brown fox jumped over the lazy dog"
    | ^cargo run --bin slowcat -- --interval 4 --repeat true
    | ^cargo run --bin scroller -- --width 14 --interval_ms 50 --duration_ms 2700 --delay 0 --mode marquee
    return "finished running slowcat with scroller"
  }
  return {
    name: "run slowcat with scroller"
    description: "run slowcat piped through scroller"
    closure: $closure
    result: ( if not $info { do $closure } )
  }
}

#[task]
def "run slowcat with reader" [
  --info
  ] {
  let closure = {||
    open ./README.md | to text
    | ^cargo run --bin slowcat -- --interval 1
    | ^cargo run --bin scroller -- --width 20 --interval_ms 40 --y 0 --mode reader
    return "finished running slowcat with reader"
  }
  return {
    name: "run slowcat with reader"
    description: "run slowcat piped through scroller in reader mode"
    closure: $closure
    result: ( if not $info { do $closure } )
  }
}

#[task]
def "run scroll mode tests" [
  --info
  ] {
  let closure = {||
    ^nu ./scroll_mode_tester.nu
    return "finished running scroll mode tests"
  }
  return {
    name: "run scroll mode tests"
    description: "A-B tests for scroll modes"
    closure: $closure
    result: ( if not $info { do $closure } )
  }
}


#[task]
def "install" [
  --info
  ] {
  let closure = {||
    try { build release }
    try { ^pkill dicter | complete  }
    try { ^pkill randphrase  | complete }
    try { ^pkill waybar | complete  }
    try { ^pkill scroller | complete  }
    try { ^pkill slowcat | complete  }
    try { cp target/release/dicter ~/.local/bin }
    try { cp target/release/randphrase ~/.local/bin }
    try { cp target/release/scroller ~/.local/bin }
    try { cp target/release/slowcat ~/.local/bin }
    return "finished installing"
  }
  return {
    name: "install"
    description: "build release and copy to ~/.local/bin"
    closure: $closure
    result: ( if not $info { do $closure } )
  }
}

#[task]
def "time ticker" [
  --info
  ] {
  let closure = {||
      try { install }
      print (ansi -e "?25l") # hide cursor
      clear
      for x in 67..76  {
        ( date now | to text | str substring 0..-16
        | scroller
          --width 40
          --interval_ms 100
          --duration_ms 6200
          --delay 0
          --x 5
          --y 5
          --nopad
          --mode marquee )
      }
    return "finished time ticker"
  }
  return {
    name: "time ticker"
    description: "display scrolling clock"
    closure: $closure
    result: ( if not $info { do $closure } )
  }
}

let meta_tasks = [
  ( clean --info )
  ( run dicter --info )
  ( debug dicter --info )
  ( build release --info )
  ( run scroller --info )
  ( run slowcat --info )
  ( run slowcat with scroller --info )
  ( run slowcat with reader --info )
  ( run scroll mode tests --info )
  ( install --info )
  ( time ticker --info )
]

export def main [
  --task : string
 ] {
  if  ( $task | is-not-empty )  {
    let t = $meta_tasks | where $in.name == $task
    let result = ( do $t.0.closure )
    print $result
    return 
  }
  print "pick task to run"
  print "================"
  let t = ( $meta_tasks  | input list -d
    {|r| $"(ansi blue)($r.name)(ansi reset), ($r.description)"}
  )
  let result = ( do $t.closure )
  print $result
  return 
}
