import unittest
from datetime import datetime
from babylon.data.entity_registry import EntityRegistry
from babylon.systems.contradiction_analysis import ContradictionAnalysis
from babylon.data.models.contradiction import Contradiction, Entity
from babylon.metrics.collector import MetricsCollector

class TestMetricsCollection(unittest.TestCase):
    def setUp(self):
        self.entity_registry = EntityRegistry()
        self.contradiction_analysis = ContradictionAnalysis(self.entity_registry)
        
    def test_entity_access_metrics(self):
        # Create and register a test entity
        test_entity = Entity("test_id", "test_type", "test_role")
        self.entity_registry.register_entity(test_entity)
        
        # Access the entity multiple times
        for _ in range(5):
            self.entity_registry.get_entity("test_id")
            
        # Verify metrics
        metrics = self.entity_registry.metrics.analyze_performance()
        self.assertEqual(metrics['hot_objects'][0], "test_id")
        
    def test_contradiction_metrics(self):
        # Create test contradiction
        entities = [Entity("entity1", "type1", "role1")]
        contradiction = Contradiction(
            id="test_contra",
            name="Test Contradiction",
            description="Test description",
            entities=entities,
            universality="Universal",
            particularity="Economic",
            principal_contradiction=None,
            principal_aspect=entities[0],
            secondary_aspect=entities[0],
            antagonism="Primary",
            intensity="High",
            state="Active",
            potential_for_transformation="High",
            conditions_for_transformation=["Condition1"],
            resolution_methods={"Method1": []}
        )
        
        # Add contradiction and verify metrics
        self.contradiction_analysis.add_contradiction(contradiction)
        
        metrics = self.contradiction_analysis.metrics.analyze_performance()
        self.assertIn('test_contra', [obj for obj in metrics['hot_objects']])
        self.assertTrue(metrics['latency_stats']['context_switches'])

if __name__ == '__main__':
    unittest.main()
