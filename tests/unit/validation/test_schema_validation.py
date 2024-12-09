from tests.unit.validation.test_xml_validator import validate_xml_schema, find_corresponding_xsd

class TestSchemaValidation:
    """Test suite for XML schema validation functionality."""
    
    def test_valid_xml_schema(self, sample_xml_files):
        """Test validation of well-formed XML against its schema."""
        xml_dir, xsd_dir = sample_xml_files
        xml_file = xml_dir / "entity.xml"
        xsd_file = xsd_dir / "entity_template.xsd"
        
        result = validate_xml_schema(str(xml_file), str(xsd_file))
        assert result, "Valid XML should pass schema validation"
    
    def test_schema_not_found(self, sample_xml_files):
        """Test handling of missing schema files."""
        xml_dir, _ = sample_xml_files
        xml_file = xml_dir / "entity.xml"
        
        xsd_path = find_corresponding_xsd(str(xml_file), "nonexistent_dir")
        assert xsd_path is None, "Should handle missing schema gracefully"