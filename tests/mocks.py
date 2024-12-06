class MockMetricsCollector:
    """Mock metrics collector for testing."""
    
    def __init__(self):
        self.access_records = {}
        
    def record_object_access(self, object_id: str, context: str) -> None:
        """Record an object access."""
        if object_id not in self.access_records:
            self.access_records[object_id] = 0
        self.access_records[object_id] += 1
