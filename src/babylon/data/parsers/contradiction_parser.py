import xml.etree.ElementTree as ET

from ..models.contradiction import Contradiction, Effect, Entity


def parse_contradictions(xml_file):
    """Parse contradiction definitions from XML file."""
    tree = ET.parse(xml_file)
    root = tree.getroot()
    contradictions = []

    for contradiction_elem in root.findall("Contradiction"):
        # Parse basic information
        id = contradiction_elem.find("ID").text
        name = (
            contradiction_elem.find("Name").text
            if contradiction_elem.find("Name") is not None
            else None
        )
        description = contradiction_elem.find("Description").text

        # Parse entities
        entities = []
        between_entities_elem = contradiction_elem.find("BetweenEntities")
        for entity_elem in between_entities_elem.findall("Entity"):
            entity_id = entity_elem.find("EntityID").text
            entity_type = entity_elem.find("EntityType").text
            role = (
                entity_elem.find("Role").text
                if entity_elem.find("Role") is not None
                else None
            )
            entities.append(Entity(entity_id, entity_type, role))

        # Parse contradiction nature
        universality = contradiction_elem.find("Universality").text.lower() == "true"
        particularity = contradiction_elem.find("Particularity").text
        principal_contradiction = (
            contradiction_elem.find("PrincipalContradiction").text.lower() == "true"
        )
        principal_aspect = contradiction_elem.find("PrincipalAspect").text
        secondary_aspect_elem = contradiction_elem.find("SecondaryAspect")
        secondary_aspect = (
            secondary_aspect_elem.text if secondary_aspect_elem is not None else None
        )

        # Parse attributes
        antagonism = contradiction_elem.find("Antagonism").text
        intensity = contradiction_elem.find("Intensity").text
        state = contradiction_elem.find("State").text

        # Parse transformation
        potential_for_transformation = (
            contradiction_elem.find("PotentialForTransformation").text.lower() == "true"
        )
        conditions_for_transformation = [
            cond.text
            for cond in contradiction_elem.find("ConditionsForTransformation").findall(
                "Condition"
            )
        ]

        # Parse resolution
        resolution_methods = [
            method.text
            for method in contradiction_elem.find("ResolutionMethods").findall("Method")
        ]
        resolution_conditions = [
            cond.text
            for cond in contradiction_elem.find("ResolutionConditions").findall(
                "Condition"
            )
        ]

        # Parse effects
        effects = []
        for effect_elem in contradiction_elem.find("Effects").findall("Effect"):
            target = effect_elem.find("Target").text
            attribute = effect_elem.find("Attribute").text
            modification_type = effect_elem.find("ModificationType").text
            value_elem = effect_elem.find("Value")
            value = float(value_elem.text) if value_elem is not None else None
            description_elem = effect_elem.find("Description")
            description = (
                description_elem.text if description_elem is not None else None
            )
            effects.append(
                Effect(target, attribute, modification_type, value, description)
            )

        # Parse additional attributes
        attributes = {}
        attributes_elem = contradiction_elem.find("Attributes")
        if attributes_elem is not None:
            for attr_elem in attributes_elem.findall("Attribute"):
                name = attr_elem.find("Name").text
                value = attr_elem.find("Value").text
                attributes[name] = value

        contradiction = Contradiction(
            id,
            name,
            description,
            entities,
            universality,
            particularity,
            principal_contradiction,
            principal_aspect,
            secondary_aspect,
            antagonism,
            intensity,
            state,
            potential_for_transformation,
            conditions_for_transformation,
            resolution_methods,
            resolution_conditions,
            effects,
            attributes,
        )

        contradictions.append(contradiction)

    return contradictions
