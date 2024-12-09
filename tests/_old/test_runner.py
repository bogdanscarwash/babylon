import os
import unittest

from babylon.utils.xml_validator import (
    check_entity_imports,
    check_id_references,
    check_unused_ids,
    validate_naming_conventions,
    validate_xml_schema,
)


def find_corresponding_xsd(xml_file_path: str, xsd_dir: str) -> str:
    """Find the matching XSD schema file for a given XML file.

    This function follows the naming convention where each XML file should have a
    corresponding XSD schema file with "_template.xsd" suffix in the schema directory.

    Args:
        xml_file_path: Path to the XML file needing validation
        xsd_dir: Directory containing the XSD schema files

    Returns:
        str: Full path to the matching XSD file if found, None otherwise

    Example:
        For XML file "entity.xml", looks for "entity_template.xsd"
    """
    # Extract the base name without extension (e.g., "entity" from "entity.xml")
    base_name = os.path.splitext(os.path.basename(xml_file_path))[0]

    # Construct expected XSD filename using the template convention
    xsd_file_name = f"{base_name}_template.xsd"
    xsd_file_path = os.path.join(xsd_dir, xsd_file_name)

    # Return the path if file exists, None otherwise
    if os.path.exists(xsd_file_path):
        return xsd_file_path
    else:
        print(f"Schema file '{xsd_file_name}' not found for '{xml_file_path}'")
        return None


class TestXMLValidation(unittest.TestCase):
    """Test suite for validating XML game data files against their schemas.

    This test suite performs comprehensive validation of all XML files in the game,
    checking for proper structure, naming conventions, and reference integrity.
    """

    def setUp(self):
        """Initialize test environment with paths to XML and XSD directories.

        Uses environment variables if set, otherwise falls back to default paths:
        - XML_DIR: Directory containing game data XML files
        - XSD_DIR: Directory containing XML schema definition files
        """
        self.xml_dir = os.getenv("XML_DIR", "xml_files")
        self.xsd_dir = os.getenv("XSD_DIR", "xsd_files")

    def test_xml_files(self):
        """Validate all XML files in the game data directory.

        This test performs a comprehensive validation of each XML file by:
        1. Finding the corresponding XSD schema file
        2. Validating the XML structure against its schema
        3. Checking that all entity imports in schemas exist
        4. Verifying that all ID references are valid
        5. Ensuring naming conventions are followed
        6. Checking for unused ID definitions

        The test uses subtests to handle each XML file independently,
        ensuring that a failure in one file doesn't stop validation of others.

        Raises:
            AssertionError: If any validation check fails for any XML file
        """
        # Recursively walk through all directories under xml_dir
        for root_dir, _, files in os.walk(self.xml_dir):
            # Process only XML files
            for file in files:
                if file.endswith(".xml"):
                    # Construct full path to XML file
                    xml_file_path = os.path.join(root_dir, file)
                    # Find matching schema file
                    xsd_file_path = find_corresponding_xsd(xml_file_path, self.xsd_dir)

                    # Use subTest to handle each file independently
                    with self.subTest(xml_file=xml_file_path):
                        # If schema exists, validate structure and imports
                        if xsd_file_path:
                            self.assertTrue(
                                validate_xml_schema(xml_file_path, xsd_file_path),
                                f"Schema validation failed for {xml_file_path}",
                            )
                            self.assertTrue(
                                check_entity_imports(xsd_file_path, self.xsd_dir),
                                f"Entity import check failed for {xsd_file_path}",
                            )

                        # Perform additional validation checks
                        self.assertTrue(
                            check_id_references(xml_file_path),
                            f"ID reference check failed for {xml_file_path}",
                        )
                        self.assertTrue(
                            validate_naming_conventions(xml_file_path),
                            f"Naming convention check failed for {xml_file_path}",
                        )
                        self.assertTrue(
                            check_unused_ids(xml_file_path),
                            f"Unused ID check failed for {xml_file_path}",
                        )


if __name__ == "__main__":
    unittest.main()