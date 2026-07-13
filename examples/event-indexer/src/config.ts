export const config = {
  rpcUrl: process.env.SOROBAN_RPC_URL || "https://soroban-testnet.stellar.org",
  networkPassphrase:
    process.env.SOROBAN_NETWORK_PASSPHRASE ||
    "Test SDF Network ; September 2015",
  contracts: {
    recurringRegistry: process.env.RECURRING_CONTRACT || "",
    metadataRegistry: process.env.METADATA_CONTRACT || "",
    automationRules: process.env.AUTOMATION_CONTRACT || "",
  },
  pollingIntervalMs: parseInt(process.env.POLLING_INTERVAL_MS || "5000", 10),
  startLedger: parseInt(process.env.START_LEDGER || "0", 10),
};
