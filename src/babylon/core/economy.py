from typing import Any


class Economy:
    """
    Represents the economic system and its various components.
    Handles economic simulation including production, distribution, and exchange.
    """

    def __init__(self) -> None:
        """Initialize the economic system with default values."""
        # Core economic indicators
        self.gdp: float = 1000.0  # Base GDP value
        self.unemployment_rate: float = 5.0  # Initial unemployment rate (%)
        self.inflation_rate: float = 2.0  # Initial inflation rate (%)
        self.gini_coefficient: float = 0.37  # Measure of economic inequality

        # Production and distribution
        self.production_capacity: float = 100.0  # Maximum production capability
        self.resource_utilization: float = 0.75  # Resource usage efficiency
        self.labor_productivity: float = 1.0  # Output per unit of labor

        # Market conditions
        self.market_demand: float = 100.0  # Aggregate demand level
        self.market_supply: float = 100.0  # Aggregate supply level
        self.price_level: float = 1.0  # General price level

        # Class relations
        self.wage_share: float = 0.6  # Labor's share of total income
        self.profit_rate: float = 0.15  # Rate of profit on capital
        self.exploitation_rate: float = 0.4  # Surplus value extraction rate

    def update(self) -> None:
        """
        Update the economic state based on current conditions and policies.
        This method should be called each game tick to simulate economic changes.
        """
        self._update_production()
        self._update_distribution()
        self._update_market_conditions()
        self._update_class_relations()

    def _update_production(self) -> None:
        """Update production-related indicators."""
        # Update production based on labor productivity and resource utilization
        effective_production = (
            self.production_capacity
            * self.resource_utilization
            * self.labor_productivity
        )

        # Adjust GDP based on production changes
        production_change = effective_production - self.gdp
        self.gdp += production_change * 0.1  # Gradual adjustment

    def _update_distribution(self) -> None:
        """Update wealth and income distribution."""
        # Update Gini coefficient based on wage and profit dynamics
        wage_effect = (0.5 - self.wage_share) * 0.1
        self.gini_coefficient = max(0.0, min(1.0, self.gini_coefficient + wage_effect))

    def _update_market_conditions(self) -> None:
        """Update market supply, demand, and prices."""
        # Calculate supply-demand mismatch
        market_imbalance = self.market_demand - self.market_supply

        # Adjust price level based on supply and demand
        self.price_level *= 1 + (market_imbalance * 0.01)

        # Update inflation rate
        self.inflation_rate = (self.price_level - 1) * 100

    def _update_class_relations(self) -> None:
        """Update indicators of class relations and exploitation."""
        # Update exploitation rate based on profit and wage dynamics
        self.exploitation_rate = self.profit_rate / self.wage_share

        # Adjust unemployment based on production needs
        labor_demand = self.gdp / self.labor_productivity
        self.unemployment_rate = max(0.0, min(100.0, 100 * (1 - labor_demand / 100)))

    def calculate_production(self) -> float:
        """
        Calculate total economic production.

        Returns:
            float: Current GDP value
        """
        return self.gdp

    def adjust_prices(self, inflation_target: float = 2.0) -> None:
        """
        Adjust prices based on supply, demand, and inflation target.

        Args:
            inflation_target: Target inflation rate in percent
        """
        inflation_gap = self.inflation_rate - inflation_target
        self.price_level *= 1 - (inflation_gap * 0.01)

    def simulate_market(self) -> None:
        """Simulate market interactions between economic actors."""
        # Update market demand based on economic conditions
        demand_change = (
            1 - self.unemployment_rate / 100
        ) * 0.1 + (  # Employment effect
            1 - self.inflation_rate / 10
        ) * 0.05  # Price stability effect
        self.market_demand *= 1 + demand_change

        # Update market supply based on production capacity
        supply_change = (
            self.resource_utilization * 0.1
            + (self.profit_rate - 0.1) * 0.05  # Profit incentive effect
        )
        self.market_supply *= 1 + supply_change

    def apply_policy(self, policy: dict[str, Any]) -> None:
        """
        Apply an economic policy and calculate its effects.

        Args:
            policy: Dictionary containing policy parameters and values
        """
        if "wage_adjustment" in policy:
            self.wage_share = max(
                0.0, min(1.0, self.wage_share + policy["wage_adjustment"])
            )

        if "production_boost" in policy:
            self.production_capacity *= 1 + policy["production_boost"]

        if "resource_efficiency" in policy:
            self.resource_utilization = max(
                0.0, min(1.0, self.resource_utilization + policy["resource_efficiency"])
            )

    def get_economic_indicators(self) -> dict[str, float]:
        """
        Get current economic indicators.

        Returns:
            dict[str, float]: Dictionary of current economic indicators
        """
        return {
            "gdp": self.gdp,
            "unemployment": self.unemployment_rate,
            "inflation": self.inflation_rate,
            "gini": self.gini_coefficient,
            "wage_share": self.wage_share,
            "profit_rate": self.profit_rate,
            "exploitation_rate": self.exploitation_rate,
        }
