-- Standalone members have no project MCP policy to inherit.
-- Keep existing grants and leave project member migration to its own policy.
UPDATE chat_session_agents
SET execution_config = json_set(execution_config, '$.mcp', json('{"mcpServers":{}}'))
WHERE project_member_id IS NULL
  AND (json_type(execution_config, '$.mcp') IS NULL
       OR json_type(execution_config, '$.mcp') = 'null');
