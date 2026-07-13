import * as SorobanRpc from "@stellar/stellar-sdk";

export interface EventData {
  contractId: string;
  topics: string[];
  data: string;
  ledger: number;
}

export function handleSubscriptionCreated(event: EventData): void {
  const [, owner] = event.topics;
  const subId = event.data;
  console.log(
    `[Subscription] Created: ID=${subId}, Owner=${owner}, Ledger=${event.ledger}`
  );
}

export function handleSubscriptionUpdated(event: EventData): void {
  const [, owner] = event.topics;
  const subId = event.data;
  console.log(
    `[Subscription] Updated: ID=${subId}, Owner=${owner}, Ledger=${event.ledger}`
  );
}

export function handleSubscriptionCancelled(event: EventData): void {
  const [, owner] = event.topics;
  const subId = event.data;
  console.log(
    `[Subscription] Cancelled: ID=${subId}, Owner=${owner}, Ledger=${event.ledger}`
  );
}

export function handleSubscriptionPaused(event: EventData): void {
  const [, owner] = event.topics;
  const subId = event.data;
  console.log(
    `[Subscription] Paused: ID=${subId}, Owner=${owner}, Ledger=${event.ledger}`
  );
}

export function handlePaymentConfirmed(event: EventData): void {
  const [, owner] = event.topics;
  const subId = event.data;
  console.log(
    `[Payment] Confirmed: SubID=${subId}, Owner=${owner}, Ledger=${event.ledger}`
  );
}
