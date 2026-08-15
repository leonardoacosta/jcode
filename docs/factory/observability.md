# Observability

> Status: proposed factory requirement

Agent observability should capture traces, turns, context changes, tool calls, tool results, model/provider selection, file mutations, subagent relationships, latency, token metrics, gate results, retries, approvals, and final artifacts.

The key question is causal: why did this change happen, which worker produced it, what context and tools were used, what checks ran, and why was it accepted? Jcode already records substantial session, swarm, memory, and workflow state; the factory target is a unified replayable run trace.
