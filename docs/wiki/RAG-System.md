# RAG System

The Retrieval Augmented Generation (RAG) system is a core component of the Babylon project, providing efficient object management, embedding generation, and context handling for AI interactions.

## Overview

The RAG system enables the game to maintain a large number of objects while efficiently managing which ones are actively used in the AI context window. It consists of several key components that work together to provide a seamless experience.

## Implementation Status

### Completed Components

#### 1. Object Lifecycle Management System

The lifecycle management system, implemented in `src/babylon/rag/lifecycle.py`, handles the state transitions of objects between different tiers:

- **Immediate Context**: 20-30 objects that are currently being used
- **Active Cache**: 100-200 objects that were recently used
- **Background Context**: 300-500 objects that might be needed soon
- **Inactive**: Objects that are not currently needed

Key features:
- State transition logic based on access patterns
- Memory pressure management
- Performance metrics tracking
- Access count decay over time

#### 2. Embeddings and Debeddings Implementation

The embedding system, implemented in `src/babylon/rag/embeddings.py`, handles the creation and management of vector embeddings for objects:

- Connects to OpenAI API for real-time embeddings
- Implements caching with LRU eviction policy
- Supports batch processing for efficiency
- Features comprehensive error handling and recovery
- Supports concurrent operations with async/await
- Includes detailed metrics collection

#### 3. Working Set Optimization

The working set optimization ensures that the most relevant objects are kept in the appropriate tiers:

- Implements tiered context management
- Includes promotion/demotion logic between tiers
- Tracks access counts and implements decay
- Optimizes memory usage based on object relevance

### Completed Components

#### 1. Object Lifecycle Management System

The lifecycle management system, implemented in `src/babylon/rag/lifecycle.py`, handles the state transitions of objects between different tiers:

- **Immediate Context**: 20-30 objects that are currently being used
- **Active Cache**: 100-200 objects that were recently used
- **Background Context**: 300-500 objects that might be needed soon
- **Inactive**: Objects that are not currently needed

Key features:
- State transition logic based on access patterns
- Memory pressure management
- Performance metrics tracking
- Access count decay over time

#### 2. Embeddings and Debeddings Implementation

The embedding system, implemented in `src/babylon/rag/embeddings.py`, handles the creation and management of vector embeddings for objects:

- Connects to OpenAI API for real-time embeddings
- Implements caching with LRU eviction policy
- Supports batch processing for efficiency
- Features comprehensive error handling and recovery
- Supports concurrent operations with async/await
- Includes detailed metrics collection

#### 3. Working Set Optimization

The working set optimization ensures that the most relevant objects are kept in the appropriate tiers:

- Implements tiered context management
- Includes promotion/demotion logic between tiers
- Tracks access counts and implements decay
- Optimizes memory usage based on object relevance

#### 4. Pre-embeddings System

The pre-embeddings system, implemented in `src/babylon/rag/pre_embeddings/`, handles preprocessing of content before embedding:

- **ContentPreprocessor**: Normalizes and cleans text with configurable options
- **ChunkingStrategy**: Divides content intelligently with fixed-size and semantic chunking
- **EmbeddingCacheManager**: Reduces duplicate operations with LRU caching
- **PreEmbeddingsManager**: Integrates all components with lifecycle management

Key features:
- Comprehensive text normalization and cleaning
- Intelligent content chunking for optimal embedding
- Caching to reduce duplicate embedding operations
- Batch processing for efficiency
- Detailed metrics collection
- Configurable through dedicated config classes
- Integration with Object Lifecycle Management

### Pending Components

#### 2. Context Window Management

Context window management will ensure that the total token usage stays within the limits of the AI model:

- Token counting and tracking
- Strategies for staying within token limits (150,000 tokens at 75% capacity)
- Automatic summarization when approaching limits
- Prioritization of content based on relevance

#### 3. Priority Queuing Implementation

Priority queuing will enhance the lifecycle management by adding importance-based decisions:

- Configurable priority levels for different object types
- Priority-based promotion/demotion logic
- Importance scoring based on object relationships
- Dynamic priority adjustment based on game state

## Technical Debt and Issues

- Type errors in lifecycle management related to handling `str | None` types
- Potential issues in ChromaDB Manager with the `heartbeat()` method call

## Integration with Other Systems

The RAG system integrates with:

- **ChromaDB**: For vector storage and similarity search
- **Metrics Collection**: For performance tracking and optimization
- **Entity Registry**: For managing game objects

## Next Steps

1. Implement the pre-embeddings system
2. Develop context window management
3. Add priority queuing
4. Fix type errors and other technical debt
5. Enhance testing with stress tests and performance benchmarks
