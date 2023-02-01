# Trade-Agent

## Possible execution

The agent runs forever (until exited using CTRL+c).

It is possible to start the agent with:

```shell
$ agent --strategy="hold-and-buy" # Run agent on all markets with hold-and-buy strategy
$ agent --strategy="hold-and-buy" --markets=["sgx", "zse"] # Run agent with hold-and-buy strategy only on market SGX and ZSE

# Maybe it makes sense to set a maximum time (days)?
$ agent ... --max-days=30 # Run for max. 30 days

# Maybe it makes sense to set the duration of a day?
$ agent ... --day-in-seconds=60 # One day is 60 seconds long
```
