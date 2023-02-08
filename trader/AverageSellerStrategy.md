# AverageSellerStrategy

## Buying strategy

This strategy always tries to buy the cheapest good available. The problems that arise with
that strategy is:

1. What good do we buy
2. At what max. price to we buy
3. What quantity do we buy
4. How do we stop the trader to buy (spent all the available EUR)

The selection of the good to buy is simple: Just select the good with the lowest owned
quantity. The assumptions are, if the quantity is low, then markets own a lot of that good
and price is cheap.

To solve the second problem, the strategy is allowed to pay at max. 30% of the owned EUR
quantity. 30% because there are 3 different goods to buy.

For the second problem, the strategy tries to find the highest quantity for the max. price.

To stop the trader to spent all EUR, the strategy has a specific threshold of allowed buy
operations. This threshold depends on the sell operations. The difference between a buy
and a sell operations is not allowed to be higher than *n* (e.g. 5). If the trader has
performed *n* more buy operations than sell operations, the trader is not allowed to buy,
and it is expected that the trader sells before buying again.

## Selling strategy

The strategy for selling is simple: Just sell at a higher price than bought. To do that, it
calculates the average price for one piece of the good paid by now and compares that with the 
sell price for one a single piece given by market. If found, sell as much as possible.