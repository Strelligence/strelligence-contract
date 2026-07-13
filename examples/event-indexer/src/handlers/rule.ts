import * as SorobanRpc from "@stellar/stellar-sdk";

export interface EventData {
  contractId: string;
  topics: string[];
  data: string;
  ledger: number;
}

export function handleRuleCreated(event: EventData): void {
  const [, owner] = event.topics;
  const ruleId = event.data;
  console.log(
    `[Rule] Created: ID=${ruleId}, Owner=${owner}, Ledger=${event.ledger}`
  );
}

export function handleRuleUpdated(event: EventData): void {
  const [, owner] = event.topics;
  const ruleId = event.data;
  console.log(
    `[Rule] Updated: ID=${ruleId}, Owner=${owner}, Ledger=${event.ledger}`
  );
}

export function handleRulePaused(event: EventData): void {
  const [, owner] = event.topics;
  const ruleId = event.data;
  console.log(
    `[Rule] Paused: ID=${ruleId}, Owner=${owner}, Ledger=${event.ledger}`
  );
}

export function handleRuleDeleted(event: EventData): void {
  const [, owner] = event.topics;
  const ruleId = event.data;
  console.log(
    `[Rule] Deleted: ID=${ruleId}, Owner=${owner}, Ledger=${event.ledger}`
  );
}

export function handleRuleExecuted(event: EventData): void {
  const [, owner] = event.topics;
  const ruleId = event.data;
  console.log(
    `[Rule] Executed: ID=${ruleId}, Owner=${owner}, Ledger=${event.ledger}`
  );
}
