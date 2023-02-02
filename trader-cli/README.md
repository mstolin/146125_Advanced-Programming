# Trader-CLI

This is a CLI tool to execute a trader using the *trader* library.

## Installation

From the workspace directory execute the following:

```shell
$ cargo install --path ./trader-cli
```

After that, you can use the command as `$ trader-cli`.

## Usage

To see its features, execute `$ trader-cli --help`, it will print out the following:

```text
Usage: trader-cli [OPTIONS] <STRATEGY> [MARKETS]...

Arguments:
  <STRATEGY>    Name of the strategy the trader is supposed to use. Available strategy names: mostsimple
  [MARKETS]...  List of markets the trader should work with. Available market names: sgx, smse, tase, zse

Options:
  -c, --capital <CAPITAL>
          The starting capital in EUR for the trader [default: 1000000]
  -d, --days <DAYS>
          The number of days this trader is suppose to run [default: 1]
  -l, --log-level <LOG_LEVEL>
          The log level of the application [default: error]
  -m, --minute-interval <MINUTE_INTERVAL>
          The interval of minutes, when the trader applies its strategy during the day [default: 60]
  -a, --as-json
          Indicates if the history should be printed as JSON. Otherwise, it will be printed as plain text
  -p, --print-history
          Print the history after a successful run
  -h, --help
          Print help
  -V, --version
          Print version
```

## Examples

*Run `MostSimple` for 30 days, every 60 minutes on SGX and TASE and print history as JSON*
```shell
$ trader-cli mostsimple sgx tase -d 30 -m 60 --as-json
```

*Run `MostSimple` for 7 days, every 10 minutes on SGX, SMSE, and TASE wth 30.000.00 EUR start capital, and print history
as plain text*
```shell
$ trader-cli mostsimple sgx smse tase -d 7 -m 10 -c 3000000
```
