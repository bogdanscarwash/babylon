import os
import unittest
from babylon.utils.xml_validator import (
    validate_xml_schema,
    check_id_references,
    check_entity_imports,
    validate_naming_conventions,
    check_unused_ids
)

def find_corresponding_xsd(xml_file_path, xsd_dir):
    base_name = os.path.splitext(os.path.basename(xml_file_path))[0]
    xsd_file_name = f"{base_name}_template.xsd"
    xsd_file_path = os.path.join(xsd_dir, xsd_file_name)
    if os.path.exists(xsd_file_path):
        return xsd_file_path
    else:
        print(f"Schema file '{xsd_file_name}' not found for '{xml_file_path}'")
        return None

class TestXMLValidation(unittest.TestCase):
    def setUp(self):
        self.xml_dir = os.getenv('XML_DIR', 'xml_files')
        self.xsd_dir = os.getenv('XSD_DIR', 'xsd_files')

    def test_xml_files(self):
        for root_dir, _, files in os.walk(self.xml_dir):
            for file in files:
                if file.endswith('.xml'):
                    xml_file_path = os.path.join(root_dir, file)
                    xsd_file_path = find_corresponding_xsd(xml_file_path, self.xsd_dir)

                    with self.subTest(xml_file=xml_file_path):
                        if xsd_file_path:
                            self.assertTrue(validate_xml_schema(xml_file_path, xsd_file_path))
                            self.assertTrue(check_entity_imports(xsd_file_path, self.xsd_dir))
                        
                        self.assertTrue(check_id_references(xml_file_path))
                        self.assertTrue(validate_naming_conventions(xml_file_path))
                        self.assertTrue(check_unused_ids(xml_file_path))

if __name__ == '__main__':
    unittest.main()
