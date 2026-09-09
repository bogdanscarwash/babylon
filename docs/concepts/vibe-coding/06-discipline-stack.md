# Part VI: The Discipline Stack

## Why Vibe Coding Without Discipline Fails

The critics have a point: naive AI-assisted development produces garbage. Paste whatever ChatGPT outputs, don't verify, don't test, don't review—and you'll ship bugs, security holes, and unmaintainable messes.

The answer isn't to reject AI assistance. It's to pair it with discipline.

The Babylon project demonstrates what disciplined vibe coding looks like. Here's the stack:

## 1. Pre-Commit Hooks as Guardrails

Every commit passes through automated checks:

```yaml
# From .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: ruff-check
        name: ruff-check
        entry: uv run ruff check --fix
        language: system
        types: [python]

      - id: ruff-format
        name: ruff-format
        entry: uv run ruff format
        language: system
        types: [python]

      - id: mypy
        name: mypy
        entry: uv run mypy
        language: system
        types: [python]

      - id: pytest
        name: pytest
        entry: uv run pytest -x -q
        language: system
        types: [python]

      - id: commitizen
        name: commitizen
        entry: cz check --commit-msg-file
        language: python
```

You cannot commit code that:

- Fails linting (ruff)
- Has type errors (mypy)
- Fails tests (pytest)
- Has a malformed commit message (commitizen)

The AI can generate whatever it wants. The hooks ensure only valid code reaches the repository. This is automatic, unavoidable, and applies equally to human and AI-generated code.

## 2. TDD as the Verification Loop

Test-Driven Development isn't just a testing strategy. It's a verification loop that ensures AI output matches intent.

The cycle:

1. **Red:** Write a test that describes what you want
2. **Green:** Ask AI to implement code that passes the test
3. **Refactor:** Clean up while keeping tests green

The test is the specification. The AI generates an implementation. If the tests pass, the implementation matches the specification. If they fail, iterate.

This is profound: TDD transforms AI assistance from "trust the output" to "verify the output." You don't need to read every line of generated code (though you can). You need to write tests that would fail if the code were wrong.

```python
# From tests/unit/engine/systems/test_solidarity_system.py

def test_consciousness_transmission_when_source_active():
    """Consciousness flows through SOLIDARITY edges when source is active."""
    # Arrange: revolutionary periphery, passive core, solidarity connection
    graph = create_graph_with_solidarity_edge(
        source_consciousness=0.9,  # Revolutionary
        target_consciousness=0.1,  # Passive
        solidarity_strength=0.8,   # Strong connection
    )

    # Act: run solidarity system
    SolidaritySystem().step(graph, services, context)

    # Assert: target consciousness increased
    target = get_node(graph, "C_w")
    assert target.consciousness > 0.1  # Was raised
    assert target.consciousness < 0.9  # Didn't fully equalize
```

This test specifies behavior. If AI-generated code passes it, the behavior is correct. The test doesn't care who wrote the implementation.

## 3. Type Systems as Contracts

Strict typing isn't just about catching bugs. It's about defining contracts that both humans and AIs must respect.

```python
# From src/babylon/models/types.py

# Constrained types that prevent invalid states
Probability = Annotated[float, Field(ge=0.0, le=1.0)]
Currency = Annotated[float, Field(ge=0.0)]
Ideology = Annotated[float, Field(ge=-1.0, le=1.0)]
Intensity = Annotated[float, Field(ge=0.0, le=1.0)]
Coefficient = Annotated[float, Field(ge=0.0, le=1.0)]
```

An AI can't generate code that assigns -5 to a `Probability`. The type system rejects it. The AI can't return `None` from a function typed to return `Currency`. MyPy catches it.

Types are contracts that constrain what the AI can produce. They're not trust—they're verification at the language level.

## 4. ADRs as Decision Memory

Architecture Decision Records document why decisions were made:

```yaml
# From ai/decisions.yaml

ADR016_fascist_bifurcation:
  status: "accepted"
  date: "2025-12-09"
  title: "The Fascist Bifurcation: Solidarity as Infrastructure"
  context: |
    Sprint 3.4.2 implemented Proletarian Internationalism, but a critical
    design question emerged: should solidarity_strength be auto-calculated
    from source organization, or stored as a persistent edge attribute?
  decision: |
    Store solidarity_strength as a PERSISTENT EDGE ATTRIBUTE, not auto-calculated.
  rationale:
    - "Encodes anti-accelerationist theory into mechanics"
    - "Enables emergent Fascist vs Revolutionary outcomes"
    - "Player agency: must choose to BUILD solidarity before crisis"
  mantra: "Agitation without solidarity produces fascism, not revolution."
```

When future sessions encounter this code, they can read why it was designed this way. The AI doesn't need to rediscover the reasoning—it's documented. Decisions compound; knowledge accumulates.

## 5. Session Memory as Context Continuity

The claude-mem system provides continuity between sessions. Recent observations, decisions, and learnings are available at session start:

```
**Legend:** [BUG] bugfix | [FEAT] feature | [REFACT] refactor | [CHG] change | [DISC] discovery | [DEC] decision

| ID | Time | T | Title |
|----|------|---|-------|
| #6270 | 11:58 AM | [FEAT] | Created Comprehensive Architecture Documentation |
| #6271 | 11:58 AM | [FEAT] | Created George Jackson Bifurcation Model Documentation |
| #6279 | 12:03 PM | [CHG] | Committed Comprehensive RST Documentation to Git |
```

This is organizational memory externalized. The AI's context window is ephemeral; the memory system is persistent. Knowledge survives session boundaries.

## Why Discipline Enables Freedom

Paradoxically, these constraints enable freedom. When you know the pre-commit hooks will catch errors, you can experiment more freely. When you know TDD will verify correctness, you can accept AI suggestions more confidently. When you know types will enforce contracts, you can refactor more aggressively.

Discipline isn't the opposite of vibe coding. It's what makes vibe coding sustainable.

Without discipline, vibe coding is a sugar rush: productive in the moment, disastrous over time. With discipline, vibe coding is a power tool: amplified capability with maintained quality.
