# AI Communications Guide

## Overview
This document outlines the approach for using Large Language Models (LLMs) like Claude as an intelligent game master for our Marxist political simulation.

## Vector Database Integration

### Basic Flow
1. Present users with pre-generated options plus custom input ability
2. Convert user input into vector embeddings
3. Compare against stored vectors in database
4. Retrieve top N most relevant entries (typically 10)
5. Use these entries to build context for the AI

### Vector Database Contents
- Previous actions and outcomes
- Entity descriptions and properties  
- Historical events and consequences
- Contradiction patterns and resolutions

## Prompting Guidelines

### Core Principles
1. Ground all responses in material conditions
2. Maintain internal consistency 
3. Consider class relations and power dynamics
4. Provide structured, parseable responses
5. Include both mechanical effects and narrative description
6. Keep track of historical continuity
7. Allow for emergence of new contradictions

### Basic Prompt Structure
```python
def create_base_prompt():
    return """You are the game master for a Marxist political simulation. Your role is to:
    - Analyze player actions through dialectical materialism
    - Generate realistic consequences based on material conditions
    - Maintain internal consistency with previous events
    - Escalate or de-escalate contradictions appropriately
    - Consider class interests and power relations in all outcomes"""
```

### Context Hierarchy
Important elements to include:

1. Material Conditions
   - Economic conditions
   - Class relations
   - Political power distribution

2. Active Contradictions
   - Current intensity
   - Involved entities
   - Historical development

3. Recent Events
   - Outcomes
   - Affected parties
   - Changes to material conditions

4. Player Status
   - Position
   - Resources
   - Relationships

### Response Format
Request structured responses like:
```
IMMEDIATE EFFECTS:
- [Effect 1]
- [Effect 2]

CONTRADICTION CHANGES:
- [Contradiction ID]: [Change in intensity] because [reasoning]

NEW EVENTS:
- [Event description]
- Likelihood: [High/Medium/Low]
- Affected entities: [List]

NARRATIVE DESCRIPTION:
[2-3 sentences describing outcomes in narrative form]
```

## Implementation Examples

### Processing XML Data
Rather than sending raw XML to the AI, parse and extract relevant information:

```python
def extract_relevant_context(xml_objects):
    context_elements = []
    for obj in xml_objects:
        if isinstance(obj, Event):
            context = {
                'type': 'event',
                'name': obj.name,
                'description': obj.description,
                'effects': [e.description for e in obj.effects],
                'escalation_level': obj.escalation_level
            }
        elif isinstance(obj, Contradiction):
            context = {
                'type': 'contradiction',
                'name': obj.name,
                'description': obj.description,
                'intensity': obj.intensity,
                'involved_entities': [e.name for e in obj.entities]
            }
        context_elements.append(context)
    return context_elements
```

### Specialized Prompts

#### Economic Actions
```python
def economic_action_prompt(action, state):
    return f"""
    The player has taken an economic action: {action}
    
    Current Economic Indicators:
    - Unemployment Rate: {state.unemployment}
    - Class Inequality Index: {state.inequality}
    - Industrial Capacity: {state.industry_capacity}
    
    How do the following groups respond:
    1. Working Class
    2. Capitalist Class
    3. State Apparatus
    """
```

#### Political Actions
```python
def political_action_prompt(action, state):
    return f"""
    The player has taken a political action: {action}
    
    Current Political Landscape:
    - Class Consciousness Level: {state.class_consciousness}
    - State Legitimacy: {state.legitimacy}
    - Opposition Strength: {state.opposition_power}
    
    Analyze effects on:
    1. Balance of class forces
    2. State authority
    3. Popular support
    4. Potential for resistance
    """
```

## API Integration

### Claude API Usage
Using Claude's 200k token context window effectively:

1. Format the request:
```python
def generate_game_response(prompt):
    response = anthropic.messages.create(
        model="claude-2.1",
        max_tokens=1024,
        temperature=0.7,
        system="You are an AI game master for a Marxist political simulation.",
        messages=[{"role": "user", "content": prompt}]
    )
    return parse_claude_response(response.content)
```

2. Parse the response:
```python
def parse_claude_response(response):
    game_effects = {
        'immediate_effects': [],
        'new_contradictions': [],
        'contradiction_changes': [],
        'triggered_events': [],
        'entity_effects': []
    }
    return game_effects
```

## Best Practices

1. **Consistency Management**
   - Track all significant decisions and outcomes
   - Maintain a history of events and their consequences
   - Ensure new developments follow logically from established patterns

2. **Context Building**
   - Only include relevant information
   - Structure context hierarchically
   - Prioritize recent and directly related events

3. **Response Parsing**
   - Use structured formats for easy parsing
   - Include both mechanical and narrative elements
   - Validate responses against game rules and logic

4. **Error Handling**
   - Have fallback options for unclear responses
   - Validate AI outputs against game constraints
   - Maintain game stability if AI service is unavailable

## Context Window Management

The system implements a robust Context Window Management component to efficiently utilize Claude's 200k token context window:

1. **ContextWindowManager**
   - Tracks token usage across all content
   - Prioritizes content based on importance scores
   - Automatically optimizes context when reaching 75% capacity (configurable)
   - Integrates with metrics collection for performance monitoring

2. **Content Prioritization**
   - Uses hybrid prioritization strategy (configurable)
   - Considers content importance, recency, and access frequency
   - Maintains priority queue for efficient content management
   - Preserves most relevant content when optimization is needed

3. **Integration**
   - Works with MetricsCollector to track token usage
   - Prepares for LifecycleManager integration
   - Configurable through ContextWindowConfig

## Future Improvements

1. **Enhanced Context Selection**
   - Improve vector similarity matching
   - Develop better context relevance scoring
   - Enhance existing context window management with more sophisticated algorithms

2. **Response Quality**
   - Fine-tune prompts based on response quality
   - Develop better parsing for complex responses
   - Implement validation for logical consistency

3. **Performance Optimization**
   - Cache common responses
   - Optimize vector searches
   - Reduce API calls when possible
