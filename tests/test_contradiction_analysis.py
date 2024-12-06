import unittest

from src.babylon.data.models.contradiction import Contradiction, Effect, Entity
from src.babylon.data.models.event import Event
from src.babylon.systems.contradiction_analysis import ContradictionAnalysis


class MockEntity:
    """Mock entity class for testing."""

    def __init__(self, entity_id: str, entity_type: str) -> None:
        self.entity_id: str = entity_id
        self.entity_type: str = entity_type
        self.freedom: float = 1.0
        self.wealth: float = 1.0
        self.stability: float = 1.0
        self.power: float = 1.0


class MockEntityRegistry:
    """Mock entity registry for testing."""

    def __init__(self) -> None:
        self.entities: dict[str, MockEntity] = {}

    def register_entity(self, entity: MockEntity) -> None:
        self.entities[entity.entity_id] = entity

    def get_entity(self, entity_id: str) -> MockEntity | None:
        return self.entities.get(entity_id)


class TestContradictionAnalysis(unittest.TestCase):
    """Test cases for the ContradictionAnalysis system."""

    def setUp(self) -> None:
        """Set up test fixtures."""
        self.entity_registry: MockEntityRegistry = MockEntityRegistry()
        self.game_state = {
            "economy": type(
                "Economy", (), {"gini_coefficient": 0.4, "unemployment_rate": 0.1}
            )(),
            "politics": type("Politics", (), {"stability_index": 0.5})(),
            "entity_registry": self.entity_registry,
            "event_queue": [],
            "is_player_responsible": False,
        }

        # Add mock entities
        upper_class = MockEntity("upper_class", "Class")
        working_class = MockEntity("working_class", "Class")
        self.entity_registry.register_entity(upper_class)
        self.entity_registry.register_entity(working_class)

        self.contradiction_analysis = ContradictionAnalysis(self.entity_registry)

    def _create_sample_contradiction(self) -> Contradiction:
        """Create a sample contradiction for testing."""
        upper_class: Entity = Entity("upper_class", "Class", "Oppressor")
        working_class: Entity = Entity("working_class", "Class", "Oppressed")
        entities: list[Entity] = [upper_class, working_class]

        return Contradiction(
            id="economic_inequality",
            name="Economic Inequality",
            description="Testing contradiction.",
            entities=entities,
            universality="Universal",
            particularity="Economic",
            principal_contradiction=None,
            principal_aspect=upper_class,
            secondary_aspect=working_class,
            antagonism="Antagonistic",
            intensity="Low",
            state="Active",
            potential_for_transformation="High",
            conditions_for_transformation=["Revolutionary Movement"],
            resolution_methods={
                "Reform": [
                    Effect("upper_class", "wealth", "Decrease", 0.5, "Test effect")
                ],
                "Revolution": [
                    Effect("upper_class", "wealth", "Decrease", 1.0, "Test effect")
                ],
            },
            attributes={},
        )

    def test_add_contradiction(self) -> None:
        """Test adding a contradiction to the system."""
        contradiction = self._create_sample_contradiction()
        self.contradiction_analysis.add_contradiction(contradiction)
        self.assertIn(contradiction, self.contradiction_analysis.contradictions)

    def test_detect_new_contradictions(self) -> None:
        """Test detection of new contradictions."""
        self.game_state["economy"].gini_coefficient = 0.6
        new_contradictions: list[Contradiction] = (
            self.contradiction_analysis.detect_new_contradictions(self.game_state)
        )
        self.assertTrue(len(new_contradictions) > 0)
        self.assertEqual(new_contradictions[0].id, "economic_inequality")

    def test_update_contradictions(self) -> None:
        """Test updating contradiction states."""
        contradiction: Contradiction = self._create_sample_contradiction()
        self.contradiction_analysis.add_contradiction(contradiction)
        self.game_state["economy"].gini_coefficient = 0.5

        self.contradiction_analysis.update_contradictions(self.game_state)
        self.assertTrue(len(contradiction.intensity_history) > 0)

    def test_generate_events(self) -> None:
        """Test event generation from contradictions."""
        contradiction: Contradiction = self._create_sample_contradiction()
        contradiction.state = "Active"
        contradiction.intensity = "High"
        self.contradiction_analysis.contradictions.append(contradiction)

        events: list[Event] = self.contradiction_analysis.generate_events(
            self.game_state
        )
        self.assertTrue(len(events) > 0)
        self.assertIsInstance(events[0], Event)

    def test_effect_application(self) -> None:
        """Test applying effects to entities."""
        contradiction = self._create_sample_contradiction()
        self.contradiction_analysis.add_contradiction(contradiction)

        upper_class = self.entity_registry.get_entity("upper_class")
        initial_wealth = upper_class.wealth

        # Apply Reform resolution method effects
        effects = contradiction.resolution_methods["Reform"]
        for effect in effects:
            self.contradiction_analysis._apply_effects([effect], self.game_state)

        self.assertEqual(upper_class.wealth, initial_wealth - 0.5)


if __name__ == "__main__":
    unittest.main()
