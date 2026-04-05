import { invoke } from "@tauri-apps/api/core";

export interface PasswordStrength {
  score: number; // 0-4
  feedback: string;
}

export interface GeneratedPassword {
  password: string;
  strength: PasswordStrength;
}

export interface GeneratedPassphrase {
  passphrase: string;
  strength: PasswordStrength;
}

export interface PasswordOptions {
  length: number;
  include_symbols: boolean;
}

export interface PassphraseOptions {
  word_count: number;
}

export async function generatePassword(
  options: PasswordOptions
): Promise<GeneratedPassword> {
  return invoke("generate_password_cmd", { options });
}

export async function generatePassphrase(
  options: PassphraseOptions
): Promise<GeneratedPassphrase> {
  return invoke("generate_passphrase_cmd", { options });
}

export async function checkPasswordStrength(
  password: string
): Promise<PasswordStrength> {
  return invoke("check_password_strength", { password });
}
