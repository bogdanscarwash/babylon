import unittest
from babylon.data.models.event import Event
from babylon.data.models.contradiction import Contradiction, Effect, Entity
from babylon.core.contradiction import ContradictionAnalysis


class MockEntity:
    """Mock entity class for testing."""

    def __init__(self, id: str, type: str) -> None:
        self.id: str = id
        self.type: str = type
        self.freedom: float = 1.0
        self.wealth: float = 1.0
        self.stability: float = 1.0
        self.power: float = 1.0


class MockEntityRegistry:
    """Mock entity registry for testing."""

    def __init__(self) -> None:
        self.entities: dict[str, MockEntity] = {}

    def create_entity(self, type: str, role: str) -> MockEntity:
        """Create and register a new entity."""
        entity = MockEntity(f"{type.lower()}_{role.lower()}", type)
        self.entities[entity.id] = entity
        return entity

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
        upper_class = self.entity_registry.create_entity("Class", "Oppressor")
        working_class = self.entity_registry.create_entity("Class", "Oppressed")

        self.contradiction_analysis = ContradictionAnalysis(self.entity_registry)

    def _create_sample_contradiction(self) -> Contradiction:
        """Create a sample contradiction for testing."""
        # Create entities with just type and role
        upper_class = Entity("Class", "Oppressor")
        working_class = Entity("Class", "Oppressed")
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
                    Effect(target_id=upper_class.id, attribute="wealth", operation="Decrease", magnitude=0.5, description="Test effect")
                ],
                "Revolution": [
                    Effect(target_id=upper_class.id, attribute="wealth", operation="Decrease", magnitude=1.0, description="Test effect")
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

        # Record initial intensity
        initial_intensity = contradiction.intensity
        
        # Update contradictions
        self.contradiction_analysis.update_contradictions(self.game_state)
        
        # Verify intensity history was updated
        self.assertTrue(len(contradiction.intensity_history) > 0)
        # Verify intensity changed
        self.assertNotEqual(contradiction.intensity, initial_intensity)

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

        # Get the entity from the registry using the correct ID
        upper_class = self.entity_registry.get_entity("class_oppressor")
        initial_wealth = upper_class.wealth

        # Apply Reform resolution method effects
        effects = contradiction.resolution_methods["Reform"]
        for effect in effects:
            self.contradiction_analysis._apply_effects([effect], self.game_state)

        self.assertEqual(upper_class.wealth, initial_wealth - 0.5)


if __name__ == "__main__":
    unittest.main()
