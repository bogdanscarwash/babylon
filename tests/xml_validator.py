"""XML Validation Module

This module provides comprehensive XML validation functionality, including:
- Schema validation against XSD files
- ID reference integrity checking
- Schema import/include verification
- XML naming convention enforcement

The module uses xmlschema for XSD validation and ElementTree for XML parsing.
It implements a thorough validation pipeline that can be used for both
development-time validation and runtime checks.
"""

import os
import logging
import xmlschema
import xml.etree.ElementTree as ET

# Configure module-level logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def validate_xml_schema(xml_file_path: str, xsd_file_path: str) -> bool:
    """Validate an XML file against its XSD schema definition.
    
    Performs full schema validation including:
    - Data type validation 
    - Element structure validation
    - Attribute validation
    - Complex type validation
    
    The validation is strict - any schema violation will cause a failure.

    Args:
        xml_file_path: Path to the XML file to validate
        xsd_file_path: Path to the XSD schema file

    Returns:
        bool: True if validation succeeds, False if validation fails

    Raises:
        XMLSchemaValidationError: If the XML fails to validate against the schema
    """
    schema = xmlschema.XMLSchema(xsd_file_path)
    try:
        schema.validate(xml_file_path)
        return True
    except xmlschema.validators.exceptions.XMLSchemaValidationError as e:
        logger.error(f"Validation error in '{xml_file_path}': {e}")
        return False

def check_id_references(xml_file_path: str) -> bool:
    """Verify referential integrity of ID/IDREF attributes in XML.
    
    Performs a two-pass validation:
    1. First pass collects all ID attributes
    2. Second pass verifies all IDREF attributes point to existing IDs
    
    This ensures that all cross-references within the XML document are valid
    and prevents dangling references that could cause runtime errors.
    
    Args:
        xml_file_path: Path to the XML file to check
        
    Returns:
        bool: True if all references are valid, False otherwise
    """
    ensuring that every referenced ID exists in the document.

    Args:
        xml_file_path: Path to the XML file to check

    Returns:
        bool: True if all references are valid, False otherwise
    """
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

def check_entity_imports(xsd_file_path: str, base_dir: str) -> bool:
    """Verify that all imported/included schema files exist and are accessible.
    
    Validates the schema dependency chain by:
    1. Parsing the schema file
    2. Finding all xs:include and xs:import elements
    3. Verifying each referenced schema exists in the base directory
    
    This prevents schema compilation failures due to missing dependencies.
    Critical for maintaining schema modularity and reuse.
    
    Args:
        xsd_file_path: Path to the XSD schema file to check
        base_dir: Base directory to look for imported schemas
        
    Returns:
        bool: True if all imports are valid, False if any imports are missing
    """
    elements can be found in the specified base directory.

    Args:
        xsd_file_path: Path to the XSD schema file to check
        base_dir: Base directory to look for imported schemas

    Returns:
        bool: True if all imports are valid, False if any imports are missing
    """
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

def validate_naming_conventions(xml_file_path: str) -> bool:
    """Verify XML elements and attributes follow project naming conventions.
    
    Enforces lowercase naming requirements for:
    - Element names
    - Attribute names
    
    This maintains consistency across XML files and aligns with
    project coding standards. Particularly important for:
    - Data exchange formats
    - Configuration files
    - Game data definitions
    
    Args:
        xml_file_path: Path to the XML file to validate
        
    Returns:
        bool: True if all names follow conventions, False otherwise
    """
    as per project conventions.

    Args:
        xml_file_path: Path to the XML file to validate

    Returns:
        bool: True if all names follow conventions, False otherwise
    """
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

def check_unused_ids(xml_file_path: str) -> bool:
    """Check for orphaned ID attributes that are never referenced.
    
    Performs static analysis to:
    1. Collect all defined ID attributes
    2. Track all IDREF references
    3. Identify IDs that are never referenced
    
    While unused IDs don't cause errors, they may indicate:
    - Obsolete definitions
    - Incomplete refactoring
    - Dead code/data
    
    This helps maintain clean and efficient XML documents.
    
    Args:
        xml_file_path: Path to the XML file to check
        
    Returns:
        bool: True if no unused IDs found, False if unused IDs exist
    """
    referenced by any other elements, which may indicate unused
    or obsolete definitions.

    Args:
        xml_file_path: Path to the XML file to check

    Returns:
        bool: True if no unused IDs found, False if unused IDs exist
    """
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
