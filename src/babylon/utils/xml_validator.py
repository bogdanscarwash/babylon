import os
import logging
import xmlschema
import xml.etree.ElementTree as ET

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def validate_xml_schema(xml_file_path, xsd_file_path):
    schema = xmlschema.XMLSchema(xsd_file_path)
    try:
        schema.validate(xml_file_path)
        return True
    except xmlschema.validators.exceptions.XMLSchemaValidationError as e:
        logger.error(f"Validation error in '{xml_file_path}': {e}")
        return False

def check_id_references(xml_file_path):
    tree = ET.parse(xml_file_path)
    root = tree.getroot()

    ids = set()
    idrefs = set()

    for elem in root.iter():
        id_attr = elem.attrib.get('id')
        if id_attr:
            ids.add(id_attr)

        ref_attr = elem.attrib.get('ref')
        if ref_attr:
            idrefs.add(ref_attr)

    missing_refs = idrefs - ids
    if missing_refs:
        logger.error(f"Missing ID references in '{xml_file_path}': {missing_refs}")
        return False
    return True

def check_entity_imports(xsd_file_path, base_dir):
    tree = ET.parse(xsd_file_path)
    root = tree.getroot()
    xs_namespace = 'http://www.w3.org/2001/XMLSchema'
    ns = {'xs': xs_namespace}

    for elem in root.findall('xs:include', ns) + root.findall('xs:import', ns):
        schema_location = elem.attrib.get('schemaLocation')
        if schema_location:
            included_schema = os.path.join(base_dir, schema_location)
            if not os.path.exists(included_schema):
                logger.error(f"Error: Included schema '{schema_location}' not found in '{xsd_file_path}'")
                return False
    return True

def validate_naming_conventions(xml_file_path):
    tree = ET.parse(xml_file_path)
    root = tree.getroot()
    naming_issue = False

    for elem in root.iter():
        tag = elem.tag
        if not tag.islower():
            logger.error(f"Naming convention error in '{xml_file_path}': Element '{tag}' should be lowercase")
            naming_issue = True
        for attr in elem.attrib:
            if not attr.islower():
                logger.error(f"Naming convention error in '{xml_file_path}': Attribute '{attr}' in element '{tag}' should be lowercase")
                naming_issue = True
    return not naming_issue

def check_unused_ids(xml_file_path):
    tree = ET.parse(xml_file_path)
    root = tree.getroot()

    ids = set()
    idrefs = set()

    for elem in root.iter():
        id_attr = elem.attrib.get('id')
        if id_attr:
            ids.add(id_attr)

        ref_attr = elem.attrib.get('ref')
        if ref_attr:
            idrefs.add(ref_attr)

    unused_ids = ids - idrefs
    if unused_ids:
        logger.warning(f"Unused IDs in '{xml_file_path}': {unused_ids}")
        return False
    return True
