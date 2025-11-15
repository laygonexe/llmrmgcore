# `llmrmgcore`

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)

> Rust LLM-driven deterministic graph core (v0 prototype).

---
## Executive Summary

What do you get when you combine the creativity of today's LLMs with the deterministic, auto-enforcement of invariants from RMGs and build it all on top of an immutable history? 

> **Safe, debuggable, auditable, long-term memory for AI that's anti-hallucination by construction.**

### The AI Brain Upgrade Everyone's Been Waiting For

Imagine today's AI chatbots (Gemini, Claude, Grok, ChatGPT, etc.) as brilliant but forgetful storytellers. They're great at coming up with ideas on the fly, but they "hallucinate" (make stuff up), forget conversations, and can't be trusted for serious tasks like medical advice or business decisions. This repository offers a solution: a smart way to fix that by teaming up two tech worlds: the creative chaos of AI language models with a super-organized "knowledge graph" system that's like a bulletproof database on steroids.

### The Core Problems `librmgcore` Solves

#### Forgetfulness

Normal AI has to start fresh every time and has no real long-term memory.

#### Mistakes

LLMs spit out confident nonsense without checks.

#### Unreliability

There's no way to prove an AI's answer is safe or consistent over time.

### How `librmgcore` Works (Super Simple Version)

#### The Creative Side (LLMs) 

LLMs act as "idea generators." You ask something like "Explain graph rewriting," and the LLM proposes a structured plan (in a special code-like format) for how to add that info to a shared "brain" graph. Think of it as a web of connected ideas, facts, and decisions. Sort of like how your own mind works.

#### The Reliable Side (RMG Core)

The RMG core is a deterministic computer (results are fixed and exactly predictable for a given input) system that checks the LLM's proposal against rules like "*No contradictions*," "*No personal data leaks*," or "*Fits the knowledge structure.*" If it passes? It adds the update permanently. If not? Rejected, no harm done. **The RMG is the key that enables automatic enforcement of safety policies**. It's what makes it safe, auditable, and anti-hallucination by construction.

#### The Magic Glue (Versioning) 

Every change gets logged like Git (the tool programmers use for tracking code history). By borrowing ideas from Git, we can "rewind" time and see exactly what an AI was thinking, branch into "what-if" scenarios, or audit why a decision was made.

### Why `librmgcore` a Game-Changer for Everyone

#### Real Long-Term Memory

AI could finally remember everything across chats. Build a personal knowledge base that grows, like a smart notebook that learns from you.

#### Trustworthy Answers

Built-in checks mean fewer wild errors. For example, in a medical app, it could prove "This advice follows safety rules."

#### Superpowers Unlocked 

Time-travel debugging (fix mistakes by perfectly replaying history), collaborative brains (multiple AI agents team up without chaos), and safe creativity (we brainstorm wildly, but the core keeps it real).

#### Everyday Wins 

> ***Less "oops!";***
> ***More "eureka!"***

Safer AI in finance, science, or even creative writing. Everybody would benefit from safer, reliable AI that we can trust.

**In short:** `librmgcore` is a library that turns flaky AIs into dependable partners. Built from tools like graph rewriting (math for editing idea webs) and Git-style tracking, and it's already prototyped. 

**Excited?** This could be what makes AI feel less like a gimmick and more like a superpower!

---
## Overview

This repository contains the source code for `llmrmgcore`, a `v0` prototype for a deterministic system where a Large Language Model (LLM) acts as a "Heuristic Oracle", proposing graph rewrites to change a verifiable Recursive Meta-Graph (RMG) Core.

> **The core architectural principle is the separation of stochastic, generative intelligence (the LLM) from deterministic, verifiable logic (the Rust core).**

**The LLM Oracle** (`LlmOracle` trait) is responsible for interpreting natural language commands and proposing changes to the graph. These proposals take the form of a formal `DpoRule` (Double-Pushout Rule) object.

**The RMG Core** (`RmgCore` trait) is the source of truth. It receives `DpoRule` proposals, validates them against a set of strict, hardcoded invariants, and only applies them if they are valid. The application of a rule is a deterministic process; the same rule applied to the same graph state will always produce the identical resulting state.

> **This hybrid approach aims to leverage the creative power of LLMs while guaranteeing the integrity and predictability of the underlying system.**

### Key Concepts

#### Deterministic Core

The `RmgCore`'s `apply` method is purely deterministic.

#### Invariant Validation

The core validates every proposed change against a set of domain-specific invariants (e.g., `no_orphan_messages`, `well_typed_edges`, `immutable_history`).

#### Typed Schema

The graph's domain is strongly typed using Rust enums (`NodeType`, `EdgeType`) to prevent schema violations at a fundamental level.

#### Separation of Concerns

The non-deterministic, "creative" work is isolated in the Oracle, while the "bookkeeping" and state management is handled by the deterministic Core.

---
## `v0` Prototype Demonstration

This repository currently comes with a "`v0`", proof-of-concept toy model which demonstrates that the idea behind `librmgcore` works. 

By providing a simulated LLM proposal for changes to the RMG core, the demo produces the same output, bit-for-bit, no matter what computer is used to run the experiment. 

In other words, this proof-of-concept demonstrates that the outcome is deterministic.

### Example: Conversation & Decision Log

The domain modeled by this prototype is a **Conversation & Decision Log**. Think of it as a structured, machine-readable representation of a collaborative process, like a project meeting or a technical support thread.

#### Domain Model: The Memory Graph

The prototype demonstrates the technology that let AI perfectly remember every detail of a given conversation log. What sort of data structure does `librmgcore` provide and how does the AI read and write that information. 
##### Who, What, Where, When: Core Conversation Entities

The AI needs to remember the messages and how they were organized. It would reason that **Conversations** are organized by **Thread**. An **Actor** uses their **Turn** to submit a **Message** to the conversation. A message has a **Timestamp** that represents when they were sent.

```text
Conversation -> Thread -> Message
```
> *How messages are organized*

##### Why: Semantic Context

The AI needs to store more than just the messages themselves. It also needs to extract _semantics_.

- **Concept** is what is being discussed
- **Task** is what needs to be done
- **Decision** is what was decided

For example, a **Message** is `AUTHORED_BY` an **Actor**. A **Message** can `CREATE_TASK`. A **Decision** can `APPLY_TO` a **Thread**, and so on. The semantic context are the reasons and actions captured by the text logged in the conversation's messages.
##### How: Invariants

A **Policy** is a rule that governs the conversation. For example, a **policy** declares who may post  **Messages** to a given **Thread**. Or, a **policy** declares the maximum length for a **Message**. These are invariants that must be deterministically enforced to guarantee that the model can be trusted to provide safe, accurate data.

##### The Recursive Meta-Graph

Together, these concepts are represented by a graph. The _nouns_ become the nodes in the graph. The _verbs_ become the graph's edges. The _adjectives_ are semantic context, which become entities stored in an edge. In a RMG, everything is itself a graph, all the way down. That allows for a rich and complex web of interconnected entities and provides the mathematics that prove the determinism and safety of graph rewrites.

The model creates a ***semantically rich, queryable knowledge graph*** of who said what, what it was about, what decisions were made, and what actions resulted from it. 

### Goal: Demonstrate Deterministic Memory

The model described in the previous section is exactly what an AI would construct and use to remember the details of a particular conversation. The goal of the prototype is to demonstrate how the model is deterministically constructed (given the same inputs, the result is always exactly the same), bit-for-bit.

---

## The Significance of This Experiment

The feasibility of `librmgcore` is demonstrated by `v0`, a toy solution that successfully separates the creative but chaotic LLM from a logical, deterministic core and fuses them together to result in deterministic outcome, bit-for-bit every time.

This experiment is significant because it tackles one of the most fundamental problems in applied AI: LLMs are powerful but unreliable. You cannot trust them to directly manage critical data because they are non-deterministic and can "hallucinate", misunderstand commands, or make mistakes, leading to irreversible errors. The goal of this experiment is to demonstrate how `librmgcore` makes those problems impossible by construction.

#### ✅ Separation of Concerns 

By design, `librmgcore` works like a "designated driver"; it works by taking the keys away from LLMs (the Oracles). LLMs are allowed to say where they want to go, but they can't get behind the wheel and drive there.  In other words, LLMs can propose changes to their memory graphs in a formal, structured way (a `DpoRule`), but the safe, deterministic Rust-based `RmgCore` enforces invariants as it carries out the database operations the LLM proposes deterministically thanks to the inherent properties of how the graph rewrites work.

#### ✅ Verifiable Safety Enforcement

The Core validates every single proposal against a set of hardcoded, unbreakable rules (the invariants). If the LLM proposes a rule that would corrupt the graph's structure (e.g., creating an orphan message) or that would violate a domain constraint (e.g., deleting a message when history is meant to be immutable), the proposal is rejected. This guarantees the integrity of the system.

Thanks to how these invariants are stored–as artifacts stored in the memory graph itself–the act of constructing the graph becomes reproducible. At any given point in time, the rules are known. Replaying a conversation from 5 years ago automatically enforces the exact rules from that time, providing perfect reproduction by construction. 

#### ✅ Achieves Determinism

Because only the Core can modify the graph, and its `apply` method is deterministic, the system's state is always predictable and auditable. Given the same starting point and the same rule, the result will be identical, every time. **This is impossible with a pure LLM-based system.**

In short, this experiment is a practical step towards building safe, reliable, and auditable AI systems. It demonstrates a pattern for combining the creative, language-understanding capabilities of LLMs with the rigor and safety of formal, deterministic systems. It's a model for how humans can safely collaborate with AI on complex tasks without handing over ultimate authority.

---
## Don't Trust; Verify: How to Run the Experiment Yourself

### Prerequisites

-   [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
-   `git`

### 1. Clone the Repository

```bash
git clone https://github.com/flyingrobots/librmgcore.git
cd llmrmgcore
```

### 2. Build the Project

This command will download all dependencies and compile the project.

```bash
cargo build
```

### 3. Run the Tests

The project includes a comprehensive test suite that verifies the core logic, the mock oracle, the end-to-end integration, and the deterministic nature of the core.

```bash
cargo test
```

You should see all tests pass:

```
running 4 tests
test test_full_round_trip ... ok
test test_core_is_deterministic ... ok
test test_mock_oracle_propose_rule ... ok
test test_v0_apply_create_task_rule ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

---
## Project Status

This is a `v0` prototype. The core concepts are implemented and tested. The next logical steps for evolving the prototype include:

### Roadmap

This proof-of-concept works; 

Now, the goal is to fill in the missing pieces by reusing work from other active open source projects that the creator of `librmgcore` also created. [**GATOS** (Git As The Operating Surface)](https://github.com/flyingrobots/gatos) will provide most of the foundational technology that will power `librmgcore`, therefore the two projects virtually share a roadmap.

##### Implement a true DPO engine

Replace the naive, additive `apply_rewrite` method with a proper Double-Pushout (DPO) graph transformation.

##### Implement the `query` method

Build out the `RmgCore::query` method with a proper subgraph matching algorithm.

##### Integrate a real LLM

Replace the `MockOracle` with an implementation that calls a real LLM API.
