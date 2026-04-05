import { invoke } from "@tauri-apps/api/core";

export interface VaultEntry {
  id: number;
  title: string;
  username: string | null;
  password: string;
  url: string | null;
  notes: string | null;
  tags: string | null;
  created_at: number;
  updated_at: number;
}

export interface NewEntry {
  title: string;
  username?: string;
  password: string;
  url?: string;
  notes?: string;
  tags?: string[];
}

export interface EntryUpdate {
  id: number;
  title?: string;
  username?: string;
  password?: string;
  url?: string;
  notes?: string;
  tags?: string[];
}

export async function listEntries(search?: string): Promise<VaultEntry[]> {
  return invoke("list_entries", { search });
}

export async function getEntry(query: string): Promise<VaultEntry | null> {
  return invoke("get_entry", { query });
}

export async function addEntry(entry: NewEntry): Promise<number> {
  return invoke("add_entry", { entry });
}

export async function updateEntry(update: EntryUpdate): Promise<void> {
  return invoke("update_entry", { update });
}

export async function deleteEntry(id: number): Promise<void> {
  return invoke("delete_entry", { id });
}

export async function copyPassword(entryId: number): Promise<void> {
  return invoke("copy_password", { entryId });
}
