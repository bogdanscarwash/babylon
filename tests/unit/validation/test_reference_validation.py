class TestReferenceValidation:
    """Test suite for XML reference validation functionality."""
    
    def test_valid_id_references(self, sample_xml_files):
        """Test validation of correct ID references."""
        xml_dir, _ = sample_xml_files
        xml_file = xml_dir / "entity.xml"
        
        assert check_id_references(str(xml_file)), (
            "Valid ID references should pass validation"
        )
    
    def test_unused_id_detection(self, sample_xml_files):
        """Test detection of unused ID definitions."""
        xml_dir, _ = sample_xml_files
        xml_file = xml_dir / "entity.xml"
        
        assert check_unused_ids(str(xml_file)), (
            "Should detect unused ID definitions"
        )