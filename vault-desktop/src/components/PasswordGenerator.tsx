import { useState } from "react";
import { generatePassword, PasswordOptions, GeneratedPassword } from "../api";

export default function PasswordGenerator() {
  const [options, setOptions] = useState<PasswordOptions>({
    length: 16,
    include_symbols: true,
  });
  const [result, setResult] = useState<GeneratedPassword | null>(null);
  const [copied, setCopied] = useState(false);

  async function handleGenerate() {
    try {
      const generated = await generatePassword(options);
      setResult(generated);
      setCopied(false);
    } catch (e) {
      console.error("Failed to generate password:", e);
    }
  }

  function copyToClipboard() {
    if (result?.password) {
      navigator.clipboard.writeText(result.password);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  }

  function updateOption<K extends keyof PasswordOptions>(
    key: K,
    value: PasswordOptions[K]
  ) {
    setOptions({ ...options, [key]: value });
  }

  const strengthColor =
    result?.strength.score === undefined
      ? ""
      : result.strength.score >= 3
      ? "strong"
      : result.strength.score >= 2
      ? "medium"
      : "weak";

  return (
    <div className="password-generator">
      <h2>Password Generator</h2>

      <div className="generator-options">
        <div className="option-row">
          <label>Length: {options.length}</label>
          <input
            type="range"
            min="8"
            max="64"
            value={options.length}
            onChange={(e) =>
              updateOption("length", parseInt(e.target.value))
            }
          />
        </div>


        <div className="option-row checkbox">
          <input
            type="checkbox"
            id="symbols"
            checked={options.include_symbols}
            onChange={(e) =>
              updateOption("include_symbols", e.target.checked)
            }
          />
          <label htmlFor="symbols">Symbols (!@#$...)</label>
        </div>
      </div>

      <button onClick={handleGenerate} className="btn-primary">
        Generate Password
      </button>

      {result && (
        <div className="generator-result">
          <div className="password-display">
            <code>{result.password}</code>
            <button onClick={copyToClipboard}>
              {copied ? "Copied!" : "Copy"}
            </button>
          </div>
          <div className={`strength-indicator ${strengthColor}`}>
            Strength: {result.strength.feedback}
          </div>
        </div>
      )}
    </div>
  );
}
