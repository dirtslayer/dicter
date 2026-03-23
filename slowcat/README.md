## slowcat

`slowcat` takes input from stdin, strips all newlines replacing with space, it will accumulate 80 chars (parameter `linelen`) then output them. After it outputs the 80 chars it will wait for an interval (parameter `interval`, in seconds; default 10) before consuming the next chars (at least 80). If we hit the EOF, we will optionally repeat (param `repeat`, default true).

