# sdk/typescript/src/tools.ts

- ToolExecutionContext · interface · L5-L10 — interface ToolExecutionContext
- ToolResult · interface · L12-L15 — interface ToolResult
- SdkTool · interface · L18-L23 — interface SdkTool extends SessionToolDefinition
- SessionToolsOptions · interface · L25-L32 — interface SessionToolsOptions
- Handler · type · L34-L34 — type Handler = { tool: SdkTool; validate: ValidateFunction };
- PreparedTools · interface · L35-L38 — interface PreparedTools
- prepareTools · function · L41-L72 — function prepareTools(options: SessionToolsOptions): PreparedTools
- ToolCallbacks · class · L75-L154 — class ToolCallbacks
- constructor · method · L80-L83 — constructor( private readonly submit: (sessionId: string, callId: string, result: ToolResult) => Promise<void>, private readonly report: (error: unknown) => void, )
- dispatch · method · L85-L103 — dispatch(sessionId: string, callId: string, name: string, input: unknown): void
- execute · method · L105-L138 — private async execute(handler: Handler | undefined, input: unknown, context: ToolExecutionContext, controller: AbortController): Promise<ToolResult>
- abort · method · L140-L148 — abort(sessionId?: string): void
- close · method · L150-L153 — close(): void
