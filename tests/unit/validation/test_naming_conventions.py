class TestNamingConventions:
    """Test suite for XML naming convention validation."""
    
    def test_valid_naming_conventions(self, sample_xml_files):
        """Test validation of correct naming patterns."""
        xml_dir, _ = sample_xml_files
        xml_file = xml_dir / "entity.xml"
        
        assert validate_naming_conventions(str(xml_file)), (
            "Valid naming conventions should pass validation"
        )
    
    def test_invalid_naming_patterns(self, tmp_path):
        """Test detection of invalid naming patterns."""
        # Create XML with invalid names
        bad_xml = tmp_path / "bad.xml"
        bad_xml.write_text("""
            <?xml version="1.0" encoding="UTF-8"?>
            <Entity id="Bad_ID">
                <NAME>Invalid Name</NAME>
            </Entity>
        """)
        
        assert not validate_naming_conventions(str(bad_xml)), (
            "Should detect invalid naming patterns"
        )