---

# Generic Application Intelligence Platform

## Comprehensive Module Design

### For Codebase Understanding, Business Behavior Explanation, Runtime Diagnosis, and Evidence-Based Root Cause Analysis

---


## Table of Contents

1. [Purpose](#1-purpose)
2. [Core Design Principle](#2-core-design-principle)
3. [Scope of the Platform](#3-scope-of-the-platform)
4. [High-Level Architecture](#4-high-level-architecture)
5. [High-Level Data Flow](#5-high-level-data-flow)
6. [Supporting Platform Layers](#6-supporting-platform-layers)
7. [Core Runtime Modules](#7-core-runtime-modules)
8. [How the Modules Work Together](#8-how-the-modules-work-together)
9. [Canonical Scenario Walkthroughs](#9-canonical-scenario-walkthroughs)
10. [Genericity Requirements](#10-genericity-requirements)
11. [Where Application-Specific Knowledge Still Lives](#11-where-application-specific-knowledge-still-lives)
12. [Data Contracts Between Modules](#12-data-contracts-between-modules)
13. [Non-Functional Requirements](#13-non-functional-requirements)
14. [MVP Boundaries](#14-mvp-boundaries)
15. [Suggested Initial Supported Reasoning Modes](#15-suggested-initial-supported-reasoning-modes)
16. [Suggested Initial Tooling](#16-suggested-initial-tooling)
17. [Final Build Guidance for Engineering Team](#17-final-build-guidance-for-engineering-team)
18. [Final Summary](#18-final-summary)
19. [Technology Stack Recommendations](#19-technology-stack-recommendations)

---

# 1. Purpose

This system is intended to answer questions such as:

* Why did this stacktrace happen?
* Why did this bill not go for discount approval?
* Why is this record not visible?
* Where can I see this bill?
* Why is this transaction still in draft state?
* Why is this amount calculated like this?
* Why did this workflow not move to the next stage?
* Which code path, query, config, or data condition caused this behavior?

The system must be **generic** and not limited to one application or one workflow.

It should work for:

* legacy monoliths
* modern services
* enterprise apps with config-heavy behavior
* business applications where behavior depends on code + DB + config + logs

The system must be able to:

1. understand the user’s question
2. identify the relevant business entity or technical artifact
3. discover the relevant code/data/config/runtime context
4. gather evidence
5. compare expected vs actual behavior
6. generate a grounded explanation

This architecture is designed so the system becomes **smart and generic**, while still supporting application-specific refinement where necessary.

---

# 2. Core Design Principle

Do **not** build separate hardcoded analyzers like:

* Calculation Analyzer
* Visibility Analyzer
* State Transition Analyzer
* Approval Analyzer
* Worklist Analyzer

Instead, build a **general diagnosis engine** with reusable generic modules.

Every question should go through the same high-level pipeline:

```text
User Question
→ Question Interpreter
→ Context Resolver
→ Evidence Collector
→ Constraint Evaluator
→ Explanation Composer
→ Final Answer
```

This allows the system to handle both known and unknown scenarios using the same reasoning flow.

---

# 3. Scope of the Platform

This platform should support all of the following classes of use cases.

## 3.1 Technical Diagnosis

Examples:

* Why did this stacktrace happen?
* Why is this API failing?
* Which code path caused this exception?
* Which query caused this SQL error?
* What is the probable root cause of this failure?

## 3.2 Business Behavior Explanation

Examples:

* Why is this bill amount like this?
* Why is patient amount zero and payer amount non-zero?
* Why did this discount not apply?
* Why is tax applied here but not there?
* Why was this value rounded differently?

## 3.3 Visibility and Location Questions

Examples:

* Why can’t I see this bill?
* Where can I see this draft bill?
* Why is this record not appearing in worklist?
* Why is this visible to one user but not another?
* Why is this item visible only in one unit or location?

## 3.4 State and Workflow Progression Questions

Examples:

* Why is this bill still in draft?
* Why is this request pending?
* Why did this order not move to the next stage?
* Why did approval not trigger?
* Why is this transaction not finalized?

## 3.5 Code and Flow Discovery

Examples:

* Where is this logic implemented?
* Which files and queries are involved in this flow?
* Which DAO or service handles this?
* Which tables are touched by this action?
* What depends on this class or query?

---

# 4. High-Level Architecture

The platform should be built around five core runtime modules:

1. **Question Interpreter**
2. **Context Resolver**
3. **Evidence Collector**
4. **Constraint Evaluator**
5. **Explanation Composer**

These runtime modules operate on top of supporting infrastructure:

* Repository Scanner
* Parser Layer
* Code/Route/Query Graph Builder
* Vector Index
* Metadata Store
* Optional DB Read Tool
* Optional Log Reader
* Optional Config Reader
* Optional Runtime Correlation Layer
* Adapter/Knowledge Pack Layer

---

# 5. High-Level Data Flow

```text
                    ┌──────────────────────┐
                    │    User / API / UI   │
                    └──────────┬───────────┘
                               │
                               v
                 ┌────────────────────────────┐
                 │   Question Interpreter     │
                 └──────────┬─────────────────┘
                            │
                            v
                 ┌────────────────────────────┐
                 │     Context Resolver       │
                 └──────────┬─────────────────┘
                            │
                            v
                 ┌────────────────────────────┐
                 │    Evidence Collector      │
                 └──────────┬─────────────────┘
                            │
                            v
                 ┌────────────────────────────┐
                 │   Constraint Evaluator     │
                 └──────────┬─────────────────┘
                            │
                            v
                 ┌────────────────────────────┐
                 │   Explanation Composer     │
                 └──────────┬─────────────────┘
                            │
                            v
                    ┌──────────────────────┐
                    │    Final Response     │
                    └──────────────────────┘
```

Supporting layers:

```text
Codebase / Docs / Config / Logs / DB
        ↓
Discovery + Parsing + Indexing + Graphing
        ↓
Graph Store / Vector Store / Metadata Store / SQL Store / Runtime Store
```

---

# 6. Supporting Platform Layers

Before describing the five runtime modules, define the supporting layers the runtime depends on.

---

## 6.1 Repository and Artifact Ingestion Layer

Purpose:

* ingest source code
* ingest docs
* ingest configs
* optionally ingest stacktraces, logs, query captures, runtime traces

Inputs:

* git repositories
* zip folders
* local folders
* docs and runbooks
* optional logs
* optional DB schema exports

Outputs:

* normalized file inventory
* typed artifacts
* parse-ready content

This layer must support:

* incremental updates and event-driven rescans (e.g., via webhooks) to avoid 'Cold Start' graph staleness
* reindexing on code changes
* metadata preservation (path, module, language, last modified, tags)

---

## 6.2 Parser and Discovery Layer

Purpose:

* parse code and structured files
* extract routes, classes, methods, references
* extract SQL and table references
* discover identifiers, configs, and probable entities

Supported strategies:

* Java parser
* JavaScript parser
* XML parser
* HTML parser
* properties parser
* fallback text parser
* JSP hybrid extractor
* config candidate detector
* identifier detector

This layer must not assume every language is cleanly parseable. For mixed legacy stacks like JSP, it should use a best-effort hybrid strategy.

---

## 6.3 Knowledge Graph and Retrieval Layer

Purpose:

* represent code structure and dependencies
* support traversal from one layer to another
* support semantic retrieval

Sub-stores:

* **Graph Store**: route/class/service/DAO/table dependencies
* **Vector Store**: semantic retrieval of code/docs
* **Metadata Store**: identifiers, table mappings, config candidates, file tags
* **SQL Store**: extracted queries, tables, parameters
* **Runtime Store**: optional logs, traces, stacktraces, issue history

---

## 6.4 Adapter / Knowledge Pack Layer

Purpose:

* refine generic discovery with application knowledge
* provide entity definitions, synonyms, config semantics, identifier aliases, workflow hints

This layer should be optional but supported.

Examples of what it may contain:

* `bill` maps to `bill_m`, `bill_detail_m`
* `visit_rid` has aliases like `visitRID`, `visitId`
* `discount approval` maps to routing config tables and specific modules
* `draft bill` is defined by a combination of status flags

This is not mandatory for the platform to function, but it increases diagnostic accuracy.

---

## 6.5 Tool Layer

The runtime modules may call tools such as:

* Repo search tool
* Graph traversal tool
* DB read-only query tool
* Log search tool
* Config lookup tool
* Trace/stack parser
* Query replay or query match tool

All tools must be:

* auditable
* read-only by default
* deterministic where possible
* environment-aware

---

# 7. Core Runtime Modules

Now the main design.

---

# 7.1 Question Interpreter

## Purpose

The Question Interpreter converts raw user input into a structured investigation request.

It determines:

* what the user is asking
* what type of reasoning is needed
* what entities or identifiers are present
* what is missing before investigation can continue

This is the first smart entry point into the system.

---

## Responsibilities

### A. Intent Classification

The system should classify questions into one or more reasoning patterns, such as:

* **Root Cause / Failure Diagnosis**

  * “Why did this fail?”
  * “Why this stacktrace?”
  * “Why did this API error happen?”

* **Business Behavior Explanation**

  * “Why is this amount like this?”
  * “Why did this calculation happen?”

* **Visibility / Location Explanation**

  * “Why can’t I see this?”
  * “Where should I see this?”
  * “Why not in worklist?”

* **State / Transition Explanation**

  * “Why is this still draft?”
  * “Why did it not move forward?”

* **Trace / Discovery**

  * “Where is this logic implemented?”
  * “Which query handles this?”

### B. Entity Extraction

It should identify business or technical entities such as:

* bill
* receipt
* invoice
* lab order
* visit
* approval request
* stacktrace
* exception
* API endpoint
* query

### C. Identifier Extraction

It should extract identifiers if present:

* bill number
* receipt number
* order number
* visit ID
* transaction ID
* request ID
* stacktrace class/method/line

### D. Missing Information Detection

If required information is missing, it should ask only the minimum needed to continue.

Examples:

* “Please provide bill number.”
* “Which unit/entity is this for?”
* “Please paste the stacktrace or at least the exception line.”

It must avoid asking too many questions.

---

## Inputs

* raw question text
* optional attachments (stacktrace, logs, screenshots, IDs)
* optional user role/context

## Outputs

A normalized investigation request, for example:

```json
{
  "intent": "visibility_explanation",
  "entity_type": "bill",
  "identifiers": {
    "bill_no": "BILL12345"
  },
  "confidence": 0.88,
  "requires_more_info": false,
  "reasoning_mode": "expected_vs_actual"
}
```

---

## Internal Components

### 1. Intent Resolver

Maps natural language to reasoning category.

### 2. Entity Resolver

Maps nouns/phrases to entities.

### 3. Identifier Extractor

Extracts technical/business identifiers.

### 4. Clarification Policy

Determines if the engine can continue or needs minimal follow-up.

---

# 7.2 Context Resolver

## Purpose

The Context Resolver determines **what parts of the system are relevant** to the question.

This is the module that makes the system feel intelligent.

It should automatically discover:

* relevant code paths
* related routes/screens/endpoints
* related queries and tables
* related configs
* likely data entities
* likely runtime evidence sources

It answers:

> “Where should I look to understand this question?”

---

## Responsibilities

### A. Locate Relevant Code Context

Find classes, methods, JSPs, controllers, services, DAOs, and queries likely involved.

### B. Build Investigation Scope

Determine the likely scope of investigation:

* which modules
* which files
* which routes
* which tables
* which config areas
* which logs/traces

### C. Resolve Flow Paths

Build likely cross-layer flow such as:

* UI → endpoint → service → DAO → DB
* stacktrace → method → dependent call chain
* business entity → relevant modules and tables

### D. Expand Context Intelligently

Start from initial signals and expand outward:

* from a bill number to billing module
* from a stacktrace method to call chain
* from a worklist question to worklist query, filters, and screen path

---

## Inputs

* structured investigation request from Question Interpreter
* graph store
* vector store
* metadata store
* adapter knowledge pack
* optional schema/config information

## Outputs

A context package, for example:

```json
{
  "entity": "bill",
  "probable_modules": ["billing", "approval"],
  "relevant_routes": ["/billing/viewDraftBill", "/billing/finalizeBill"],
  "relevant_code_nodes": [
    "BillingService.finalizeBill",
    "DiscountApprovalService.isApprovalRequired"
  ],
  "relevant_queries": [
    "BillingDao.findBillByNo",
    "ApprovalDao.findRoute"
  ],
  "relevant_tables": [
    "bill_m",
    "bill_detail_m",
    "approval_route_m"
  ],
  "reasoning_scope": "code_db_config"
}
```

---

## Internal Components

### 1. Code Context Locator

Uses graph + vector retrieval to find relevant code nodes.

### 2. Route and Screen Resolver

Finds UI and endpoint relationships.

### 3. Query and Table Resolver

Finds likely queries and DB tables.

### 4. Config Context Resolver

Finds likely config/master/flag sources.

### 5. Scope Expander

Determines how broad the search should be.

---

## Retrieval Strategy

The Context Resolver should combine:

* graph traversal
* lexical retrieval
* semantic retrieval
* identifier-based lookup
* adapter refinement

It must not rely on semantic search alone.

---

# 7.3 Evidence Collector

## Purpose

The Evidence Collector gathers all factual evidence required for diagnosis.

This module must be deterministic and evidence-first.

It should not speculate.
It should collect:

* code evidence
* query evidence
* DB evidence
* config evidence
* log evidence
* runtime correlation evidence

It answers:

> “What do we know for sure?”

---

## Responsibilities

### A. Collect Code Evidence

Fetch the exact methods, conditions, queries, and flow points relevant to the question.

Examples:

* `if (billStatus == DRAFT) ...`
* worklist query filter conditions
* approval threshold check
* route lookup logic

### B. Collect Query Evidence

Fetch relevant SQL or query builder fragments and table/column references.

### C. Collect DB Evidence

If read-only DB access exists:

* fetch the relevant transaction row(s)
* fetch related child rows
* fetch config rows
* fetch master mappings

### D. Collect Config Evidence

Fetch or infer relevant settings, flags, routes, thresholds, permissions, unit mappings.

### E. Collect Log / Runtime Evidence

If logs exist:

* search by request ID / transaction ID / time window / stacktrace signature
* extract relevant errors and sequence


### G. Collect Temporal & Audit Evidence

Fetch historical context for state-based queries (e.g., "Why is this still draft?"):

* fetch Change Data Capture (CDC) logs
* fetch audit table history (e.g., intermediate states that were rolled back)

### F. Normalize Evidence

Convert all evidence into normalized comparison-ready structures.

---

## Inputs

* context package from Context Resolver
* DB read tool
* log tool
* config reader
* graph/vector/metadata/sql stores

## Outputs

A structured evidence package, for example:

```json
{
  "code_evidence": [
    {
      "path": "billing/service/DiscountApprovalService.java",
      "line_range": "120-165",
      "condition": "approval required only if discount_pct > threshold and route exists"
    }
  ],
  "query_evidence": [
    {
      "query_id": "approval.findRoute",
      "tables": ["approval_route_m"],
      "filters": ["entity_rid", "payer_type", "active_flag"]
    }
  ],
  "db_evidence": {
    "bill": {
      "bill_no": "BILL12345",
      "discount_pct": 8,
      "status": "DRAFT",
      "entity_rid": 84
    },
    "approval_route": null
  },
  "config_evidence": {
    "approval_threshold": 10
  },
  "log_evidence": []
}
```

---

## Internal Components

### 1. Code Evidence Fetcher

Gets file snippets, method bodies, flow edges.

### 2. Query Evidence Fetcher

Gets SQL/query definitions and filters.

### 3. DB Evidence Fetcher

Runs safe read-only data fetches.

### 4. Config Evidence Fetcher

Retrieves settings and mappings.

### 5. Log Evidence Fetcher

Gets relevant errors/traces/events.

### 6. Evidence Normalizer

Converts everything into a common schema.

### 7. Evidence Pruner / Ranker (Token Management)

An optional but highly recommended step before returning evidence.
Determines exact excerpts to keep to avoid overflowing context limits.
Drops low-relevance logs or massive code files and replaces them with summaries or exact sliding-window slices.


---

## Safety Requirements & RBAC

* DB access must be read-only
* all queries must be audited
* evidence must preserve provenance
* logs must be filtered according to privacy/compliance rules
* **RBAC & Authorization**: Queries and evidence gathering must strictly execute under the security context of the user asking the question. This prevents leaking sensitive data.

---

# 7.4 Constraint Evaluator

## Purpose

The Constraint Evaluator performs the core diagnosis.

This is the module that compares:

* what **should** happen according to code and config
* what **actually** happened according to DB/logs/runtime data

This single generic module replaces many specific analyzers.

It answers:

> “Which rule, condition, or missing dependency explains the observed behavior?”

---

## Core Reasoning Model

All supported use cases reduce to:

### Expected vs Actual

* What behavior does the code/config define?
* What values/states actually exist?
* Which condition passed or failed?
* Which missing or mismatched dependency explains the result?

This works for:

* stacktrace analysis
* calculation explanation
* visibility problems
* state transition failures
* approval issues
* worklist exclusion
* permission-based visibility
* “where can I see this?”

---

## Responsibilities

### A. Build Expected Condition Set

From code/config/query evidence, derive expectations such as:

* status must be `FINALIZED`
* route must exist
* threshold must be crossed
* `active_flag = 1`
* user must belong to unit
* filter excludes draft records

### B. Build Actual State Set

From DB/log/runtime evidence, build actual values:

* current status = `DRAFT`
* threshold = 10, discount = 8
* route = null
* user unit = X, bill unit = Y
* log shows missing config or null pointer

### C. Compare and Rank Explanations

Determine:

* definitive cause
* likely causes
* missing evidence requiring follow-up

### D. Identify Blocking Conditions

Return the exact conditions that prevented the expected behavior.

### E. Produce Diagnostic Findings

Structured findings such as:

* threshold_not_crossed
* record_hidden_due_to_status
* visibility_blocked_by_unit_filter
* approval_route_missing
* null_dependency_from_config_gap
* stacktrace_triggered_by_missing_mapping

---

## Inputs

* evidence package
* adapter semantics
* reasoning policies
* workflow templates if available

## Outputs

A diagnostic result, for example:

```json
{
  "outcome": "root_cause_identified",
  "primary_cause": "approval route missing",
  "matched_constraints": [
    "discount_pct > threshold = false",
    "route_exists = false"
  ],
  "blocking_condition": "route_exists = false",
  "secondary_observations": [
    "bill still in draft state"
  ],
  "confidence": 0.91
}
```

---

## Internal Components

### 1. Condition Extractor

Extracts conditions from code/query/config evidence.

### 2. Actual State Builder

Builds actual values from DB/log/runtime evidence.

### 3. Comparator

Matches expected conditions against actual states.

### 4. Cause Ranker

Ranks explanations by strength of evidence.

### 5. Uncertainty Manager

Determines when evidence is insufficient and follow-up is needed.

---

## Example Comparison Patterns

### Calculation Question

Question:
“Why is this bill amount 17600?”

Expected:

* total = base - discount + tax + payer rules

Actual:

* base = 16000
* discount = 0
* tax = 1600
* payer adjustment = 0

Result:

* amount explained by tax addition

### Visibility Question

Question:
“Why can’t I see this bill?”

Expected:

* billing list shows only finalized bills for selected unit

Actual:

* bill status = DRAFT
* unit mismatch

Result:

* hidden because status and unit do not match list criteria

### State Question

Question:
“Why is this bill still draft?”

Expected:

* bill moves to finalized only after mandatory confirmation

Actual:

* mandatory confirmation flag not completed

Result:

* state transition blocked by missing prerequisite

### Stacktrace Question

Question:
“Why did this null pointer happen?”

Expected:

* route/config object must not be null

Actual:

* no row found for config lookup

Result:

* null pointer caused by missing configuration

---

# 7.5 Explanation Composer

## Purpose

The Explanation Composer turns the diagnostic result into a grounded, human-usable answer.

It must serve multiple audiences:

* support teams
* developers
* QA
* product owners
* business users
* administrators

It answers:

> “What happened, why, where, and what next?”

---

## Responsibilities

### A. Summarize Clearly

State the most likely or confirmed cause in plain language.

### B. Present Evidence

Show:

* relevant code files / methods
* DB/config values used
* relevant query/filter conditions
* relevant logs if applicable

### C. Explain the Logic

Describe how the system reached the conclusion:

* code says X
* data shows Y
* therefore Z happened

### D. Provide Next Step Guidance

Examples:

* check mapping table
* finalize bill first
* review threshold config
* look in draft bill screen under unit X
* provide missing request ID for deeper log analysis

### E. Tailor Response Depth

Support should get concise operational guidance.
Developers may get deeper trace details.

---

## Inputs

* diagnostic result
* evidence package
* audience or role
* output preferences

## Outputs

Human-readable structured answer.

Example:

```text
Root cause:
This bill did not go for discount approval because no active approval route exists for entity 84.

Why:
The code in DiscountApprovalService.java checks that a matching approval route exists before initiating approval.
The bill row exists and discount logic was evaluated, but the related route lookup returned no result.

Evidence:
- Code: billing/service/DiscountApprovalService.java lines 120–165
- Query: ApprovalDao.findRoute filters by entity_rid, payer_type, and active_flag
- DB: no active row found in approval_route_m for entity 84

Additional note:
The bill is also still in DRAFT status, which may affect visibility in some billing screens.

Next steps:
1. Verify approval route configuration for entity 84
2. Confirm whether the bill must be finalized before approval is shown
3. Re-run after route is configured
```

---

## Internal Components

### 1. Summary Generator

Creates concise conclusion.

### 2. Evidence Formatter

Formats code/DB/log evidence.

### 3. Logic Narrator

Explains expected vs actual reasoning.

### 4. Guidance Generator

Suggests next checks or likely actions.

### 5. Audience Adapter

Changes tone/detail depending on role.

---


---

# 7.6 Continuous Learning & Feedback Loop

## Purpose

The feedback loop ensures the platform gets smarter over time. If the Explanation Composer outputs a wrong answer and a user corrects it, that knowledge must not be lost.

## Responsibilities

### A. Feedback Ingress
Provide an API for users to rate conclusions or correct specific evidence facts.

### B. Knowledge Pack Generation
Convert verified corrections into persistent "Knowledge Pack" snippets.

### C. Context Weighting
Inject verified past RCAs (Root Cause Analyses) back into Qdrant or Graph Store so the Context Resolver can prioritize this past path if a similar stacktrace or user question occurs.

# 8. How the Modules Work Together

Now the full runtime flow.

---

## 8.1 Generic Investigation Pipeline

### Step 1: User asks a question

Examples:

* “Why can’t I see bill BILL12345?”
* “Why is this bill still draft?”
* “Why is the amount 17600?”
* “Here is the stacktrace, what happened?”
* “Where is discount approval logic implemented?”

### Step 2: Question Interpreter

Determines:

* intent
* entity
* identifiers
* whether more info is needed

### Step 3: Context Resolver

Finds:

* likely code path
* route/screen
* query
* table
* config
* logs to inspect

### Step 4: Evidence Collector

Fetches:

* code snippets
* query conditions
* DB rows
* config rows
* log excerpts

### Step 5: Constraint Evaluator

Compares:

* expected rules from code/query/config
* actual states from DB/log/runtime

Finds:

* blocking condition
* mismatch
* likely root cause

### Step 6: Explanation Composer

Produces:

* summary
* evidence
* reasoning
* next steps
* uncertainty if any

---

# 9. Canonical Scenario Walkthroughs

These are the examples your tech team should support conceptually.

---

## 9.1 Stacktrace-Based Diagnosis

### User Input

“NullPointerException in DiscountApprovalService line 142”

### Flow

* Question Interpreter: technical diagnosis, entity unknown, stacktrace mode
* Context Resolver: locate class/method, related services/config lookups/DAOs
* Evidence Collector: get code block, related query, optional config rows
* Constraint Evaluator: determine which required dependency was null and why
* Explanation Composer: explain stacktrace in business and technical terms

### Output

* root cause
* exact code path
* missing config/data condition
* next checks

---

## 9.2 Calculation Explanation

### User Input

“Why is this bill amount 17600?”

### Flow

* Question Interpreter: business explanation + calculation
* Context Resolver: billing code path, total calculation logic, tax rules, payer split rules
* Evidence Collector: fetch bill rows, pricing inputs, config values, calculation code
* Constraint Evaluator: reconstruct calculation and compare output
* Explanation Composer: explain which inputs and rules produced 17600

### Output

* step-by-step calculation explanation
* relevant config/rules
* code path where amount was computed

---

## 9.3 Visibility / Location Explanation

### User Input

“Why can’t I see this bill?” or “Where can I see this bill?”

### Flow

* Question Interpreter: visibility question
* Context Resolver: locate list/worklist/query/screen filters
* Evidence Collector: get bill status, unit, ownership, permissions, query filters
* Constraint Evaluator: determine which visibility condition excludes the record
* Explanation Composer: explain why hidden and where it should be visible

### Output

* “This bill is hidden from finalized billing list because it is still in DRAFT and belongs to Unit X”
* “You can view it under Draft Bills in Unit X”

---

## 9.4 State Transition Explanation

### User Input

“Why is this bill still draft?”

### Flow

* Question Interpreter: state explanation
* Context Resolver: locate finalize/save/transition logic
* Evidence Collector: fetch bill state, required flags, transitions, validations
* Constraint Evaluator: determine missing prerequisite
* Explanation Composer: explain which condition blocked state transition

### Output

* current state
* expected next state
* blocking condition
* exact code/config/data involved

---

## 9.5 Workflow Exclusion Example

### User Input

“Why is lab order 100234 not showing in worklist?”

### Flow

* Question Interpreter: visibility/workflow question
* Context Resolver: locate lab worklist query, filters, service flow
* Evidence Collector: fetch order row, status, location, department, cancellation flags, screen filters
* Constraint Evaluator: compare query conditions with row values
* Explanation Composer: explain exclusion condition and where to check it

### Output

* worklist query filters
* actual order state
* exact exclusion reason

---

## 9.6 Code Discovery Example

### User Input

“Where is discount approval logic implemented?”

### Flow

* Question Interpreter: trace/discovery
* Context Resolver: locate relevant modules/classes/services
* Evidence Collector: get flow edges and snippets
* Constraint Evaluator: not needed deeply here; mostly structural relevance ranking
* Explanation Composer: summarize the logic path and key files

### Output

* files
* methods
* route/flow
* related tables/queries

---

# 10. Genericity Requirements

To make the system generic, the architecture must avoid hardcoding domain-specific analyzers.

Instead, genericity comes from:

* common reasoning pipeline
* generalized intent types
* common evidence schema
* common expected-vs-actual evaluator
* optional knowledge packs/adapters

The platform should be able to work across:

* ERP
* HIMS/HIS
* CRM
* billing platforms
* workflow systems
* internal enterprise tools

It should not assume:

* specific tables
* specific entity names
* specific states
* specific framework
* specific UI layer

Those should be discovered or refined through adapters.

---

# 11. Where Application-Specific Knowledge Still Lives

The system is generic, but some application knowledge may still improve results:

## Knowledge Pack Examples

* entity definitions
* synonyms
* config table meanings
* state meanings
* common identifier aliases
* route or module hints
* source-of-truth table declarations

Examples:

* “draft bill” means `status = 0 and finalized_flag = N`
* `visitRID` and `visit_rid` refer to the same business identifier
* `approval_route_m` is the source-of-truth route mapping table

This layer should be pluggable and versioned.

---

# 12. Data Contracts Between Modules

To keep the system maintainable, each module should exchange explicit structured objects.

## 12.1 Investigation Request

Produced by Question Interpreter

Fields:

* intent
* entity type
* identifiers
* reasoning mode
* confidence
* missing info flags

## 12.2 Context Package

Produced by Context Resolver

Fields:

* probable modules
* relevant code nodes
* relevant routes
* relevant queries
* relevant tables
* relevant configs
* scope

## 12.3 Evidence Package

Produced by Evidence Collector

Fields:

* code evidence
* query evidence
* DB evidence
* config evidence
* log evidence
* provenance

## 12.4 Diagnostic Result

Produced by Constraint Evaluator

Fields:

* primary cause
* supporting conditions
* blocked condition
* alternative causes
* confidence
* unresolved gaps

## 12.5 Response Package

Produced by Explanation Composer

Fields:

* summary
* evidence
* reasoning narrative
* next actions
* confidence/disclaimer

---

# 13. Non-Functional Requirements

## Accuracy and Trust

* no unsupported claims
* must surface uncertainty
* must cite evidence sources internally

## Safety

* DB tools read-only
* no automated fixes in MVP
* audit every tool call

## Scalability

* support large codebases
* incremental indexing
* modular language parsers

## Extensibility

* new parsers can be added
* new tools can be plugged in
* adapters are optional but supported

## Observability

* log every investigation
* log retrieval scope
* log tool usage
* log confidence and failure cases

---

# 14. MVP Boundaries

For MVP, do not attempt full automation of everything.

## MVP Must Include

* Question Interpreter
* Context Resolver
* Evidence Collector
* Constraint Evaluator
* Explanation Composer
* repository indexing
* graph + vector retrieval
* code/query/table discovery
* structured answers

## MVP Optional

* DB read-only support for a limited number of entities
* stacktrace parsing
* small knowledge pack

## MVP Excluded Initially

* autonomous remediation
* DB writes
* advanced agent loops
* broad log correlation if no request IDs exist
* highly dynamic self-modifying workflows

---

# 15. Suggested Initial Supported Reasoning Modes

To keep implementation clean, start with these five:

1. **Trace**

   * where is this logic / path / query / dependency

2. **Explain Result**

   * why did this amount/result/value happen

3. **Explain Missing Visibility**

   * why can’t I see this / where should I see it

4. **Explain Blocked Transition**

   * why is it still draft/pending/not moved

5. **Diagnose Failure**

   * why did this error/stacktrace/failure happen

These five are enough to cover most business and support scenarios.

---

# 16. Suggested Initial Tooling

Minimum tools:

* repo search
* graph traversal
* query lookup
* optional DB read-only query runner
* optional stacktrace parser

Later tools:

* log search
* config diff
* issue/ticket history
* runtime trace correlation

---

# 17. Final Build Guidance for Engineering Team

## Do not build:

* separate analyzers for each use case
* hardcoded business-specific modules as the main architecture
* ungrounded free-form “agent reasoning” first
* a chat system without strong evidence flow

## Build:

* one generic investigation pipeline
* one strong context resolution layer
* one strong evidence model
* one generic expected-vs-actual evaluator
* one structured explanation layer

The power of the product will come from:

* good discovery
* good evidence gathering
* good comparison logic
* grounded answers

Not from adding more and more separate analyzers.

---

# 18. Final Summary

This platform should be implemented as a **generic application reasoning engine**.

It should not be positioned or built as:

* just code chat
* just RAG over docs
* just a debugging assistant

It should be built as:

> A system that understands a user’s question, discovers the relevant code/data/config/runtime context, gathers evidence, compares expected vs actual behavior, and explains why the application behaved the way it did.

That architecture supports:

* stacktrace diagnosis
* calculation explanation
* visibility explanation
* state transition explanation
* workflow debugging
* flow discovery
* business rule understanding

All through the same generic pipeline.

---

# 19. Technology Stack Recommendations

Yes. If the goal is **the fastest, strongest technical stack for this kind of platform**, and you do **not** care whether it matches your current team skills, this is the stack I would recommend.

This is for building a **generic application-intelligence engine** that can do:

* codebase understanding
* stacktrace diagnosis
* query/table tracing
* business-rule explanation
* visibility/state explanation
* optional DB/log correlation

---

# Recommended stack

## 1) Core backend: **Rust**

Build the main platform services in **Rust**.

Why:

* best fit for high-throughput indexing, parsing pipelines, graph/retrieval orchestration, and low-latency APIs
* strongest option if you want something fast, memory-efficient, and safe at scale
* also aligns well with tools like **Qdrant** and **Tantivy**, which are themselves Rust-based/search-oriented. ([Qdrant][1])

Use Rust for:

* ingestion service
* context resolver
* evidence collector
* constraint evaluator
* retrieval orchestrator
* API layer

---

## 2) Java parsing: **JavaParser**

For Java source understanding, use **JavaParser** instead of relying only on Tree-sitter.

Why:

* it is specifically built for Java
* it gives you a real Java AST
* it supports modern Java versions and advanced analysis functionality. ([GitHub][2])

Use JavaParser for:

* class/method extraction
* imports/inheritance
* call-site hints
* annotation extraction
* service/DAO detection
* code condition extraction

For a Java-heavy legacy system, this is the right choice.

---

## 3) Multi-language parsing: **Tree-sitter**

Use **Tree-sitter** for everything around the Java core:

* JavaScript / AngularJS
* HTML
* XML
* properties-like config handling via custom layers
* general incremental parsing support

Tree-sitter is an incremental parsing system that builds syntax trees efficiently and supports multi-language parsing workflows. ([tree-sitter.github.io][3])

Use Tree-sitter for:

* AngularJS controller/service extraction
* HTML/XML/config parsing
* general syntax-aware chunking
* mixed-language code intelligence support

### Important

Do **not** trust Tree-sitter alone for JSP.

For JSP:

* use a **custom JSP preprocessor/extractor**
* split JSP into:

  * HTML/text blocks
  * form/action/include/taglib extraction
  * embedded Java/scriptlet extraction where possible

JSP should be treated as a **hybrid format**, not a first-class parse target.

---

## 4) Lexical search: **Tantivy**

For exact/code-aware lexical retrieval, use **Tantivy**.

Why:

* Lucene-style full-text search engine library in Rust
* fast and lightweight
* ideal for exact lookup of:

  * class names
  * method names
  * column names
  * stacktrace fragments
  * SQL text
  * identifiers like bill/order/request IDs. ([Docs.rs][4])

Use Tantivy for:

* BM25 search
* exact token search
* stacktrace-to-code lookup
* precise schema/query matching

This is much better than trying to do all retrieval with vectors.

---

## 5) Vector search: **Qdrant**

For semantic retrieval, use **Qdrant**.

Why:

* open-source vector engine written in Rust
* supports fast vector search
* supports metadata filtering
* supports hybrid retrieval patterns, including dense+sparse/hybrid query workflows. ([Qdrant][1])

Use Qdrant for:

* semantic retrieval of code/doc chunks
* “find logic that means X even if names differ”
* natural-language codebase search
* retrieval over docs + generated summaries + knowledge packs

For your use case, Qdrant is a better fit than starting with a heavier OpenSearch deployment.

---

## 6) Graph store: **Neo4j-compatible graph, with abstraction**

Your graph layer should be **Cypher-based** and abstracted so you can run either **Neo4j Community** or **Memgraph**.

Why:

* you need graph traversal for:

  * JSP → endpoint → servlet → service → DAO → query → table
  * dependency and impact analysis
  * “where does this flow go?”
* **Neo4j** gives you the most established Cypher ecosystem and documentation. Cypher is its declarative graph query language. ([Graph Database & Analytics][5])
* **Memgraph** is open-source, Cypher-compatible, and positioned for high-performance/in-memory graph workloads. ([memgraph.com][6])

### My recommendation

* Start with **Neo4j-compatible schema and Cypher query model**
* Keep the graph adapter swappable
* If traversal performance becomes a bottleneck, switch runtime to **Memgraph**

That gives you maturity first and speed options later.

---

## 7) Metadata / control plane DB: **PostgreSQL**

Use **PostgreSQL** for platform metadata.

Store:

* repository registry
* indexing jobs
* file manifests
* normalized identifiers
* workflow runs
* adapter definitions
* audit logs
* tool execution history

---

## 8) Local model serving: **vLLM**

For self-hosted models, serve them with **vLLM**.

Why:

* production-grade model serving
* OpenAI-compatible API
* easy to swap local models under the same interface. ([vLLM][7])

Use vLLM for:

* local coding/diagnostic models
* private deployments
* hybrid routing behind one API contract

This is the right choice if you want local + cloud provider flexibility.

---

## 9) LLM strategy: **hybrid by design**

Do **not** tie the platform to one model vendor.

Use:

* **local models** for routine code retrieval, explanation, summarization
* **cloud frontier models** for hard multi-hop reasoning, long-context synthesis, and ambiguous investigations

The platform should talk to:

* local vLLM endpoint
* OpenAI-compatible providers
* Anthropic/OpenAI adapters as optional backends

This should be an abstraction at the API layer, not hardcoded in business logic.

---

## 10) Observability and correlation: **OpenTelemetry**

For anything involving logs, traces, and runtime diagnosis, standardize around **OpenTelemetry**.

Why:

* OpenTelemetry Java supports logs, metrics, traces
* Java instrumentation ecosystem exists
* log/trace context propagation and MDC injection are directly useful for request correlation. ([OpenTelemetry][8])

Use it for:

* request correlation IDs
* trace-to-log linking
* future runtime RCA
* consistent telemetry across services

If you want “why did this exact request fail?” this is the right base.

---

# Exact platform layout I would recommend

## Core services

* **Rust**

  * API gateway
  * Question Interpreter
  * Context Resolver
  * Evidence Collector
  * Constraint Evaluator
  * Explanation Composer
  * Index orchestration
  * Tool orchestration

## Parsing workers

* **Java worker**: JavaParser
* **Generic language worker**: Tree-sitter
* **JSP worker**: custom extractor
* **SQL extractor**: custom parser + heuristics + runtime capture later

## Storage

* **PostgreSQL**: metadata/control plane
* **Neo4j/Memgraph**: graph
* **Qdrant**: vectors
* **Tantivy**: lexical search

## Model layer

* **vLLM** for local
* provider adapters for cloud

## Telemetry

* **OpenTelemetry**

---

# Why this stack is the best fit

Because your problem is **not** “just build chat over files.”

Your problem needs four retrieval modes at once:

### 1. Syntax/AST understanding

Needed for:

* code path extraction
* call relationships
* rule extraction

That’s JavaParser + Tree-sitter. ([GitHub][2])

### 2. Exact lexical retrieval

Needed for:

* stacktraces
* identifiers
* SQL snippets
* method names

That’s Tantivy. ([Docs.rs][4])

### 3. Semantic retrieval

Needed for:

* “why is this amount like this?”
* “where is discount approval logic?”
* “find the relevant files even if naming is bad”

That’s Qdrant. ([Qdrant][1])

### 4. Structural traversal

Needed for:

* UI → service → DAO → DB tracing
* dependency reasoning
* impact analysis

That’s Neo4j/Memgraph. ([Graph Database & Analytics][5])

No single tool does all four well. That is why this stack is layered.

---

---

# MVP stack vs long-term stack

## MVP

* Rust API/orchestrator
* JavaParser
* Tree-sitter
* Tantivy
* Qdrant
* PostgreSQL
* Neo4j Community
* vLLM + one cloud fallback
* OpenTelemetry planning started

## Long-term

* same core stack
* add Memgraph option if graph speed matters
* add runtime query capture
* add DB read-only tool layer
* add log/trace correlation
* add IDE plugin / UI integration

---
