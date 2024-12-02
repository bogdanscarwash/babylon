from typing import Any, Dict, List, Optional, Union
import matplotlib.pyplot as plt
import networkx as nx
from ..data.models.contradiction import Contradiction, Effect, Entity
from ..data.models.event import Event
from ..data.models.entity import EntityRegistry

class ContradictionAnalysis:
    """System for analyzing and managing contradictions in the game."""
    
    def __init__(self, entity_registry: EntityRegistry) -> None:
        self.entity_registry: EntityRegistry = entity_registry
        self.contradictions: List[Contradiction] = []
        
    def add_contradiction(self, contradiction: Contradiction) -> None:
        """Add a new contradiction to the system."""
        self.contradictions.append(contradiction)
        self._link_contradiction_entities(contradiction)
        
    def _link_contradiction_entities(self, contradiction: Contradiction) -> None:
        """Link contradiction entities to actual game entities."""
        for entity in contradiction.entities:
            actual_entity = self.entity_registry.get_entity(entity.entity_id)
            entity.game_entity = actual_entity
            
    def detect_new_contradictions(self, game_state: Dict[str, Any]) -> List[Contradiction]:
        """Detect new contradictions based on the game state."""
        new_contradictions: List[Contradiction] = []

        # Economic inequality check
        if self._check_economic_inequality(game_state):
            contradiction = self._create_economic_inequality_contradiction(game_state)
            new_contradictions.append(contradiction)

        # Political unrest check
        if self._check_political_unrest(game_state):
            contradiction = self._create_political_unrest_contradiction(game_state)
            new_contradictions.append(contradiction)

        # Add detected contradictions
        for contradiction in new_contradictions:
            self.add_contradiction(contradiction)
            
        return new_contradictions

    def _check_economic_inequality(self, game_state: Dict[str, Any]) -> bool:
        """Check if economic inequality exceeds a threshold."""
        gini_coefficient = game_state['economy'].gini_coefficient
        inequality_threshold = 0.4  # Define thresholds as per game design
        if gini_coefficient >= inequality_threshold:
            return not self._contradiction_exists('economic_inequality')
        return False

    def _check_political_unrest(self, game_state: Dict[str, Any]) -> bool:
        """Check if political stability is below a threshold."""
        stability_index = game_state['politics'].stability_index
        unrest_threshold = 0.3
        if stability_index <= unrest_threshold:
            return not self._contradiction_exists('political_unrest')
        return False

    def _contradiction_exists(self, contradiction_id: str) -> bool:
        """Check if a contradiction already exists."""
        return any(c.id == contradiction_id and c.state != 'Resolved' 
                  for c in self.contradictions)

    def _create_economic_inequality_contradiction(self, game_state):
        """Create an economic inequality contradiction."""
        upper_class = Entity('upper_class', 'Class', 'Oppressor')
        working_class = Entity('working_class', 'Class', 'Oppressed')
        entities = [upper_class, working_class]

        contradiction = Contradiction(
            id='economic_inequality',
            name='Economic Inequality',
            description='Growing disparity between rich and poor.',
            entities=entities,
            universality='Universal',
            particularity='Economic',
            principal_contradiction=None,
            principal_aspect=upper_class,
            secondary_aspect=working_class,
            antagonism='Antagonistic',
            intensity='Medium',
            state='Active',
            potential_for_transformation='High',
            conditions_for_transformation=['Revolutionary Movement'],
            resolution_methods=['Policy Reform', 'Revolution'],
            resolution_conditions=['Reduce Inequality'],
            effects=[],
            attributes={}
        )
        
        # Define resolution methods and their effects
        contradiction.resolution_methods = {
            'Suppression': [Effect('working_class', 'freedom', 'Decrease', 0.1, 'Suppress dissent')],
            'Reform': [Effect('economy', 'gini_coefficient', 'Decrease', 0.1, 'Implement wealth redistribution')],
            'Revolution': [
                Effect('upper_class', 'wealth', 'Decrease', 0.5, 'Expropriate assets'),
                Effect('working_class', 'wealth', 'Increase', 0.5, 'Redistribute wealth')
            ]
        }
        
        # Add custom intensity update method
        def update_intensity(self, game_state):
            gini_coefficient = game_state['economy'].gini_coefficient
            unemployment_rate = game_state['economy'].unemployment_rate
            
            # Define weights
            gini_weight = 0.7
            unemployment_weight = 0.3
            
            # Calculate weighted intensity value
            self.intensity_value = (
                gini_weight * gini_coefficient +
                unemployment_weight * unemployment_rate
            )
            
            # Set categorical intensity based on value
            if self.intensity_value >= 0.6:
                self.intensity = 'High'
            elif self.intensity_value >= 0.4:
                self.intensity = 'Medium'
            else:
                self.intensity = 'Low'
        
        # Bind the method to the contradiction instance
        from types import MethodType
        contradiction.update_intensity = MethodType(update_intensity, contradiction)
        
        return contradiction

    def _create_political_unrest_contradiction(self, game_state):
        """Create a political unrest contradiction."""
        government = Entity('government', 'Organization', 'Oppressor')
        citizens = Entity('citizens', 'Faction', 'Oppressed')
        entities = [government, citizens]

        contradiction = Contradiction(
            id='political_unrest',
            name='Political Unrest',
            description='Citizens are losing trust in the government.',
            entities=entities,
            universality='Universal',
            particularity='Political',
            principal_contradiction=None,
            principal_aspect=government,
            secondary_aspect=citizens,
            antagonism='Antagonistic',
            intensity='Medium',
            state='Active',
            potential_for_transformation='Medium',
            conditions_for_transformation=['Mass Protests'],
            resolution_methods=['Policy Changes', 'Suppression'],
            resolution_conditions=['Increase Stability'],
            effects=[],
            attributes={}
        )
        
        # Define resolution methods and their effects
        contradiction.resolution_methods = {
            'Suppression': [Effect('citizens', 'freedom', 'Decrease', 0.2, 'Impose martial law')],
            'Reform': [Effect('politics', 'stability_index', 'Increase', 0.2, 'Enact democratic reforms')],
            'Revolution': [Effect('government', 'power', 'Decrease', 1.0, 'Overthrow the government')]
        }
        return contradiction
        
    def update_contradictions(self, game_state: Dict[str, Any]) -> None:
        """Update all active contradictions based on current game state."""
        for contradiction in self.contradictions:
            if contradiction.state != 'Resolved':
                self._update_contradiction(contradiction, game_state)
                
        # Generate events after updating contradictions
        new_events: List[Event] = self.generate_events(game_state)
        # Add events to the game state's event queue
        game_state['event_queue'].extend(new_events)
                
    def _update_contradiction(self, contradiction: Contradiction, game_state: Dict[str, Any]) -> None:
        """Update a single contradiction's state."""
        old_intensity: str = contradiction.intensity
        
        # Update intensity using contradiction's own method
        contradiction.update_intensity(game_state)
        
        # Record intensity history
        contradiction.intensity_history.append(contradiction.intensity_value)
        if len(contradiction.intensity_history) > 10:
            contradiction.intensity_history.pop(0)
            
        # Log intensity changes
        if contradiction.intensity != old_intensity:
            print(f"Contradiction '{contradiction.name}' intensity changed from {old_intensity} to {contradiction.intensity}")
        
        # Check for resolution or transformation
        if self._check_resolution_conditions(contradiction, game_state):
            self._resolve_contradiction(contradiction, game_state)
        elif self._check_transformation_conditions(contradiction, game_state):
            self._transform_contradiction(contradiction, game_state)
            
    def _calculate_intensity(self, contradiction, game_state):
        """Calculate the intensity of a contradiction."""
        if contradiction.id == 'economic_inequality':
            gini_coefficient = game_state['economy'].gini_coefficient
            if gini_coefficient >= 0.6:
                return 'High'
            elif gini_coefficient >= 0.4:
                return 'Medium'
            else:
                return 'Low'
        elif contradiction.id == 'political_unrest':
            stability_index = game_state['politics'].stability_index
            if stability_index <= 0.2:
                return 'High'
            elif stability_index <= 0.3:
                return 'Medium'
            else:
                return 'Low'
        else:
            return 'Low'
        
    def _check_resolution_conditions(self, contradiction, game_state):
        """Check if conditions for resolution are met."""
        if contradiction.id == 'economic_inequality':
            gini_coefficient = game_state['economy'].gini_coefficient
            return gini_coefficient <= 0.35  # Threshold for resolution
        elif contradiction.id == 'political_unrest':
            stability_index = game_state['politics'].stability_index
            return stability_index >= 0.5
        return False
        
    def _resolve_contradiction(self, contradiction, game_state):
        """Resolve a contradiction using the selected resolution method."""
        resolution_method = self._select_resolution_method(contradiction, game_state)
        contradiction.selected_resolution_method = resolution_method
        contradiction.state = f'Resolved by {resolution_method}'
        
        effects = contradiction.resolution_methods.get(resolution_method, [])
        self._apply_effects(effects, game_state)
        
        print(f"Contradiction '{contradiction.name}' resolved through {resolution_method}.")
        self._post_resolution_check(contradiction, game_state)
        
    def _check_transformation_conditions(self, contradiction, game_state):
        """Check if conditions for transformation are met."""
        for condition in contradiction.conditions_for_transformation:
            if not self._evaluate_condition(condition, game_state):
                return False
        return True
        
    def _transform_contradiction(self, contradiction, game_state):
        """Transform a contradiction's nature."""
        # Implement transformation logic
        pass
        
    def _apply_effects(self, effects, game_state):
        """Apply contradiction effects to the game state."""
        for effect in effects:
            target_entity = self.entity_registry.get_entity(effect.target)
            if target_entity:
                self._modify_attribute(target_entity, effect)
                
    def _select_resolution_method(self, contradiction, game_state):
        """Determine the resolution method for a contradiction."""
        if game_state.get('is_player_responsible', False):
            available_methods = list(contradiction.resolution_methods.keys())
            print(f"Choose a resolution method for '{contradiction.name}':")
            for idx, method in enumerate(available_methods, 1):
                print(f"{idx}. {method}")
            choice = int(input("Enter the number of your choice: "))
            return available_methods[choice - 1]
        else:
            return self._ai_select_resolution_method(contradiction, game_state)

    def _ai_select_resolution_method(self, contradiction, game_state):
        """AI selects a resolution method based on strategy."""
        if contradiction.intensity == 'High' and 'Revolution' in contradiction.resolution_methods:
            return 'Revolution'
        elif contradiction.intensity == 'Medium' and 'Reform' in contradiction.resolution_methods:
            return 'Reform'
        elif 'Suppression' in contradiction.resolution_methods:
            return 'Suppression'
        return list(contradiction.resolution_methods.keys())[0]  # Fallback

    def _post_resolution_check(self, contradiction, game_state):
        """Handle side effects and potential new contradictions after resolution."""
        method = contradiction.selected_resolution_method
        if method == 'Suppression':
            print(f"Suppression of '{contradiction.name}' may lead to further unrest.")
            self._check_for_new_contradictions(contradiction, game_state)
        elif method == 'Reform':
            print(f"Reforms implemented for '{contradiction.name}'. Stability may improve.")
        elif method == 'Revolution':
            print(f"Revolution occurred due to '{contradiction.name}'. Game state changed significantly.")

    def _modify_attribute(self, target, effect):
        """Modify an entity's attribute based on an effect."""
        if hasattr(target, effect.attribute):
            current_value = getattr(target, effect.attribute)
            if effect.modification_type == 'Increase':
                new_value = current_value + effect.value
            elif effect.modification_type == 'Decrease':
                new_value = current_value - effect.value
            else:  # Change
                new_value = effect.value
            setattr(target, effect.attribute, new_value)
            print(f"{effect.description}: {target} {effect.modification_type}d {effect.attribute} by {effect.value}.")
            
    def generate_events(self, game_state):
        """Generate events based on active contradictions."""
        events = []
        for contradiction in self.contradictions:
            if contradiction.state == 'Active':
                # Generate event based on contradiction properties
                event = self._create_event_from_contradiction(contradiction, game_state)
                if event:
                    events.append(event)
        return events
        
    def _create_event_from_contradiction(self, contradiction, game_state):
        """Create an Event object procedurally based on a Contradiction."""
        event_id = f"event_{contradiction.id}_{len(contradiction.intensity_history)}"
        event_name = f"{contradiction.intensity} {contradiction.name}"
        event_description = (
            f"The contradiction '{contradiction.name}' involving "
            f"{', '.join([entity.entity_id for entity in contradiction.entities])} "
            f"is escalating."
        )
        
        # Procedurally generate effects based on contradiction's intensity and entities
        effects = self._generate_effects_from_contradiction(contradiction, game_state)
        
        triggers = []  # Define any triggers if necessary
        escalation_level = self._determine_escalation_level(contradiction)
        
        return Event(event_id, event_name, event_description, effects, triggers, escalation_level)

    def _generate_effects_from_contradiction(self, contradiction, game_state):
        """Generate a list of Effect objects based on the contradiction."""
        effects = []
        for entity in contradiction.entities:
            target = entity.entity_id
            attribute = 'stability'  # Example attribute affected
            
            # Determine effect based on intensity
            if contradiction.intensity == 'High':
                modification_type = 'Decrease'
                value = 0.3
                description = f"{contradiction.name} severely impacts {target}'s {attribute}."
            elif contradiction.intensity == 'Medium':
                modification_type = 'Decrease'
                value = 0.2
                description = f"{contradiction.name} moderately affects {target}'s {attribute}."
            else:  # Low intensity
                modification_type = 'Decrease'
                value = 0.1
                description = f"{contradiction.name} slightly affects {target}'s {attribute}."
            
            effect = Effect(target, attribute, modification_type, value, description)
            effects.append(effect)
        return effects
        
    def _determine_escalation_level(self, contradiction):
        """Determine the escalation level based on contradiction intensity and antagonism."""
        if contradiction.intensity == 'High' and contradiction.antagonism == 'Antagonistic':
            return 'Critical'
        elif contradiction.intensity == 'High':
            return 'High'
        elif contradiction.intensity == 'Medium':
            return 'Medium'
        else:
            return 'Low'
            
    def _get_intensity_color(self, intensity: str) -> str:
        """Map intensity levels to colors."""
        return {
            'Low': 'green',
            'Medium': 'yellow',
            'High': 'red'
        }.get(intensity, 'grey')
        
    def _get_entity_color(self, entity_type: str) -> str:
        """Map entity types to colors."""
        color_map = {
            'Faction': 'blue',
            'Class': 'green',
            'Character': 'orange',
            'Organization': 'purple'
        }
        return color_map.get(entity_type, 'grey')

    def visualize_entity_relationships(self):
        """Visualize relationships between entities based on contradictions."""
        G = nx.Graph()

        # Add nodes for entities
        entity_ids = set()
        for contradiction in self.contradictions:
            for entity in contradiction.entities:
                entity_id = entity.entity_id
                entity_type = entity.entity_type
                entity_ids.add((entity_id, entity_type))
                G.add_node(entity_id, label=entity_type)

        # Add edges between entities involved in the same contradiction
        for contradiction in self.contradictions:
            involved_entities = [entity.entity_id for entity in contradiction.entities]
            for i in range(len(involved_entities)):
                for j in range(i + 1, len(involved_entities)):
                    G.add_edge(
                        involved_entities[i],
                        involved_entities[j],
                        label=contradiction.name
                    )

        # Position the nodes using a layout
        pos = nx.spring_layout(G)

        # Prepare node colors based on entity types
        node_colors = [
            self._get_entity_color(G.nodes[node]['label'])
            for node in G.nodes()
        ]

        # Draw nodes with labels and colors
        nx.draw_networkx_nodes(G, pos, node_size=800, node_color=node_colors)
        node_labels = nx.get_node_attributes(G, 'label')
        nx.draw_networkx_labels(G, pos, labels=node_labels, font_size=10)

        # Draw edges
        nx.draw_networkx_edges(G, pos)

        # Add edge labels if not too cluttered
        if len(G.edges()) <= 20:
            edge_labels = nx.get_edge_attributes(G, 'label')
            nx.draw_networkx_edge_labels(G, pos, edge_labels=edge_labels, font_color='gray')

        # Display the graph
        plt.title('Entity Relationship Network')
        plt.axis('off')
        plt.show()

    def visualize_contradictions(self):
        """Visualize contradictions and their relationships."""
        G = nx.DiGraph()

        # Add nodes for contradictions
        for contradiction in self.contradictions:
            G.add_node(contradiction.id, label=contradiction.name,
                      intensity=contradiction.intensity)

        # Add edges for principal contradictions
        for contradiction in self.contradictions:
            if contradiction.principal_contradiction:
                G.add_edge(contradiction.principal_contradiction.id,
                          contradiction.id)

        # Get colors based on numerical intensity values
        max_intensity = max([c.intensity_value for c in self.contradictions] + [1])
        node_colors = [
            plt.cm.hot(c.intensity_value / max_intensity)
            for c in self.contradictions
        ]

        # Create a layout for the nodes
        pos = nx.spring_layout(G)

        # Draw nodes with labels and colors
        nx.draw_networkx_nodes(G, pos, node_size=800, node_color=node_colors)
        labels = {contradiction.id: contradiction.name
                 for contradiction in self.contradictions}
        nx.draw_networkx_labels(G, pos, labels, font_size=10)

        # Draw edges
        nx.draw_networkx_edges(G, pos)

        # Display the graph
        plt.title('Dialectical Map of Contradictions')
        plt.axis('off')
        plt.show()
            
    def _evaluate_condition(self, condition, game_state):
        """Evaluate if a condition is met based on game state."""
        # Implement condition evaluation logic
        return False  # Placeholder
        
    def _check_class_struggle_conditions(self, game_state):
        """Check if conditions for class struggle contradiction exist."""
        # Implement specific detection logic
        return False  # Placeholder
        
    def _create_class_struggle_contradiction(self):
        """Create a new class struggle contradiction."""
        # Implement contradiction creation logic
        return None  # Placeholder
