# Replance - REPL for nc

Only supports the simplest use of netcat - opening a connection and writing
to/reading from a socket - but adds line editing that isn't incredibly
frustrating.

## Usage

```sh
$ rnc <HOST> <PORT>
```

That's pretty much it. The prompt should support most line editing offered by
readline.
