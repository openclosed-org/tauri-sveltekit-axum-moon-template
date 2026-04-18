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

- `platform/model/` is the single source of truth for all platform definitions
- Services, deployables, resources, workflows, policies, topologies, and environments are all declared in YAML
- All infrastructure manifests (`infra/kubernetes/rendered/`), SDKs (`packages/sdk/`), documentation (`docs/generated/`), and catalogs (`platform/catalog/`) are **generated** from the model
- The model is validated against JSON schemas before generation
- Manual modifications to generated directories are forbidden; they must be regenerated from the model

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
- Adding new services: define in model, regenerate, done
- Changing topology: edit topology YAML, regenerate, deploy
- Keeping docs in sync with code: regenerate docs from model
- Onboarding new developers: read model to understand system

### What becomes more difficult
- Initial setup requires understanding the model schema
- Simple changes require model updates first (extra step, but safer)
- Generator bugs can block progress (mitigated by model validation)

### Trade-offs
- **Pros**: Consistency, automation, verification, topology independence
- **Cons**: Learning curve, upfront investment in generators/validators

## References
- `agent/codemap.yml` - Repository layout specification
- `agent/codemap.yml` - Module constraints and dependency rules
- `platform/schema/*.schema.json` - Model schemas
- `platform/model/` - Platform model definitions
