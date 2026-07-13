import * as SorobanRpc from "@stellar/stellar-sdk";
import { config } from "./config";
import {
  handleSubscriptionCreated,
  handleSubscriptionUpdated,
  handleSubscriptionCancelled,
  handleSubscriptionPaused,
  handlePaymentConfirmed,
} from "./handlers/subscription";
import {
  handleMetadataAdded,
  handleMetadataUpdated,
} from "./handlers/metadata";
import {
  handleRuleCreated,
  handleRuleUpdated,
  handleRulePaused,
  handleRuleDeleted,
  handleRuleExecuted,
} from "./handlers/rule";

interface EventData {
  contractId: string;
  topics: string[];
  data: string;
  ledger: number;
}

const EVENT_HANDLERS: Record<
  string,
  (event: EventData) => void
> = {
  sub_crtd: handleSubscriptionCreated,
  sub_upd: handleSubscriptionUpdated,
  sub_can: handleSubscriptionCancelled,
  sub_psd: handleSubscriptionPaused,
  pay_conf: handlePaymentConfirmed,
  meta_add: handleMetadataAdded,
  meta_upd: handleMetadataUpdated,
  rule_cr: handleRuleCreated,
  rule_up: handleRuleUpdated,
  rule_ps: handleRulePaused,
  rule_dl: handleRuleDeleted,
  rule_ex: handleRuleExecuted,
};

async function processEvents(
  server: SorobanRpc.Server,
  contractId: string,
  startLedger: number
): Promise<number> {
  try {
    const events = await server.getEvents({
      contractId,
      startLedger,
      limit: 100,
    });

    for (const event of events.events) {
      const topics = event.topics.map((t) => t.value().toString());
      const topic0 = topics[0];
      const handler = EVENT_HANDLERS[topic0];

      if (handler) {
        handler({
          contractId,
          topics,
          data: event.data.value().toString(),
          ledger: event.ledgerLedger || startLedger,
        });
      }
    }

    return startLedger + 1;
  } catch (error) {
    console.error(`Error processing events from ledger ${startLedger}:`, error);
    return startLedger;
  }
}

async function main(): Promise<void> {
  console.log("Starting Strelligence Event Indexer...");
  console.log(`RPC URL: ${config.rpcUrl}`);
  console.log(`Contracts:`);
  console.log(`  Recurring Registry: ${config.contracts.recurringRegistry}`);
  console.log(`  Metadata Registry:  ${config.contracts.metadataRegistry}`);
  console.log(`  Automation Rules:   ${config.contracts.automationRules}`);
  console.log("");

  const server = new SorobanRpc.Server(config.rpcUrl);
  let currentLedger = config.startLedger;

  while (true) {
    const contracts = [
      config.contracts.recurringRegistry,
      config.contracts.metadataRegistry,
      config.contracts.automationRules,
    ].filter((c) => c.length > 0);

    for (const contractId of contracts) {
      currentLedger = await processEvents(server, contractId, currentLedger);
    }

    await new Promise((resolve) =>
      setTimeout(resolve, config.pollingIntervalMs)
    );
  }
}

main().catch(console.error);
