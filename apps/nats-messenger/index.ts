import { resolveConfig } from "./config";
import { createService } from "./service";
import { createNatsTool } from "./tool";

import type { NatsConfig } from "./types";

const plugin = {
  id: "nats-messenger",
  name: "NATS Messenger",

  register(api: any) {
    const rawConfig: Partial<NatsConfig> = api.pluginConfig ?? {};
    const config = resolveConfig(rawConfig);

    if (!config.enabled) {
      api.logger.info("nats-messenger: disabled by config");
      return;
    }

    const { service, handle } = createService(config, api.runtime, api.logger);

    api.registerService(service);
    api.registerTool(createNatsTool(config.agentName, handle));

    api.logger.info(
      `nats-messenger: registered (agent=${config.agentName}, subjects=${config.subjects.length})`,
    );
  },
};

export default plugin;
