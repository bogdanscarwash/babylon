class Event:
    def __init__(self, id, name, description, effects, triggers, escalation_level):
        self.id = id
        self.name = name
        self.description = description
        self.effects = effects      # List of Effect objects
        self.triggers = triggers    # Conditions or functions
        self.escalation_level = escalation_level
