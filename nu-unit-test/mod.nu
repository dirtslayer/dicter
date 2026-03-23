#!/usr/bin/env nu
#
# nu-unit-test.nu
# emulate a junit test fixture in nushell
#




#
# unit assert, test a condition and report the result
#
export def 'unit assert' [
    condition: bool, # Condition, which should be true
    --error-label: record # Label for custom assert
    --success-label: record
] {
    if $condition {
     return {
        label: ($success_label | default {
            pass: true
        })
    }
    }

    return {
        label: ($error_label | default {
            pass: false
        })
    }
}

#
# unit assert $left == $right
#
export def 'unit assert equal' [left: any, right: any ] {
    unit assert ($left == $right) --error-label {
        left: $left,
        right: $right,
        pass: false
    } --success-label {
        left: $left,
        right: $right,
        pass: true
    }
}


#### some tests

#[test]
def 'test ok' [] {
    return {
        name: 'test ok'
        result: ( unit assert (0 == 0)  )
    }
}

#[test]
def 'test not ok' [] {
    return {
        name: 'test not ok'
        result: ( unit assert (1 == 0)  )
    }
}

#[test]
def 'test ok equal' [] {
    let input = 'value'
    let expected = 'value'
    return {
        name: 'test ok equal'
        result:( unit assert equal $input $expected)
    }
}

#[test]
def 'test not ok equal' [] {
    let input = 'value'
    let expected = 'wrong'
    return {
        name: 'test not ok equal'
        result: ( unit assert equal $input $expected  )
    }
}

# CALLBACK
# runs once at the start of test run
def 'test before all' [] {
    # 'insert set up before all' | print
    ()
}

# CALLBACK
# gets called every test
def 'test before each' [] {
    # 'insert set up before each' | print
    ()
}

# CALLBACK
# tear down code after each test
def 'test after each' [] {
    # 'insert tear down after each' | print
    ()
}

# CALLBACK
# tear down code after last test
def 'test after all' [] {
    # 'insert tear down after all' | print
    ()
}

# todo: with env?
# 
export def 'test run' [tests:list<any>] {  #-> list<any>
    test before all
    let resultone = $tests | wrap t | upsert r {|test|
        test before each
        let result = ( do $test.t ) 
        test after each
        $result
     }
    test after all
    # print $resultone
    $resultone | get r
}


def 'show selftest results' [] {
    let selftests = [
        {|| test ok}
        {|| test not ok}
        {|| test ok equal}
        {|| test not ok equal}
    ]
    let results = test run $selftests
    $results | flatten | flatten | flatten | table -e -t light  | print
    $results | flatten | flatten | select name pass | table -e -t light | print
}

export def main [] {
'
====== this is a module but we can test it with itself by executing main  =========' | print

    show selftest results
}
