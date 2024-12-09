import pytest
import os
from typing import Dict, Tuple
from pathlib import Path

@pytest.fixture
def test_directories() -> Dict[str, str]:
    """Provide configured test directories for XML validation.
    
    This fixture sets up the test environment with proper paths to XML and
    schema files, allowing for consistent testing across different environments.
    """
    return {
        "xml_dir": os.getenv("XML_DIR", "xml_files"),
        "xsd_dir": os.getenv("XSD_DIR", "xsd_files")
    }

@pytest.fixture
def sample_xml_files(tmp_path: Path) -> Tuple[Path, Path]:
    """Create sample XML files for testing.
    
    This fixture creates a set of test XML files with known content,
    allowing for predictable validation scenarios.
    """
    xml_dir = tmp_path / "xml"
    xsd_dir = tmp_path / "xsd"
    xml_dir.mkdir()
    xsd_dir.mkdir()
    
    # Create a sample XML file
    test_xml = xml_dir / "entity.xml"
    test_xml.write_text("""
        <?xml version="1.0" encoding="UTF-8"?>
        <entity id="test_entity">
            <name>Test Entity</name>
            <references>
                <ref id="other_entity"/>
            </references>
        </entity>
    """)
    
    # Create corresponding schema
    test_xsd = xsd_dir / "entity_template.xsd"
    test_xsd.write_text("""
        <?xml version="1.0" encoding="UTF-8"?>
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
            <xs:element name="entity">
                <xs:complexType>
                    <xs:sequence>
                        <xs:element name="name" type="xs:string"/>
                        <xs:element name="references"/>
                    </xs:sequence>
                    <xs:attribute name="id" type="xs:string"/>
                </xs:complexType>
            </xs:element>
        </xs:schema>
    """)
    
    return xml_dir, xsd_dir