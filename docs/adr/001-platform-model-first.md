# ADR-001: Platform Model First

## Status
- [x] Proposed
- [x] Accepted
- [ ] Deprecated
- [ ] Superseded

## Context
The repository needed a single source of truth for describing the platform's services, deployables, resources, workflows, policies, and topologies. Without a centralized model, infrastructure configurations, deployment manifests, and documentation would drift out of sync, leading to inconsistency and operational risk.

Multiple approaches were considered:
1. Infrastructure-first (write Kubernetes manifests directly)
2. Documentation-first (maintain docs separately from code)
3. Platform model-first (declare platform in YAML, generate everything else)

## Decision
We adopted a **Platform Model First** approach where:

- `platform/model/` is the source of truth for platform metadata, deployables, topologies, workflows, resources, and environments
- service-local distributed semantics still belong in `services/<name>/model.yaml`, not in `platform/model/`
- the model is validated against JSON schemas before generation or delivery checks
- generated directories remain read-only, but the current repository does **not** yet generate every infra/doc artifact mechanically from the model
- platform generators and validators are a control-plane aid, not proof that every target-state platform capability is already implemented

### Rationale
1. **Consistency**: Single source of truth eliminates drift between code, infrastructure, and docs
2. **Automation**: Changes to the model automatically propagate to all consumers
3. **Verification**: Model can be validated against schemas before generating anything
4. **Topology independence**: The same services can be deployed in different topologies (local-dev, single-vps, k3s-microservices) without changing business logic
5. **Agent-friendly**: YAML models are easier for AI agents to read and modify than complex infrastructure code

### Implementation
- JSON Schema definitions in `platform/schema/` for each entity type
- YAML models in `platform/model/` validated against schemas
- Generators in `platform/generators/` produce infrastructure, SDKs, docs, and catalogs
- Validators in `platform/validators/` ensure model integrity
- `just validate-platform` runs model validation
- `just gen-platform` regenerates all artifacts

## Consequences
### What becomes easier
- Adding new deployables/resources/topology metadata in one place
- Checking platform-level consistency before delivery
- Keeping ownership and topology intent explicit
- Onboarding new developers to the intended control-plane structure

### What becomes more difficult
- Initial setup requires understanding the model schema
- Some changes require model updates first (extra step, but safer)
- Generator or validator drift can create false confidence if not checked against live code

### Trade-offs
- **Pros**: Consistency, automation, verification, topology independence
- **Cons**: Learning curve, upfront investment in generators/validators, risk of target-state docs outrunning code

## References
- `agent/codemap.yml` - Repository layout specification
- `agent/codemap.yml` - Module constraints and dependency rules
- `platform/schema/*.schema.json` - Model schemas
- `platform/model/` - Platform model definitions
