#!/usr/bin/env node
// Runs the chartlet MCP server over stdio. Standard output carries the protocol only.

import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";

import { createServer } from "./mcp.mjs";

await createServer().connect(new StdioServerTransport());
