# StingyStrategy

## The main idea
The Stingy Strategy is based on the fact that the trader is stingy, so it is not happy to spend high quantity of its money and also it is
not happy to give away high quantity of one of its goods.
From this assumption, the trader will always try to buy a small quantity of a good, searching for the one that has the lower exchange rate for EUR.
After the buy operation, the trader will always try to sell a small quantity of a good, searching for the one that has the higher exchange rate to EUR.\
The idea is to get a small gain at every operation.

## Buying strategy
This strategy will try to buy a small amount of a good that has a profitable exchange rate.
To do this, the strategy has to consider the following:
- how to choose the good to buy?
- how to choose the quantity of the good to buy?

The strategy search for a deal considering spending only a little percentage of its EUR as the 1% and collects a set of possible deals.
Afterwards the strategy search for the deal with the lower exchange rate for EUR: lower the exchange rate, more the quantity of the good.\
To perform this research, the strategy compare the exchange rate of every good with the average exchange rate registered in the last 10 operations.
If it can't find a good deal, it will try to buy the one with the lower price, to let the prices fluctuate.

## Selling strategy
The strategy, for selling, follows the same scheme of the buying strategy. First it search for a deal considering spending only a little percentage of its goods 
(USD, YEN or YUAN) and collects a set of possible deals. Afterward the strategy search for the deal with the higher exchange rate to EUR: higher the exchange rate, more the quantity of EUR.
Also the selling strategy look at the average exchange rate during the last 10 operations and if it can't find a good deal, it will try to sell the one with the higher price, to let the prices fluctuate.
The strategy will always try to sell after every buy operation.