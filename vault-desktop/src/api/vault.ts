import { invoke } from "@tauri-apps/api/core";

// Vault operations
export async function initVault(masterPassword: string): Promise<void> {
  return invoke("init_vault", { masterPassword });
}

export async function unlockVault(masterPassword: string): Promise<void> {
  return invoke("unlock_vault", { masterPassword });
}

export async function lockVault(): Promise<void> {
  return invoke("lock_vault");
}

export async function isUnlocked(): Promise<boolean> {
  return invoke("is_unlocked");
}

export async function vaultExists(): Promise<boolean> {
  return invoke("vault_exists");
}
