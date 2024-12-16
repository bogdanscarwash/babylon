from datetime import datetime
from typing import Any

import matplotlib.pyplot as plt
import networkx as nx

from babylon.data.entity_registry import EntityRegistry
from babylon.data.models.contradiction import Contradiction, Effect
from babylon.data.models.event import Event
from babylon.data.models.trigger import Trigger
from babylon.core.entity import Entity
from babylon.metrics.collector import MetricsCollector


class ContradictionAnalysis:
    """System for analyzing and managing dialectical contradictions in the game."""

    def __init__(self, entity_registry: EntityRegistry, metrics: MetricsCollector) -> None:
        self.entity_registry: EntityRegistry = entity_registry
        self.contradictions: list[Contradiction] = []
        self.metrics = metrics

    def add_contradiction(self, contradiction: Contradiction) -> None:
        """Add a new contradiction to the analysis system."""
        try:
            start_time = datetime.now()
            self.contradictions.append(contradiction)
            self._link_contradiction_entities(contradiction)

            # Record metrics
            self.metrics.record_object_access(contradiction.id, "contradiction_system")
            processing_time = (datetime.now() - start_time).total_seconds() * 1000
            self.metrics.record_query_latency(processing_time)
        except Exception as e:
            self.metrics.record_metric("error:add_contradiction", 1.0, str(e))
            raise

    def _update_contradiction(self, contradiction: Contradiction, game_state: dict[str, Any]) -> None:
        """Update a single contradiction's state."""
        try:
            if contradiction is None:
                self.metrics.record_metric("error:update_contradiction", 1.0, "update_nonexistent_contradiction")
                raise AttributeError("Cannot update None contradiction")

            # Record context switch start
            context_switch_start = datetime.now()

            old_intensity: str = contradiction.intensity

            # Update intensity using the instance method
            contradiction.update_intensity(game_state)

            # Record intensity history
            contradiction.intensity_history.append(contradiction.intensity_value)
            if len(contradiction.intensity_history) > 10:
                contradiction.intensity_history.pop(0)

            # Record context switch duration
            context_switch_duration = (datetime.now() - context_switch_start).total_seconds() * 1000
            self.metrics.record_context_switch(context_switch_duration)

            # Log intensity changes
            if contradiction.intensity != old_intensity:
                print(
                    f"Contradiction '{contradiction.name}' intensity changed from {old_intensity} to {contradiction.intensity}"
                )

            # Check for resolution or transformation
            if self._check_resolution_conditions(contradiction, game_state):
                self._resolve_contradiction(contradiction, game_state)
            elif self._check_transformation_conditions(contradiction, game_state):
                self._transform_contradiction(contradiction, game_state)

        except Exception as e:
            self.metrics.record_metric("error:update_contradiction", 1.0, str(e))
            raise

    def _link_contradiction_entities(self, contradiction: Contradiction) -> None:
        """Link contradiction entities to actual game entities."""
        try:
            for entity in contradiction.entities:
                actual_entity = self.entity_registry.get_entity(entity.id)
                entity.game_entity = actual_entity
        except Exception as e:
            self.metrics.record_metric("error:link_entities", 1.0, str(e))
            raise

    def detect_new_contradictions(self, game_state: dict[str, Any]) -> list[Contradiction]:
        """Detect and create new contradictions based on the current game state."""
        try:
            new_contradictions: list[Contradiction] = []

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
        except Exception as e:
            self.metrics.record_metric("error:detect_contradictions", 1.0, str(e))
            raise

    def _check_economic_inequality(self, game_state: dict[str, Any]) -> bool:
        """Check if economic inequality exceeds a threshold."""
        try:
            gini_coefficient = game_state["economy"].gini_coefficient
            inequality_threshold = 0.4
            if gini_coefficient >= inequality_threshold:
                return not self._contradiction_exists("economic_inequality")
            return False
        except Exception as e:
            self.metrics.record_metric("error:check_economic_inequality", 1.0, str(e))
            return False

    def _check_political_unrest(self, game_state: dict[str, Any]) -> bool:
        """Check if political stability is below a threshold."""
        try:
            stability_index = game_state["politics"].stability_index
            unrest_threshold = 0.3
            if stability_index <= unrest_threshold:
                return not self._contradiction_exists("political_unrest")
            return False
        except Exception as e:
            self.metrics.record_metric("error:check_political_unrest", 1.0, str(e))
            return False

    def _contradiction_exists(self, contradiction_id: str) -> bool:
        """Check if a contradiction already exists."""
        try:
            return any(
                c.id == contradiction_id and c.state != "Resolved"
                for c in self.contradictions
            )
        except Exception as e:
            self.metrics.record_metric("error:check_contradiction_exists", 1.0, str(e))
            return False

    def _check_resolution_conditions(self, contradiction: Contradiction, game_state: dict[str, Any]) -> bool:
        """Check if resolution conditions are met."""
        try:
            if contradiction.id == "economic_inequality":
                gini_coefficient = game_state["economy"].gini_coefficient
                return gini_coefficient <= 0.35
            elif contradiction.id == "political_unrest":
                stability_index = game_state["politics"].stability_index
                return stability_index >= 0.5
            return False
        except Exception as e:
            self.metrics.record_metric("error:check_resolution", 1.0, str(e))
            return False

    def _check_transformation_conditions(self, contradiction: Contradiction, game_state: dict[str, Any]) -> bool:
        """Check if transformation conditions are met."""
        try:
            for condition in contradiction.conditions_for_transformation:
                if not self._evaluate_condition(condition, game_state):
                    return False
            return True
        except Exception as e:
            self.metrics.record_metric("error:check_transformation", 1.0, str(e))
            return False

    def _evaluate_condition(self, condition: str, game_state: dict[str, Any]) -> bool:
        """Evaluate a condition based on game state."""
        try:
            # Implement condition evaluation logic
            return False
        except Exception as e:
            self.metrics.record_metric("error:evaluate_condition", 1.0, str(e))
            return False

    def _resolve_contradiction(self, contradiction: Contradiction, game_state: dict[str, Any]) -> None:
        """Resolve a contradiction through the selected resolution method."""
        try:
            resolution_method = self._select_resolution_method(contradiction, game_state)
            contradiction.selected_resolution_method = resolution_method
            contradiction.state = f"Resolved by {resolution_method}"

            effects = contradiction.resolution_methods.get(resolution_method, [])
            self._apply_effects(effects, game_state)

            print(f"Contradiction '{contradiction.name}' resolved through {resolution_method}.")
            self._post_resolution_check(contradiction, game_state)
        except Exception as e:
            self.metrics.record_metric("error:resolve_contradiction", 1.0, str(e))
            raise

    def _transform_contradiction(self, contradiction: Contradiction, game_state: dict[str, Any]) -> None:
        """Transform a contradiction into a new qualitative state."""
        try:
            # Implement transformation logic
            pass
        except Exception as e:
            self.metrics.record_metric("error:transform_contradiction", 1.0, str(e))
            raise

    def _apply_effects(self, effects: list[Effect], game_state: dict[str, Any]) -> None:
        """Apply contradiction effects to the game state."""
        try:
            for effect in effects:
                target_entity = self.entity_registry.get_entity(effect.target_id)
                if target_entity:
                    self._modify_attribute(target_entity, effect)
        except Exception as e:
            self.metrics.record_metric("error:apply_effects", 1.0, str(e))
            raise

    def _select_resolution_method(self, contradiction: Contradiction, game_state: dict[str, Any]) -> str:
        """Determine the resolution method for a contradiction."""
        try:
            if game_state.get("is_player_responsible", False):
                available_methods = list(contradiction.resolution_methods.keys())
                print(f"Choose a resolution method for '{contradiction.name}':")
                for idx, method in enumerate(available_methods, 1):
                    print(f"{idx}. {method}")
                choice = int(input("Enter the number of your choice: "))
                return available_methods[choice - 1]
            else:
                return self._ai_select_resolution_method(contradiction, game_state)
        except Exception as e:
            self.metrics.record_metric("error:select_resolution_method", 1.0, str(e))
            raise

    def _ai_select_resolution_method(self, contradiction: Contradiction, game_state: dict[str, Any]) -> str:
        """AI selects a resolution method based on strategy."""
        try:
            if contradiction.intensity == "High" and "Revolution" in contradiction.resolution_methods:
                return "Revolution"
            elif contradiction.intensity == "Medium" and "Reform" in contradiction.resolution_methods:
                return "Reform"
            elif "Suppression" in contradiction.resolution_methods:
                return "Suppression"
            return list(contradiction.resolution_methods.keys())[0]  # Fallback
        except Exception as e:
            self.metrics.record_metric("error:ai_select_resolution", 1.0, str(e))
            raise

    def _post_resolution_check(self, contradiction: Contradiction, game_state: dict[str, Any]) -> None:
        """Handle side effects and potential new contradictions after resolution."""
        try:
            method = contradiction.selected_resolution_method
            if method == "Suppression":
                print(f"Suppression of '{contradiction.name}' may lead to further unrest.")
                self._check_for_new_contradictions(contradiction, game_state)
            elif method == "Reform":
                print(f"Reforms implemented for '{contradiction.name}'. Stability may improve.")
            elif method == "Revolution":
                print(f"Revolution occurred due to '{contradiction.name}'. Game state changed significantly.")
        except Exception as e:
            self.metrics.record_metric("error:post_resolution_check", 1.0, str(e))
            raise

    def _modify_attribute(self, target: Entity, effect: Effect) -> None:
        """Modify an entity's attribute based on an effect."""
        try:
            if hasattr(target, effect.attribute):
                current_value = getattr(target, effect.attribute)
                if effect.operation == "Increase":
                    new_value = current_value + effect.magnitude
                elif effect.operation == "Decrease":
                    new_value = current_value - effect.magnitude
                else:  # Change
                    new_value = effect.magnitude
                setattr(target, effect.attribute, new_value)
                print(f"{effect.description}: {target} {effect.operation}d {effect.attribute} by {effect.magnitude}.")
        except Exception as e:
            self.metrics.record_metric("error:modify_attribute", 1.0, str(e))
            raise

    def _check_for_new_contradictions(self, contradiction: Contradiction, game_state: dict[str, Any]) -> None:
        """Check for new contradictions that may arise from resolution."""
        try:
            # Implementation for checking new contradictions
            pass
        except Exception as e:
            self.metrics.record_metric("error:check_new_contradictions", 1.0, str(e))
            raise
