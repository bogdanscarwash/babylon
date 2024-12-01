class Contradiction:
    def __init__(self, id, name, description, entities, universality,
                 particularity, principal_contradiction, principal_aspect,
                 secondary_aspect, antagonism, intensity, state,
                 potential_for_transformation, conditions_for_transformation,
                 resolution_methods, resolution_conditions, effects, attributes):
        self.id = id
        self.name = name
        self.description = description
        self.entities = entities  # List of Entity objects
        self.universality = universality
        self.particularity = particularity
        self.principal_contradiction = principal_contradiction
        self.principal_aspect = principal_aspect
        self.secondary_aspect = secondary_aspect
        self.antagonism = antagonism
        self.intensity = intensity
        self.intensity_value = 0.0  # Numerical intensity value between 0 and 1
        self.intensity_history = []  # Track historical intensity values
        self.state = state
        self.potential_for_transformation = potential_for_transformation
        self.conditions_for_transformation = conditions_for_transformation
        self.resolution_methods = resolution_methods
        self.resolution_conditions = resolution_conditions
        self.effects = effects  # List of Effect objects
        self.attributes = attributes  # Dictionary of additional attributes

class Entity:
    def __init__(self, entity_id, entity_type, role):
        self.entity_id = entity_id
        self.entity_type = entity_type
        self.role = role
        self.game_entity = None  # Reference to actual game entity

class Effect:
    def __init__(self, target, attribute, modification_type, value, description):
        self.target = target
        self.attribute = attribute
        self.modification_type = modification_type
        self.value = value
        self.description = description
        
    def apply(self, game_state):
        """Apply the effect to the target within the game state."""
        if isinstance(self.target, str):
            entity = game_state.entity_registry.get_entity(self.target)
        else:
            entity = self.target

        if entity and hasattr(entity, self.attribute):
            current_value = getattr(entity, self.attribute)
            if self.modification_type == 'Increase':
                setattr(entity, self.attribute, current_value + self.value)
            elif self.modification_type == 'Decrease':
                setattr(entity, self.attribute, current_value - self.value)
            elif self.modification_type == 'Change':
                setattr(entity, self.attribute, self.value)

    def update_intensity(self, game_state):
        """Calculate and update the intensity of this contradiction."""
        # Base method - should be overridden by specific contradiction types
        pass

class Attribute:
    def __init__(self, name, value):
        self.name = name
        self.value = value
