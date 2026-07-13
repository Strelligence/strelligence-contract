import * as SorobanRpc from "@stellar/stellar-sdk";

export interface EventData {
  contractId: string;
  topics: string[];
  data: string;
  ledger: number;
}

export function handleMetadataAdded(event: EventData): void {
  const [, owner] = event.topics;
  const txHash = event.data;
  console.log(
    `[Metadata] Added: TxHash=${txHash}, Owner=${owner}, Ledger=${event.ledger}`
  );
}

export function handleMetadataUpdated(event: EventData): void {
  const [, owner] = event.topics;
  const txHash = event.data;
  console.log(
    `[Metadata] Updated: TxHash=${txHash}, Owner=${owner}, Ledger=${event.ledger}`
  );
}
