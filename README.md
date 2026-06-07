# Fair Value Estimation in Binary Option Market

## Black-Scholes Model
Estimate price of call option

$$
C(S, t) = S_0 N(d_1) - K e^{-rT}  N(d_2)
$$

where
* C(S, t): Price of the call option
* S_0: Current price of the underlying asset
* K: Strike price
* T: Time until expiration
* r: Risk-free interest rate
* N(d): CDF of normal distribution of d
* sigma: Volatility of the underlying assets returns
* d_1:

$$
d_1 = \frac{\ln(S_0/K) + (r + \sigma^2/2)T}{\sigma\sqrt(T)}
$$
* d_2:

$$
d_2 = d_1 - \sigma\sqrt{T}
$$


Here we only care if the price at expiry time is above or below strike <br>
The $N(d_2)$ represents risk-neutral probability that the option finishes in-the-money. <br>
Therefore, for the binary option market,

$$
C(S, t) = e^{-rT}N(d_2)
$$


