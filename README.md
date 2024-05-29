# Escrow - Escrow pools for the Radix DLT

The Escrow component allows you to create account-like escrow pools
that you can put funds into—or have others put funds into—and them
retrieve them later.

You can create Allowances that allow others to pull funds from your
escrow pools.

You can use XRD in your escrow pool to help pay transaction costs, or
through use of Allowances, allow others to use your escrow pool to
fund their transactions.

An Escrow as implemented here is an example of a warehouse component
as discussed in [this
video](https://www.youtube.com/watch?v=naMAz9o9d2M).

## How to build the blueprint
Make sure you have the necessary toolchain installed, see
[here](https://docs.radixdlt.com/docs/getting-rust-scrypto)
for details. You will need Scrypto 1.1.1.
- From the command line, in the `escrow` directory, run `scrypto build`

Note that this project compiles with a number of compiler warnings.
Primarily this seems to be because I use modules with common functions
in them that are used by some test case compilations and not by
others, and the ones that don't use them complain about it.

### How to run the test suite
- From the command line, in the `escrow` directory, run `scrypto test`

### How to generate the documentation
- From the command line, in the `escrow` directory, run `cargo doc`

The generated web pages contain detailed documentation on how the
blueprint works.

## Scenarios

A few specific scenarios of how an Escrow can be used are available in
the test suite. These showcase the power of using an Escrow to fuel
your on-ledger activities. It is recommended that you examine the
source file for each scenario for a full description of what the
scenario is all about and how it's run.

For each scenario you can run it in the same way you run tests, like
this: `scrypto test scenario_1` (to run scenario 1). If you want to
generate an output report from the scenario run you can add a filename
in environment variable called e.g. `SCENARIO1_LOG_FILE` (for scenario
1). On Linux doing so simply looks like this:
`SCENARIO1_LOG_FILE=report.csv scrypto test scenario_1`.

### Scenario 1 - distributing 1 billion MEME to the world

This scenario uses the Escrow component to sell 1 billion MEME tokens
on four different exchanges without having to juggle tokens between
those exchanges as one runs out while another has hardly seen any
trade etc. This is achieved by having each exchange take the MEME it
sells direct from the Escrow at time of sale, instead of each exchange
having its own internal store of MEME tokens to sell.

Note that we can use Allowances with infinite amounts since we want to
give access to the full Escrow, and it will then all be limited by how
many tokens are actually available.

If we were to do the same with, let's say, 100G tokens then the
figures below show (first) the traditional way of distributing those
100G across four different size DEXes, and then the more
straightforward way to do it with an Escrow.

![Traditional distribution method](img/Traditional%20Distribution%20of%20MEME%20Tokens%20on%204%20DEXes.png)

![Escrow-based distribution method](img/Escrow%20Distribution%20of%20MEME%20Tokens%20on%204%20DEXes.png)

### Scenario 2 - buying from multiple DEXes

In this scenario, the Escrow is filled with 1000 XRD and this is used
to back limit buys on four DEXes, ensuring that you get all the trades
that fit your buy strategy on all the DEXes until your full amount
runs out.

Without the Escrow, you would instead need to distribute your 1000 XRD
across the four DEXes by some formula, and keep monitoring the
situation to try and transfer funds (in time!) between the DEXes if
there is a sudden spike in trade on one of them.
